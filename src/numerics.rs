//! Rendering of numeric replies into readable lines.
//!
//! Numerics arrive as positional parameters with the human-readable part in a
//! trailing `:` argument. Printed as-is they read as protocol rather than
//! prose, and the WHOIS family is the worst of it: the fields are unlabelled,
//! some read backwards, and the idle reply is two bare integers.

/// Splits the parameters of an IRC message, honouring the trailing argument.
///
/// Everything after a leading `:` is one parameter, spaces included, which is
/// what allows real names, topics and messages to contain spaces at all.
fn split_params(mut rest: &str) -> Vec<String> {
    let mut out = Vec::new();
    loop {
        rest = rest.trim_start_matches(' ');
        if rest.is_empty() {
            break;
        }
        if let Some(trailing) = rest.strip_prefix(':') {
            out.push(trailing.to_string());
            break;
        }
        match rest.find(' ') {
            Some(i) => {
                out.push(rest[..i].to_string());
                rest = &rest[i..];
            }
            None => {
                out.push(rest.to_string());
                break;
            }
        }
    }
    out
}

/// Splits a numeric line into its code and parameters, dropping the server
/// prefix and the leading parameter that names us.
fn parse(line: &str, my_nick: &str) -> Option<(String, Vec<String>)> {
    let rest = line.strip_prefix(':')?;
    let (_server, rest) = rest.split_once(' ')?;
    let (code, rest) = rest.split_once(' ')?;
    if code.len() != 3 || !code.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut params = split_params(rest);
    // Numerics addressed to us open with our own nick, which carries no
    // information for the reader.
    if !params.is_empty() && (params[0] == my_nick || params[0] == "*") {
        params.remove(0);
    }
    Some((code.to_string(), params))
}

/// Formats a duration in seconds as a compact human span.
fn human_duration(secs: u64) -> String {
    if secs == 0 {
        return "0s".to_string();
    }
    let d = secs / 86_400;
    let h = (secs % 86_400) / 3_600;
    let m = (secs % 3_600) / 60;
    let s = secs % 60;
    let mut parts = Vec::new();
    if d > 0 {
        parts.push(format!("{d}d"));
    }
    if h > 0 {
        parts.push(format!("{h}h"));
    }
    if m > 0 {
        parts.push(format!("{m}m"));
    }
    if s > 0 && d == 0 {
        parts.push(format!("{s}s"));
    }
    parts.join(" ")
}

/// Formats a unix timestamp as a local date and time.
fn human_time(unix: i64) -> String {
    use time::format_description;
    let Ok(t) = time::OffsetDateTime::from_unix_timestamp(unix) else {
        return unix.to_string();
    };
    let t = t
        .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC));
    let Ok(fmt) = format_description::parse_borrowed::<2>("[year]-[month]-[day] [hour]:[minute]") else {
        return unix.to_string();
    };
    t.format(&fmt).unwrap_or_else(|_| unix.to_string())
}

/// A readable rendering of a numeric, or None if this module has no opinion
/// about that code and the caller should fall back to `strip_preamble`.
pub fn render(line: &str, my_nick: &str) -> Option<String> {
    let (code, p) = parse(line, my_nick)?;
    let arg = |i: usize| p.get(i).map(String::as_str).unwrap_or("");
    let nick = arg(0);
    if nick.is_empty() {
        return None;
    }

    let body = match code.as_str() {
        // WHOIS/WHOWAS user: <nick> <user> <host> * :<real name>
        "311" | "314" => {
            let real = p.last().map(String::as_str).unwrap_or("");
            format!("{nick} is {}@{} ({real})", arg(1), arg(2))
        }

        // Server the user is on: <nick> <server> :<info>
        "312" => format!("{nick} using {} ({})", arg(1), arg(2)),

        // Channels: <nick> :<chan list>
        "319" => format!("{nick} on {}", arg(1)),

        // Idle: <nick> <seconds> <signon> :seconds idle, signon time
        "317" => {
            let idle = arg(1).parse::<u64>().ok();
            let signon = arg(2).parse::<i64>().ok();
            match (idle, signon) {
                (Some(i), Some(s)) => format!(
                    "{nick} idle {}, signed on {}",
                    human_duration(i),
                    human_time(s)
                ),
                (Some(i), None) => format!("{nick} idle {}", human_duration(i)),
                _ => return None,
            }
        }

        // Account: <nick> <account> :is logged in as -- the trailing text
        // comes after the value, so printed in order it reads backwards.
        "330" => format!("{nick} {} {}", arg(2), arg(1)),

        // Away: <nick> :<message>
        "301" => format!("{nick} is away: {}", arg(1)),

        // Actual host: <nick> <host> :<text>
        "338" => format!("{nick} {} {}", arg(2), arg(1)),

        // End of list.
        "318" => format!("End of WHOIS for {nick}"),
        "369" => format!("End of WHOWAS for {nick}"),

        // The rest of the family is <nick> :<text>, which reads correctly as
        // "nick is an IRC operator", "nick is using a secure connection", and
        // so on.
        "276" | "307" | "313" | "320" | "335" | "378" | "379" | "671" => {
            format!("{nick} {}", arg(1))
        }

        _ => return None,
    };

    Some(format!("*** {body}"))
}

/// Strips the `:server 251 mynick` preamble, leaving the readable remainder.
///
/// The generic fallback for numerics `render` has no special handling for.
pub fn strip_preamble(line: &str, my_nick: &str) -> Option<String> {
    let rest = line.strip_prefix(':')?;
    let (_server, rest) = rest.split_once(' ')?;
    let (code, rest) = rest.split_once(' ')?;
    if code.len() != 3 || !code.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let rest = match rest.split_once(' ') {
        Some((target, tail)) if target == my_nick || target == "*" => tail,
        _ => rest,
    };
    Some(rest.strip_prefix(':').unwrap_or(rest).to_string())
}
