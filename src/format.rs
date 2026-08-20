//! Removal of IRC inline formatting codes.
//!
//! Messages carry formatting in-band as control characters. The buffer view is
//! plain text, so the codes would otherwise show up as stray control
//! characters or invisible junk that breaks alignment and search.

/// Strips mIRC/IRCv3 formatting codes, leaving the readable text.
///
/// Handles bold, italic, underline, strikethrough, monospace, reverse and
/// reset, plus `\x03` colour (`\x03`, `\x03N`, `\x03N,M`) and `\x04` hex
/// colour (`\x04RRGGBB[,RRGGBB]`).
pub fn strip_formatting(s: &str) -> String {
    let cs: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;

    while i < cs.len() {
        match cs[i] {
            // Toggles with no payload.
            '\u{0002}' | '\u{001D}' | '\u{001F}' | '\u{001E}' | '\u{0011}' | '\u{0016}'
            | '\u{000F}' => {
                i += 1;
            }

            // Colour: up to two digits, optionally a comma and two more.
            '\u{0003}' => {
                i += 1;
                let mut digits = 0;
                while i < cs.len() && digits < 2 && cs[i].is_ascii_digit() {
                    i += 1;
                    digits += 1;
                }
                // A comma only belongs to the code when digits follow it,
                // otherwise it is ordinary punctuation in the message.
                if digits > 0
                    && i + 1 < cs.len()
                    && cs[i] == ','
                    && cs[i + 1].is_ascii_digit()
                {
                    i += 1;
                    let mut bg = 0;
                    while i < cs.len() && bg < 2 && cs[i].is_ascii_digit() {
                        i += 1;
                        bg += 1;
                    }
                }
            }

            // Hex colour: six hex digits, optionally a comma and six more.
            '\u{0004}' => {
                i += 1;
                let mut take_hex = |i: &mut usize| {
                    let mut n = 0;
                    while *i < cs.len() && n < 6 && cs[*i].is_ascii_hexdigit() {
                        *i += 1;
                        n += 1;
                    }
                    n
                };
                let n = take_hex(&mut i);
                if n > 0 && i + 1 < cs.len() && cs[i] == ',' && cs[i + 1].is_ascii_hexdigit() {
                    i += 1;
                    take_hex(&mut i);
                }
            }

            c => {
                out.push(c);
                i += 1;
            }
        }
    }

    out
}
