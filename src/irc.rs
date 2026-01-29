use crate::model::{IrcConfig, UiEvent};
use crate::util::{contains_nick_word, ts_prefix};

use anyhow::{Context, Result};
use std::sync::mpsc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// Trait-object fix: dyn objects cannot contain two non-auto traits directly.
trait AsyncReadWrite: tokio::io::AsyncRead + tokio::io::AsyncWrite {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite> AsyncReadWrite for T {}
type DynStream = Box<dyn AsyncReadWrite + Unpin + Send>;

pub async fn irc_run(
    conn_id: u64,
    cfg: IrcConfig,
    ui_tx: mpsc::Sender<UiEvent>,
    mut raw_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
    raw_tx: tokio::sync::mpsc::UnboundedSender<String>,
) -> Result<()> {
    // Connection-local status buffer.
    let status = "Status".to_string();
    ui_tx.send(UiEvent::EnsureBuffer {
        conn_id,
        buffer: status.clone(),
        make_current: false,
    })
    .ok();

    ui_tx.send(UiEvent::Append {
        conn_id,
        buffer: status.clone(),
        line: format!(
            "{} *** Connecting to {}:{} {}",
            ts_prefix(),
            cfg.server,
            cfg.port,
            if cfg.tls { "(TLS)" } else { "" }
        ),
        bump_unread: false,
        bump_highlight: false,
    })
    .ok();

    // Connect TCP.
    let tcp = TcpStream::connect((cfg.server.as_str(), cfg.port))
        .await
        .with_context(|| format!("connect {}:{}", cfg.server, cfg.port))?;

    // Optionally wrap with TLS.
    let stream: DynStream = if cfg.tls {
        let cx = native_tls::TlsConnector::new().context("tls connector")?;
        let cx = tokio_native_tls::TlsConnector::from(cx);
        let tls = cx
            .connect(&cfg.server, tcp)
            .await
            .context("tls connect")?;
        Box::new(tls)
    } else {
        Box::new(tcp)
    };

    let (mut rd, mut wr) = tokio::io::split(stream);

    // Identify.
    send_line(&mut wr, &format!("NICK {}", cfg.nick)).await?;
    send_line(&mut wr, &format!("USER {} 0 * :hexrust", cfg.nick)).await?;

    ui_tx.send(UiEvent::Append {
        conn_id,
        buffer: status.clone(),
        line: format!("{} *** Socket connected. Waiting for welcome…", ts_prefix()),
        bump_unread: false,
        bump_highlight: false,
    })
    .ok();

    // Writer task: drains raw_rx and writes to the socket.
    let writer = tokio::spawn(async move {
        while let Some(line) = raw_rx.recv().await {
            if let Err(e) = send_line(&mut wr, &line).await {
                return Err(e);
            }
        }
        Ok::<(), anyhow::Error>(())
    });

    // Reader loop: parse incoming messages and forward to UI.
    let mut buf = Vec::<u8>::with_capacity(4096);
    let mut welcomed = false;
    let mut my_nick_net = cfg.nick.clone();

    loop {
        buf.clear();
        let n = read_one_line(&mut rd, &mut buf).await?;
        if n == 0 {
            break;
        }

        while buf.last() == Some(&b'\n') || buf.last() == Some(&b'\r') {
            buf.pop();
        }
        let line = String::from_utf8_lossy(&buf).to_string();
        if line.is_empty() {
            continue;
        }

        // Respond to PING quickly to stay connected.
        if let Some(rest) = line.strip_prefix("PING ") {
            let _ = raw_tx.send(format!("PONG {}", rest));
            continue;
        }

        // Join channel after welcome (001).
        if !welcomed && line.contains(" 001 ") {
            welcomed = true;

            let _ = ui_tx.send(UiEvent::EnsureBuffer {
                conn_id,
                buffer: cfg.initial_channel.clone(),
                make_current: true,
            });

            let _ = raw_tx.send(format!("JOIN {}", cfg.initial_channel));
            let _ = raw_tx.send(format!("NAMES {}", cfg.initial_channel));

            let _ = ui_tx.send(UiEvent::Append {
                conn_id,
                buffer: status.clone(),
                line: format!("{} *** Joining {} …", ts_prefix(), cfg.initial_channel),
                bump_unread: false,
                bump_highlight: false,
            });
        }

        route_irc_line(conn_id, &line, &mut my_nick_net, &ui_tx);
    }

    drop(raw_tx);
    let _ = writer.await;

    ui_tx.send(UiEvent::Append {
        conn_id,
        buffer: status.clone(),
        line: format!("{} *** Connection closed.", ts_prefix()),
        bump_unread: true,
        bump_highlight: false,
    })
    .ok();

    Ok(())
}

async fn read_one_line<R: tokio::io::AsyncRead + Unpin>(
    rd: &mut R,
    out: &mut Vec<u8>,
) -> Result<usize> {
    // Read until '\n' or EOF.
    let mut one = [0u8; 1];
    let mut total = 0usize;

    loop {
        let n = rd.read(&mut one).await?;
        if n == 0 {
            return Ok(total);
        }
        total += 1;
        out.push(one[0]);

        if one[0] == b'\n' {
            return Ok(total);
        }

        // Hard cap: prevent unbounded growth if a server misbehaves.
        if out.len() > 64 * 1024 {
            return Ok(total);
        }
    }
}

fn route_irc_line(conn_id: u64, line: &str, my_nick_net: &mut String, ui_tx: &mpsc::Sender<UiEvent>) {
    // Numeric NAMES reply: 353
    if let Some((chan, users)) = parse_names_reply(line) {
        let _ = ui_tx.send(UiEvent::SetUsers {
            conn_id,
            channel: chan,
            users,
        });
        return;
    }

    // Topic: 332
    if let Some((chan, topic)) = parse_topic_332(line) {
        let _ = ui_tx.send(UiEvent::EnsureBuffer {
            conn_id,
            buffer: chan.to_string(),
            make_current: false,
        });
        let _ = ui_tx.send(UiEvent::Append {
            conn_id,
            buffer: chan.to_string(),
            line: format!("{} *** topic: {}", ts_prefix(), topic),
            bump_unread: true,
            bump_highlight: false,
        });
        return;
    }

    // Prefix-based parsing.
    if let Some((prefix, rest)) = line.strip_prefix(':').and_then(|s| s.split_once(' ')) {
        let nick = prefix.split('!').next().unwrap_or(prefix).to_string();

        // NICK change
        if let Some(new_nick) = parse_nick_change(rest) {
            if *my_nick_net == nick {
                *my_nick_net = new_nick.clone();
                let _ = ui_tx.send(UiEvent::SetMyNick {
                    conn_id,
                    nick: new_nick.clone(),
                });
            }
            let _ = ui_tx.send(UiEvent::RenameUserEverywhere {
                conn_id,
                old: nick.clone(),
                new_: new_nick.clone(),
            });
            let _ = ui_tx.send(UiEvent::Append {
                conn_id,
                buffer: "Status".to_string(),
                line: format!(
                    "{} *** {} is now known as {}",
                    ts_prefix(),
                    nick,
                    new_nick
                ),
                bump_unread: true,
                bump_highlight: true,
            });
            return;
        }

        // PRIVMSG
        if let Some((target, text)) = parse_privmsg(rest) {
            let is_channel = target.starts_with('#');
            let buffer = if is_channel {
                target.to_string()
            } else {
                nick.clone()
            };

            let _ = ui_tx.send(UiEvent::EnsureBuffer {
                conn_id,
                buffer: buffer.clone(),
                make_current: false,
            });

            let is_action = is_ctcp_action(&text);
            let rendered = if is_action {
                format!("{} * {} {}", ts_prefix(), nick, parse_ctcp_action(&text))
            } else {
                format!("{} <{}> {}", ts_prefix(), nick, text)
            };

            let is_highlight = (!is_channel) || contains_nick_word(&text, my_nick_net);

            let _ = ui_tx.send(UiEvent::Append {
                conn_id,
                buffer,
                line: rendered,
                bump_unread: true,
                bump_highlight: is_highlight,
            });
            return;
        }

        // JOIN
        if let Some(chan) = parse_join(rest) {
            if chan.starts_with('#') {
                let _ = ui_tx.send(UiEvent::EnsureBuffer {
                    conn_id,
                    buffer: chan.to_string(),
                    make_current: false,
                });
                let _ = ui_tx.send(UiEvent::AddUser {
                    conn_id,
                    channel: chan.to_string(),
                    nick: nick.clone(),
                });
                let _ = ui_tx.send(UiEvent::Append {
                    conn_id,
                    buffer: chan.to_string(),
                    line: format!("{} *** {} joined {}", ts_prefix(), nick, chan),
                    bump_unread: true,
                    bump_highlight: false,
                });
            }
            return;
        }

        // PART
        if let Some(chan) = parse_part(rest) {
            if chan.starts_with('#') {
                let _ = ui_tx.send(UiEvent::RemoveUser {
                    conn_id,
                    channel: chan.to_string(),
                    nick: nick.clone(),
                });
                let _ = ui_tx.send(UiEvent::Append {
                    conn_id,
                    buffer: chan.to_string(),
                    line: format!("{} *** {} left {}", ts_prefix(), nick, chan),
                    bump_unread: true,
                    bump_highlight: false,
                });
            }
            return;
        }

        // QUIT
        if rest.starts_with("QUIT") {
            let _ = ui_tx.send(UiEvent::RemoveUserEverywhere { conn_id, nick: nick.clone() });
            let _ = ui_tx.send(UiEvent::Append {
                conn_id,
                buffer: "Status".to_string(),
                line: format!("{} *** {} quit", ts_prefix(), nick),
                bump_unread: true,
                bump_highlight: false,
            });
            return;
        }

        // KICK
        if let Some((chan, victim)) = parse_kick(rest) {
            if chan.starts_with('#') {
                let _ = ui_tx.send(UiEvent::RemoveUser {
                    conn_id,
                    channel: chan.to_string(),
                    nick: victim.to_string(),
                });
                let _ = ui_tx.send(UiEvent::Append {
                    conn_id,
                    buffer: chan.to_string(),
                    line: format!("{} *** {} kicked {}", ts_prefix(), nick, victim),
                    bump_unread: true,
                    bump_highlight: true,
                });
            }
            return;
        }
    }

    // Default: put unparsed lines into Status.
    let _ = ui_tx.send(UiEvent::Append {
        conn_id,
        buffer: "Status".to_string(),
        line: format!("{} {}", ts_prefix(), line),
        bump_unread: true,
        bump_highlight: false,
    });
}

fn parse_privmsg(rest: &str) -> Option<(&str, String)> {
    // PRIVMSG <target> :<text>
    let rest = rest.strip_prefix("PRIVMSG ")?;
    let (target, text) = rest.split_once(" :")?;
    Some((target.trim(), text.to_string()))
}

fn parse_join(rest: &str) -> Option<&str> {
    // JOIN :#chan  or JOIN #chan
    let rest = rest.strip_prefix("JOIN ")?;
    Some(rest.trim().trim_start_matches(':'))
}

fn parse_part(rest: &str) -> Option<&str> {
    // PART #chan ...
    let rest = rest.strip_prefix("PART ")?;
    Some(rest.split_whitespace().next().unwrap_or("Status"))
}

fn parse_kick(rest: &str) -> Option<(&str, &str)> {
    // KICK #chan victim :reason
    let rest = rest.strip_prefix("KICK ")?;
    let mut it = rest.split_whitespace();
    let chan = it.next()?;
    let victim = it.next()?;
    Some((chan, victim))
}

fn parse_nick_change(rest: &str) -> Option<String> {
    // NICK :newnick
    let rest = rest.strip_prefix("NICK ")?;
    Some(rest.trim().trim_start_matches(':').to_string())
}

fn parse_names_reply(line: &str) -> Option<(String, Vec<String>)> {
    // :server 353 <me> <symbol> <chan> :nick1 nick2 @nick3
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }
    if parts[0].starts_with(':') && parts[1] == "353" {
        let chan = parts[4].to_string();
        let mut users: Vec<String> = Vec::new();
        for (i, tok) in parts[5..].iter().enumerate() {
            let t = if i == 0 { tok.trim_start_matches(':') } else { tok };
            let tok = t.trim();
            if tok.is_empty() {
                continue;
            }
            // Keep the first status prefix if present, but remove additional garbage.
            // Many servers will send @nick or +nick.
            let user = tok.to_string();
            users.push(user);
        }
        return Some((chan, users));
    }
    None
}

fn parse_topic_332(line: &str) -> Option<(&str, String)> {
    // :server 332 <me> #chan :topic...
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    if parts[0].starts_with(':') && parts[1] == "332" {
        let chan = parts[3];
        let idx = line.find(" :")?;
        let topic = line[idx + 2..].to_string();
        return Some((chan, topic));
    }
    None
}

fn is_ctcp_action(text: &str) -> bool {
    text.starts_with("\u{0001}ACTION ") && text.ends_with('\u{0001}')
}

fn parse_ctcp_action(text: &str) -> String {
    let inner = text.trim_start_matches("\u{0001}ACTION ");
    inner.trim_end_matches('\u{0001}').to_string()
}

async fn send_line<W: tokio::io::AsyncWrite + Unpin>(w: &mut W, line: &str) -> Result<()> {
    // IRC lines must end with CRLF. We send UTF-8 bytes.
    let mut out = String::with_capacity(line.len() + 2);
    out.push_str(line);
    out.push_str("\r\n");
    w.write_all(out.as_bytes()).await?;
    w.flush().await?;
    Ok(())
}
