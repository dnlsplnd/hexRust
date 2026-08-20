//! Rendering of numeric replies, WHOIS in particular.

use hexrust::numerics::{render, strip_preamble};

fn r(line: &str) -> String {
    render(line, "me").unwrap_or_else(|| panic!("not rendered: {line}"))
}

#[test]
fn whois_user_reads_as_a_sentence() {
    assert_eq!(
        r(":srv 311 me alice ~user cloak/alice * :Alice Example"),
        "*** alice is ~user@cloak/alice (Alice Example)"
    );
}

/// Real names contain spaces, so the trailing parameter must not be split.
#[test]
fn a_real_name_with_spaces_survives() {
    assert_eq!(
        r(":srv 311 me alice ~u host * :Alice Q. Example"),
        "*** alice is ~u@host (Alice Q. Example)"
    );
}

#[test]
fn channels_and_server_are_labelled() {
    assert_eq!(
        r(":srv 319 me alice :#rust #linux @#staff"),
        "*** alice on #rust #linux @#staff"
    );
    assert_eq!(
        r(":srv 312 me alice copper.libera.chat :Umea, SE"),
        "*** alice using copper.libera.chat (Umea, SE)"
    );
}

/// The account reply puts the value before the text, so printed in order it
/// reads "alice aliceacct is logged in as".
#[test]
fn the_account_reply_is_reordered_to_read_forwards() {
    assert_eq!(
        r(":srv 330 me alice aliceacct :is logged in as"),
        "*** alice is logged in as aliceacct"
    );
}

/// Idle arrives as two bare integers, which mean nothing to a reader.
#[test]
fn idle_time_becomes_a_span_and_a_date() {
    let out = r(":srv 317 me alice 1234 1787000000 :seconds idle, signon time");
    assert!(out.starts_with("*** alice idle 20m 34s, signed on "), "got {out}");
    assert!(!out.contains("1234"), "raw seconds left in: {out}");
    assert!(!out.contains("1787000000"), "raw timestamp left in: {out}");
}

#[test]
fn long_idle_times_read_in_days_and_hours() {
    let out = r(":srv 317 me alice 90061 1787000000 :seconds idle, signon time");
    assert!(out.contains("idle 1d 1h 1m"), "got {out}");
}

#[test]
fn a_zero_idle_time_is_still_sensible() {
    let out = r(":srv 317 me alice 0 1787000000 :seconds idle, signon time");
    assert!(out.contains("idle 0s"), "got {out}");
}

#[test]
fn the_simple_flag_replies_read_correctly_as_is() {
    assert_eq!(
        r(":srv 671 me alice :is using a secure connection"),
        "*** alice is using a secure connection"
    );
    assert_eq!(
        r(":srv 313 me alice :is an IRC operator"),
        "*** alice is an IRC operator"
    );
}

#[test]
fn away_is_shown_with_its_message() {
    assert_eq!(r(":srv 301 me alice :lunch"), "*** alice is away: lunch");
}

#[test]
fn end_of_list_names_the_subject() {
    assert_eq!(r(":srv 318 me alice :End of /WHOIS list."), "*** End of WHOIS for alice");
    assert_eq!(r(":srv 369 me alice :End of /WHOWAS list."), "*** End of WHOWAS for alice");
}

#[test]
fn whowas_uses_the_same_shape_as_whois() {
    assert_eq!(
        r(":srv 314 me alice ~u host * :Alice"),
        "*** alice is ~u@host (Alice)"
    );
}

/// Nothing in a rendered line should still look like raw protocol.
#[test]
fn no_stray_parameter_separators_remain() {
    for line in [
        ":srv 311 me alice ~u host * :Alice",
        ":srv 319 me alice :#a #b",
        ":srv 312 me alice s.example :Info",
        ":srv 330 me alice acct :is logged in as",
        ":srv 301 me alice :away msg",
        ":srv 318 me alice :End of /WHOIS list.",
    ] {
        let out = r(line);
        assert!(!out.contains(" :"), "parameter separator left in: {out}");
        assert!(!out.contains(" * "), "unused placeholder left in: {out}");
    }
}

/// Codes this module has no opinion about must fall through, not be mangled.
#[test]
fn unknown_numerics_are_left_to_the_generic_path() {
    assert!(render(":srv 251 me :There are 64 users", "me").is_none());
    assert!(render(":srv 001 me :Welcome", "me").is_none());
}

#[test]
fn non_numeric_lines_are_not_touched() {
    assert!(render(":alice!u@h PRIVMSG #c :hi", "me").is_none());
    assert!(render("PING :srv", "me").is_none());
    assert!(strip_preamble("PING :srv", "me").is_none());
}

#[test]
fn the_generic_path_still_strips_the_preamble() {
    assert_eq!(
        strip_preamble(":srv 251 me :There are 64 users", "me").unwrap(),
        "There are 64 users"
    );
}

/// A truncated numeric must not panic or produce half a sentence.
#[test]
fn malformed_numerics_are_handled() {
    assert!(render(":srv 311 me", "me").is_none());
    assert!(render(":srv 317 me alice notanumber x :idle", "me").is_none());
    // Missing fields degrade rather than panicking.
    let out = render(":srv 311 me alice", "me").unwrap();
    assert!(out.contains("alice"), "got {out}");
}
