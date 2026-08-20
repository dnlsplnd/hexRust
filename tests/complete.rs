//! Tab completion. Pure, so no GTK display is involved.

use hexrust::complete::TabCycle;

fn nicks() -> Vec<String> {
    ["@alice", "albert", "bob", "Bobby", "carol", "+dave"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Completes at the end of `text`, as if the cursor were there.
fn tab(c: &mut TabCycle, text: &str) -> Option<String> {
    c.advance(text, text.chars().count(), &nicks()).map(|r| r.text)
}

#[test]
fn a_unique_nick_at_line_start_gets_a_colon() {
    let mut c = TabCycle::default();
    assert_eq!(tab(&mut c, "car").unwrap(), "carol: ");
}

/// Mid-sentence the colon form would be wrong, so a plain space is used.
#[test]
fn a_nick_later_in_the_line_gets_a_space() {
    let mut c = TabCycle::default();
    assert_eq!(tab(&mut c, "hey car").unwrap(), "hey carol ");
}

/// The status prefix is display decoration, not part of the nick.
#[test]
fn status_prefixes_are_stripped_from_the_completion() {
    let mut c = TabCycle::default();
    assert_eq!(tab(&mut c, "ali").unwrap(), "alice: ");
    let mut c = TabCycle::default();
    assert_eq!(tab(&mut c, "hey dav").unwrap(), "hey dave ");
}

#[test]
fn matching_ignores_case_but_the_canonical_nick_is_inserted() {
    let mut c = TabCycle::default();
    assert_eq!(tab(&mut c, "bobb").unwrap(), "Bobby: ");
}

/// Repeated presses walk the candidates and wrap around.
#[test]
fn repeated_tabs_cycle_through_matches() {
    let mut c = TabCycle::default();
    let first = tab(&mut c, "b").unwrap();
    let second = tab(&mut c, &first).unwrap();
    let third = tab(&mut c, &second).unwrap();

    assert_ne!(first, second, "second Tab did not advance");
    assert_eq!(first, third, "cycle did not wrap back around");
    for t in [&first, &second] {
        assert!(t.starts_with("bob") || t.starts_with("Bobby"), "unexpected: {t}");
    }
}

/// Cycling must replace the previous completion, not append to it.
#[test]
fn cycling_does_not_accumulate_text() {
    let mut c = TabCycle::default();
    let first = tab(&mut c, "hey b").unwrap();
    let second = tab(&mut c, &first).unwrap();
    assert_eq!(second.matches(' ').count(), first.matches(' ').count());
    assert!(!second.contains("bob") || !second.contains("Bobby"), "both inserted: {second}");
}

/// Typing after a completion starts a new one rather than cycling the old.
#[test]
fn editing_the_entry_breaks_the_cycle() {
    let mut c = TabCycle::default();
    let first = tab(&mut c, "b").unwrap();
    assert!(first.starts_with("bob") || first.starts_with("Bobby"));
    // The user types something else entirely.
    assert_eq!(tab(&mut c, "car").unwrap(), "carol: ");
}

#[test]
fn commands_complete_only_at_the_start_of_the_line() {
    let mut c = TabCycle::default();
    assert_eq!(tab(&mut c, "/who").unwrap(), "/whois ");

    // Not a command here, and no nick matches, so nothing happens.
    let mut c = TabCycle::default();
    assert!(tab(&mut c, "see /who").is_none());
}

#[test]
fn a_completed_command_never_gets_the_colon_form() {
    let mut c = TabCycle::default();
    let out = tab(&mut c, "/j").unwrap();
    assert_eq!(out, "/join ");
    assert!(!out.contains(':'));
}

#[test]
fn ambiguous_commands_cycle_too() {
    let mut c = TabCycle::default();
    let a = tab(&mut c, "/w").unwrap();
    let b = tab(&mut c, &a).unwrap();
    assert_ne!(a, b);
    for t in [&a, &b] {
        assert!(t.starts_with("/who"), "unexpected command completion: {t}");
    }
}

#[test]
fn no_match_leaves_the_entry_alone() {
    let mut c = TabCycle::default();
    assert!(tab(&mut c, "zzz").is_none());
    assert!(tab(&mut c, "/zzz").is_none());
}

#[test]
fn tab_on_empty_or_trailing_space_does_nothing() {
    let mut c = TabCycle::default();
    assert!(tab(&mut c, "").is_none());
    assert!(tab(&mut c, "hello ").is_none());
}

/// Completion happens at the cursor, leaving the rest of the line intact.
#[test]
fn text_after_the_cursor_is_preserved() {
    let mut c = TabCycle::default();
    let text = "car and more";
    let r = c.advance(text, 3, &nicks()).unwrap();
    assert_eq!(r.text, "carol:  and more");
    assert_eq!(r.cursor, "carol: ".chars().count());
}

/// Cursor arithmetic is in characters, so multi-byte text must not corrupt.
#[test]
fn multibyte_text_before_the_word_is_handled() {
    let mut c = TabCycle::default();
    let text = "héllo car";
    let r = c.advance(text, text.chars().count(), &nicks()).unwrap();
    assert_eq!(r.text, "héllo carol ");
    assert_eq!(r.cursor, r.text.chars().count());
}

#[test]
fn match_count_is_reported() {
    let mut c = TabCycle::default();
    assert_eq!(c.advance("car", 3, &nicks()).unwrap().match_count, 1);
    let mut c = TabCycle::default();
    assert!(c.advance("b", 1, &nicks()).unwrap().match_count >= 2);
}
