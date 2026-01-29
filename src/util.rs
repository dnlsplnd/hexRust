use time::format_description;
use time::OffsetDateTime;

pub fn ts_prefix() -> String {
    // Prefer local time, fall back to UTC if local offset is unavailable.
    let fmt = format_description::parse("[hour]:[minute]:[second]").unwrap();
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let s = now.format(&fmt).unwrap_or_else(|_| "??:??:??".to_string());
    format!("[{s}]")
}

pub fn clean_nick(s: &str) -> &str {
    // Strip common IRC status prefixes.
    s.trim_start_matches(|c: char| matches!(c, '@' | '+' | '~' | '&' | '%'))
}

pub fn contains_nick_word(haystack: &str, nick: &str) -> bool {
    // Case-insensitive word matching without regex.
    // Splits on non-alphanumeric characters and compares tokens.
    let nick = nick.to_lowercase();
    for token in haystack
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        .filter(|t| !t.is_empty())
    {
        if token.to_lowercase() == nick {
            return true;
        }
    }
    false
}


pub fn nick_rank(nick_with_prefix: &str) -> u8 {
    // Sort order: ~ & @ % + then normal (descending "rank" means higher first).
    match nick_with_prefix.chars().next().unwrap_or('\0') {
        '~' => 5,
        '&' => 4,
        '@' => 3,
        '%' => 2,
        '+' => 1,
        _ => 0,
    }
}

pub fn nick_display(nick_with_prefix: &str) -> &str {
    // Remove one leading status prefix if present.
    if let Some(c) = nick_with_prefix.chars().next() {
        if matches!(c, '~' | '&' | '@' | '%' | '+') {
            return &nick_with_prefix[c.len_utf8()..];
        }
    }
    nick_with_prefix
}
