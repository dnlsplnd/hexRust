//! Slash commands that only talk to the server.
//!
//! These are kept separate from the handlers in `ui.rs` on purpose: the
//! commands here produce wire lines and echo text and nothing else, so they
//! are pure functions that can be tested without a GTK display. Commands
//! that also mutate the UI (opening buffers, switching tabs) stay in `ui.rs`.

/// What the caller should do as a result of a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Out {
    /// Send this line to the server verbatim.
    Raw(String),
    /// Echo this text into the current buffer (caller adds the timestamp).
    Info(String),
}

/// Everything a command needs to know about where it was typed.
pub struct Ctx<'a> {
    /// Name of the active buffer: "Status", a channel, or a nick.
    pub current_buffer: &'a str,
    /// Channel to fall back to when the active buffer is not a channel.
    pub default_target: &'a str,
}

/// True for the channel prefixes in RFC 2811.
pub fn is_channel(s: &str) -> bool {
    matches!(s.chars().next(), Some('#') | Some('&') | Some('+') | Some('!'))
}

impl Ctx<'_> {
    /// The channel a command should act on when none is given.
    fn implied_channel(&self) -> Option<&str> {
        if is_channel(self.current_buffer) {
            Some(self.current_buffer)
        } else if is_channel(self.default_target) {
            Some(self.default_target)
        } else {
            None
        }
    }
}

fn usage(text: &str) -> Option<Vec<Out>> {
    Some(vec![Out::Info(format!("*** Usage: {text}"))])
}

/// Splits off the first whitespace-delimited word, returning (word, rest).
fn word(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    match s.find(char::is_whitespace) {
        Some(i) => (&s[..i], s[i..].trim_start()),
        None => (s, ""),
    }
}

