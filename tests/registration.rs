//! Registration edge cases and message routing, against a scripted server.

use hexrust::model::{IrcConfig, UiEvent};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// One-shot IRC server. `reply` maps each received line to lines to send back;
/// everything the client sent is recorded.
async fn mock_server(
    reply: impl Fn(&str) -> Vec<String> + Send + 'static,
) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let log2 = Arc::clone(&log);

    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 1];
        let mut cur = Vec::<u8>::new();
        loop {
            match sock.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            if buf[0] != b'\n' {
                if buf[0] != b'\r' {
                    cur.push(buf[0]);
                }
                continue;
            }
            let line = String::from_utf8_lossy(&cur).to_string();
            cur.clear();
            if line.is_empty() {
                continue;
            }
            log2.lock().unwrap().push(line.clone());
            for out in reply(&line) {
                if sock.write_all(format!("{out}\r\n").as_bytes()).await.is_err() {
                    return;
                }
            }
        }
    });

    (port, log)
}

fn cfg(port: u16, nick: &str) -> IrcConfig {
    IrcConfig {
        server: "127.0.0.1".into(),
        port,
        tls: false,
        nick: nick.into(),
        initial_channel: "#c".into(),
        server_password: None,
        sasl_username: None,
        sasl_password: None,
    }
}

async fn drive(cfg: IrcConfig, secs: u64) -> Vec<String> {
    let (ui_tx, ui_rx) = std::sync::mpsc::channel::<UiEvent>();
    let (raw_tx, raw_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let run = tokio::spawn(hexrust::irc::irc_run(1, cfg, ui_tx, raw_rx, raw_tx));

    let lines = tokio::task::spawn_blocking(move || {
        let mut lines = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(secs);
        while let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) {
            match ui_rx.recv_timeout(left) {
                Ok(UiEvent::Append { line, .. }) => lines.push(line),
                Ok(_) => {}
                Err(_) => break,
            }
        }
        lines
    })
    .await
    .unwrap();

    run.abort();
    lines
}

/// A taken nick must not stall registration: without recovery the server never
/// sends 001 and the client waits forever.
#[tokio::test(flavor = "multi_thread")]
async fn nick_in_use_retries_with_a_variant_and_registers() {
    let (port, log) = mock_server(|line| {
        if line == "NICK taken" {
            // First choice is rejected.
            return vec![":srv 433 * taken :Nickname is already in use".into()];
        }
        if line == "NICK taken_" {
            // The retry is accepted; registration proceeds.
            return vec![":srv 001 taken_ :Welcome".into()];
        }
        vec![]
    })
    .await;

    let lines = drive(cfg(port, "taken"), 5).await;
    let sent = log.lock().unwrap().clone();

    assert!(
        sent.iter().any(|l| l == "NICK taken_"),
        "never retried with a new nick after 433; sent: {sent:?}"
    );
    assert!(
        lines.iter().any(|l| l.contains("Joining")),
        "never registered after the nick collision: {lines:?}"
    );
}

/// The client must give up rather than fight the server forever.
#[tokio::test(flavor = "multi_thread")]
async fn nick_in_use_stops_retrying_after_a_bounded_number_of_attempts() {
    let (port, log) = mock_server(|line| {
        if line.starts_with("NICK ") {
            return vec![":srv 433 * x :Nickname is already in use".into()];
        }
        vec![]
    })
    .await;

    let lines = drive(cfg(port, "taken"), 4).await;
    let sent = log.lock().unwrap().clone();

    let nick_lines = sent.iter().filter(|l| l.starts_with("NICK ")).count();
    assert!(
        nick_lines <= 5,
        "kept retrying nicks unbounded ({nick_lines} NICK lines): {sent:?}"
    );
    assert!(
        lines.iter().any(|l| l.contains("still in use")),
        "gave up without telling the user: {lines:?}"
    );
}

/// Notices must be visible and distinguishable from channel chat.
#[tokio::test(flavor = "multi_thread")]
async fn notices_are_rendered_distinctly_from_privmsg() {
    let (port, _log) = mock_server(|line| {
        if line.starts_with("USER ") {
            return vec![
                ":srv 001 n :Welcome".into(),
                ":alice!u@h NOTICE n :ping from alice".into(),
                ":alice!u@h PRIVMSG n :chat from alice".into(),
            ];
        }
        vec![]
    })
    .await;

    let lines = drive(cfg(port, "n"), 4).await;

    assert!(
        lines.iter().any(|l| l.contains("-alice- ping from alice")),
        "notice not rendered in -nick- form: {lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.contains("<alice> chat from alice")),
        "privmsg rendering changed: {lines:?}"
    );
}

/// Numerics should read as prose, not as raw protocol.
#[tokio::test(flavor = "multi_thread")]
async fn numeric_replies_have_their_preamble_stripped() {
    let (port, _log) = mock_server(|line| {
        if line.starts_with("USER ") {
            return vec![
                ":srv 001 n :Welcome".into(),
                ":srv 251 n :There are 64 users".into(),
            ];
        }
        vec![]
    })
    .await;

    let lines = drive(cfg(port, "n"), 4).await;

    assert!(
        lines.iter().any(|l| l.ends_with("There are 64 users")),
        "preamble not stripped: {lines:?}"
    );
    assert!(
        !lines.iter().any(|l| l.contains(":srv 251")),
        "raw numeric still shown: {lines:?}"
    );
}

/// A topic change by another user should be reported in the channel.
#[tokio::test(flavor = "multi_thread")]
async fn topic_changes_are_attributed() {
    let (port, _log) = mock_server(|line| {
        if line.starts_with("USER ") {
            return vec![
                ":srv 001 n :Welcome".into(),
                ":alice!u@h TOPIC #c :new topic here".into(),
            ];
        }
        vec![]
    })
    .await;

    let lines = drive(cfg(port, "n"), 4).await;

    assert!(
        lines.iter().any(|l| l.contains("alice") && l.contains("new topic here")),
        "topic change not attributed: {lines:?}"
    );
}
