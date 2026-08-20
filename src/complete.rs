//! Tab completion for the message entry.
//!
//! Pure: the caller passes the entry text, the cursor, and the candidate
//! nicks, and gets back the replacement text and cursor. Repeated presses
//! cycle through the matches, which is why the state lives in `TabCycle`
//! rather than being recomputed from the entry alone.

use crate::commands::COMMANDS;

/// Result of one completion step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// Replacement contents for the entry.
    pub text: String,
    /// Where to put the cursor, in characters.
    pub cursor: usize,
    /// How many candidates matched, so the caller can report ambiguity.
    pub match_count: usize,
}

/// Completion state carried between Tab presses.
#[derive(Debug, Default)]
pub struct TabCycle {
    /// The word originally typed, before any completion replaced it.
    prefix: String,
    /// Character index where that word starts.
    start: usize,
    /// Character index just past the text we last inserted.
    end: usize,
    /// Which match was used last.
    index: usize,
    /// The entry contents we last produced, used to detect that the user has
    /// not typed anything since, so Tab means "cycle" rather than "start".
    last: String,
}

/// Splits a string into characters once, so all indexing is by character.
/// Entry cursor positions in GTK are character offsets, not bytes.
fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

fn slice(cs: &[char], a: usize, b: usize) -> String {
    cs[a.min(cs.len())..b.min(cs.len())].iter().collect()
}

/// Start of the whitespace-delimited word ending at `cursor`.
fn word_start(cs: &[char], cursor: usize) -> usize {
    let mut i = cursor.min(cs.len());
    while i > 0 && !cs[i - 1].is_whitespace() {
        i -= 1;
    }
    i
}

/// Candidates matching `prefix`, case-insensitively, in a stable order.
fn matches_for(prefix: &str, nicks: &[String], is_command: bool) -> Vec<String> {
    let needle = prefix.to_lowercase();
    if is_command {
        let needle = needle.trim_start_matches('/');
        let mut v: Vec<String> = COMMANDS
            .iter()
            .filter(|c| c.starts_with(needle))
            .map(|c| format!("/{c}"))
            .collect();
        v.sort();
        return v;
    }
    // Nicks may carry a status prefix (@alice); complete the bare nick.
    let mut v: Vec<String> = nicks
        .iter()
        .map(|n| crate::util::nick_display(n).to_string())
        .filter(|n| n.to_lowercase().starts_with(&needle))
        .collect();
    v.sort_by_key(|a| a.to_lowercase());
    v.dedup();
    v
}

impl TabCycle {
    /// Forget any in-progress cycle. Call when the entry changes for a reason
    /// other than completion, or the next Tab would cycle a stale word.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Advance one completion step, returning the new entry contents.
    ///
    /// `nicks` are the candidates for the active channel; they may carry
    /// status prefixes. Returns `None` when there is nothing to complete.
    pub fn advance(&mut self, text: &str, cursor: usize, nicks: &[String]) -> Option<Completion> {
        let cs = chars(text);
        let cursor = cursor.min(cs.len());

        // Continuing a cycle only if the entry is untouched since our last
        // insertion; otherwise the user has typed and we start over.
        let continuing = !self.prefix.is_empty() && self.last == text;

        let (start, prefix, replace_end, index) = if continuing {
            (self.start, self.prefix.clone(), self.end, self.index + 1)
        } else {
            let start = word_start(&cs, cursor);
            let prefix = slice(&cs, start, cursor);
            if prefix.is_empty() {
                self.reset();
                return None;
            }
            (start, prefix, cursor, 0)
        };

        // A leading slash means a command, but only at the very start of the
        // line: "/j" completes, "see /j" does not.
        let is_command = start == 0 && prefix.starts_with('/');

        let found = matches_for(&prefix, nicks, is_command);
        if found.is_empty() {
            self.reset();
            return None;
        }

        let index = index % found.len();
        let chosen = &found[index];

        // A nick opening the line gets "nick: ", the convention other clients
        // use and which most bots and highlight rules expect. Anywhere else a
        // plain space is right, as is a completed command.
        let suffix = if is_command || start != 0 { " " } else { ": " };
        let insert = format!("{chosen}{suffix}");

        let head = slice(&cs, 0, start);
        let tail = slice(&cs, replace_end, cs.len());
        let new_text = format!("{head}{insert}{tail}");
        let new_cursor = start + insert.chars().count();

        self.prefix = prefix;
        self.start = start;
        self.end = new_cursor;
        self.index = index;
        self.last = new_text.clone();

        Some(Completion {
            text: new_text,
            cursor: new_cursor,
            match_count: found.len(),
        })
    }
}