/// Maps a slash command to wire lines.
///
/// `cmd` is the command with no leading slash, already lowercased. `args` is
/// the untouched remainder of the line. Returns `None` for commands this
/// module does not own, so the caller can handle them itself.
pub fn plan(cmd: &str, args: &str, ctx: &Ctx) -> Option<Vec<Out>> {
    let args = args.trim();

    match cmd {
        "part" | "leave" => {
            // /part            -> leave the current channel
            // /part #chan      -> leave that channel
            // /part reason...  -> leave the current channel with a reason
            let (first, rest) = word(args);
            let (chan, reason) = if is_channel(first) {
                (first.to_string(), rest.to_string())
            } else {
                (ctx.implied_channel()?.to_string(), args.to_string())
            };
            let line = if reason.is_empty() {
                format!("PART {chan}")
            } else {
                format!("PART {chan} :{reason}")
            };
            Some(vec![Out::Raw(line)])
        }

        "quit" => {
            let line = if args.is_empty() {
                "QUIT :hexrust".to_string()
            } else {
                format!("QUIT :{args}")
            };
            Some(vec![Out::Raw(line)])
        }

        "topic" => {
            // /topic                 -> ask for the current channel's topic
            // /topic text...         -> set it on the current channel
            // /topic #chan text...   -> set it on that channel
            let (first, rest) = word(args);
            let (chan, text) = if is_channel(first) {
                (first.to_string(), rest.to_string())
            } else {
                (ctx.implied_channel()?.to_string(), args.to_string())
            };
            let line = if text.is_empty() {
                format!("TOPIC {chan}")
            } else {
                format!("TOPIC {chan} :{text}")
            };
            Some(vec![Out::Raw(line)])
        }

        "whois" => {
            if args.is_empty() {
                return usage("/whois <nick>");
            }
            Some(vec![Out::Raw(format!("WHOIS {args}"))])
        }

        "whowas" => {
            if args.is_empty() {
                return usage("/whowas <nick>");
            }
            Some(vec![Out::Raw(format!("WHOWAS {args}"))])
        }

        "kick" => {
            // /kick nick [reason]        -> in the current channel
            // /kick #chan nick [reason]
            let (first, rest) = word(args);
            let (chan, victim, reason) = if is_channel(first) {
                let (v, r) = word(rest);
                (first.to_string(), v.to_string(), r.to_string())
            } else {
                (ctx.implied_channel()?.to_string(), first.to_string(), rest.to_string())
            };
            if victim.is_empty() {
                return usage("/kick [#chan] <nick> [reason]");
            }
            let line = if reason.is_empty() {
                format!("KICK {chan} {victim}")
            } else {
                format!("KICK {chan} {victim} :{reason}")
            };
            Some(vec![Out::Raw(line)])
        }

        "mode" => {
            if args.is_empty() {
                // Bare /mode in a channel reports that channel's modes.
                let chan = ctx.implied_channel()?;
                return Some(vec![Out::Raw(format!("MODE {chan}"))]);
            }
            // A leading +/- means the target was omitted.
            let (first, _) = word(args);
            if first.starts_with('+') || first.starts_with('-') {
                let chan = ctx.implied_channel()?;
                return Some(vec![Out::Raw(format!("MODE {chan} {args}"))]);
            }
            Some(vec![Out::Raw(format!("MODE {args}"))])
        }

        // Convenience wrappers over MODE, applied to the current channel.
        "op" | "deop" | "voice" | "devoice" | "halfop" | "dehalfop" => {
            if args.is_empty() {
                return usage(&format!("/{cmd} <nick>"));
            }
            let chan = ctx.implied_channel()?;
            let flag = match cmd {
                "op" => "+o",
                "deop" => "-o",
                "voice" => "+v",
                "devoice" => "-v",
                "halfop" => "+h",
                _ => "-h",
            };
            // Several nicks may be given at once: /op alice bob
            let nicks: Vec<&str> = args.split_whitespace().collect();
            let signs = format!(
                "{}{}",
                &flag[..1],
                flag[1..].repeat(nicks.len())
            );
            Some(vec![Out::Raw(format!("MODE {chan} {signs} {}", nicks.join(" ")))])
        }

        "ban" | "unban" => {
            if args.is_empty() {
                return usage(&format!("/{cmd} <mask>"));
            }
            let chan = ctx.implied_channel()?;
            let sign = if cmd == "ban" { "+b" } else { "-b" };
            Some(vec![Out::Raw(format!("MODE {chan} {sign} {args}"))])
        }

        "notice" => {
            let (target, text) = word(args);
            if target.is_empty() || text.is_empty() {
                return usage("/notice <target> <text>");
            }
            Some(vec![
                Out::Raw(format!("NOTICE {target} :{text}")),
                Out::Info(format!("-> -{target}- {text}")),
            ])
        }

        "away" => {
            if args.is_empty() {
                Some(vec![Out::Raw("AWAY".to_string()), Out::Info("*** Away status cleared".to_string())])
            } else {
                Some(vec![
                    Out::Raw(format!("AWAY :{args}")),
                    Out::Info(format!("*** Now away: {args}")),
                ])
            }
        }

        "back" => Some(vec![
            Out::Raw("AWAY".to_string()),
            Out::Info("*** Away status cleared".to_string()),
        ]),

        "list" => {
            let line = if args.is_empty() { "LIST".to_string() } else { format!("LIST {args}") };
            Some(vec![Out::Raw(line)])
        }

        "names" => {
            let chan = if is_channel(args) { args.to_string() } else { ctx.implied_channel()?.to_string() };
            Some(vec![Out::Raw(format!("NAMES {chan}"))])
        }

        "invite" => {
            let (nick, rest) = word(args);
            if nick.is_empty() {
                return usage("/invite <nick> [#chan]");
            }
            let chan = if is_channel(rest) { rest.to_string() } else { ctx.implied_channel()?.to_string() };
            Some(vec![Out::Raw(format!("INVITE {nick} {chan}"))])
        }

        "ctcp" => {
            let (target, rest) = word(args);
            if target.is_empty() || rest.is_empty() {
                return usage("/ctcp <target> <command> [args]");
            }
            let payload = rest.to_uppercase();
            Some(vec![
                Out::Raw(format!("PRIVMSG {target} :\u{0001}{payload}\u{0001}")),
                Out::Info(format!("-> [{target}] CTCP {payload}")),
            ])
        }

        "ping" => {
            if args.is_empty() {
                return usage("/ping <nick>");
            }
            Some(vec![
                Out::Raw(format!("PRIVMSG {args} :\u{0001}PING\u{0001}")),
                Out::Info(format!("-> [{args}] CTCP PING")),
            ])
        }

        "version" => {
            if args.is_empty() {
                return Some(vec![Out::Raw("VERSION".to_string())]);
            }
            Some(vec![
                Out::Raw(format!("PRIVMSG {args} :\u{0001}VERSION\u{0001}")),
                Out::Info(format!("-> [{args}] CTCP VERSION")),
            ])
        }

        "motd" => Some(vec![Out::Raw("MOTD".to_string())]),

        "oper" => {
            let (user, pass) = word(args);
            if user.is_empty() || pass.is_empty() {
                return usage("/oper <user> <password>");
            }
            // Deliberately not echoed: it contains a password.
            Some(vec![Out::Raw(format!("OPER {user} {pass}"))])
        }

        "help" => Some(help_lines()),

        _ => None,
    }
}

fn help_lines() -> Vec<Out> {
    [
        "*** Commands:",
        "***   /join #chan            /part [#chan] [reason]",
        "***   /msg <nick> <text>     /query <nick>",
        "***   /me <action>           /notice <target> <text>",
        "***   /nick <newnick>        /away [reason]   /back",
        "***   /topic [#chan] [text]  /names [#chan]",
        "***   /whois <nick>          /whowas <nick>",
        "***   /kick [#chan] <nick> [reason]",
        "***   /ban <mask>            /unban <mask>",
        "***   /op /deop /voice /devoice <nick>",
        "***   /mode [target] <modes>  /invite <nick> [#chan]",
        "***   /ctcp <target> <cmd>   /ping <nick>   /version [nick]",
        "***   /list [params]         /motd          /oper <user> <pass>",
        "***   /switch <buffer>       /server <id> <raw>",
        "***   /raw <line>            /quit [reason]",
    ]
    .iter()
    .map(|s| Out::Info((*s).to_string()))
    .collect()
}

/// True for commands this module owns that can only work inside a channel.
///
/// `plan` returns `None` both for commands it does not own and for these when
/// no channel can be resolved. The caller needs to tell those apart: the first
/// is passed through to the server, the second is a user error worth
/// reporting instead of sending a malformed line.
pub fn needs_channel(cmd: &str) -> bool {
    matches!(
        cmd,
        "mode"
            | "part"
            | "leave"
            | "topic"
            | "kick"
            | "names"
            | "invite"
            | "op"
            | "deop"
            | "voice"
            | "devoice"
            | "halfop"
            | "dehalfop"
            | "ban"
            | "unban"
    )
}
