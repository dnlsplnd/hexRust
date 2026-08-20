//! Inline formatting-code removal.

use hexrust::format::strip_formatting;

#[test]
fn plain_text_is_untouched() {
    assert_eq!(strip_formatting("hello world"), "hello world");
    assert_eq!(strip_formatting(""), "");
}

#[test]
fn simple_toggles_are_removed() {
    assert_eq!(strip_formatting("\u{2}bold\u{2}"), "bold");
    assert_eq!(strip_formatting("\u{1D}italic\u{1D}"), "italic");
    assert_eq!(strip_formatting("\u{1F}under\u{1F}"), "under");
    assert_eq!(strip_formatting("\u{1E}strike\u{1E}"), "strike");
    assert_eq!(strip_formatting("\u{11}mono\u{11}"), "mono");
    assert_eq!(strip_formatting("\u{16}rev\u{16}"), "rev");
    assert_eq!(strip_formatting("a\u{F}b"), "ab");
}

#[test]
fn colour_codes_are_removed_in_all_their_forms() {
    assert_eq!(strip_formatting("\u{3}4red"), "red");
    assert_eq!(strip_formatting("\u{3}04red"), "red");
    assert_eq!(strip_formatting("\u{3}4,2both"), "both");
    assert_eq!(strip_formatting("\u{3}04,02both"), "both");
    // A bare colour code resets and carries no digits.
    assert_eq!(strip_formatting("a\u{3}b"), "ab");
}

/// A comma after a colour is only part of the code when digits follow, or
/// ordinary punctuation gets eaten.
#[test]
fn a_comma_that_is_not_a_background_is_kept() {
    assert_eq!(strip_formatting("\u{3}4hi, there"), "hi, there");
    assert_eq!(strip_formatting("\u{3}4,x"), ",x");
}

/// Only two digits belong to a colour; a third is message text.
#[test]
fn digits_beyond_the_code_are_kept() {
    assert_eq!(strip_formatting("\u{3}123"), "3");
    assert_eq!(strip_formatting("\u{3}04,021"), "1");
}

#[test]
fn hex_colour_codes_are_removed() {
    assert_eq!(strip_formatting("\u{4}FF0000red"), "red");
    assert_eq!(strip_formatting("\u{4}FF0000,00FF00both"), "both");
}

#[test]
fn multibyte_text_survives() {
    assert_eq!(strip_formatting("\u{3}4héllo wörld\u{F}"), "héllo wörld");
    assert_eq!(strip_formatting("\u{2}日本語\u{2}"), "日本語");
}

/// The realistic case: several codes interleaved through one line.
#[test]
fn a_heavily_formatted_line_reduces_to_its_text() {
    let input = "\u{3}04,00\u{2}WARN\u{2}\u{3} \u{1F}disk\u{1F} at \u{3}0899%\u{3}";
    assert_eq!(strip_formatting(input), "WARN disk at 99%");
}

/// Stripping must not leave stray control characters behind.
#[test]
fn no_formatting_control_characters_remain() {
    let input = "\u{2}a\u{3}4b\u{4}FF0000c\u{F}d\u{16}e\u{11}f\u{1D}g\u{1E}h\u{1F}i";
    let out = strip_formatting(input);
    assert_eq!(out, "abcdefghi");
    for c in out.chars() {
        assert!(!c.is_control(), "control char {:?} survived", c);
    }
}
