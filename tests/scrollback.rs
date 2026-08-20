//! Buffer trimming. Needs a display, so it skips when there is none.

use hexrust::ui::{trim_buffer, MAX_BUFFER_LINES};

/// Appends `n` numbered lines the way the client does, trimming as it goes.
fn fill(buffer: &gtk::TextBuffer, n: usize, max: i32) {
    use gtk::prelude::*;
    for i in 0..n {
        let mut end = buffer.end_iter();
        buffer.insert(&mut end, &format!("line {i}"));
        buffer.insert(&mut end, "\n");
        trim_buffer(buffer, max);
    }
}

fn text(buffer: &gtk::TextBuffer) -> String {
    use gtk::prelude::*;
    buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string()
}

/// Returns None when no display is available, so the suite still passes
/// headless rather than reporting a false failure.
fn buffer() -> Option<gtk::TextBuffer> {
    if gtk::init().is_err() {
        eprintln!("SKIP: no display available for GTK tests");
        return None;
    }
    Some(gtk::TextBuffer::new(None))
}

#[test]
fn a_buffer_under_the_cap_is_untouched() {
    let Some(b) = buffer() else { return };
    fill(&b, 10, 100);
    let t = text(&b);
    assert!(t.starts_with("line 0\n"), "oldest line was dropped early: {t:.40}");
    assert!(t.ends_with("line 9\n"));
}

#[test]
fn the_oldest_lines_are_dropped_once_the_cap_is_passed() {
    let Some(b) = buffer() else { return };
    fill(&b, 250, 100);

    let t = text(&b);
    assert!(!t.contains("line 0\n"), "oldest line survived the trim");
    assert!(t.contains("line 249"), "newest line was trimmed away");

    let lines = t.lines().count();
    assert!(lines <= 100, "buffer kept {lines} lines, over the cap of 100");
    assert!(lines >= 95, "trimmed far more than needed: {lines} lines left");
}

/// The most recent traffic is the point of the buffer; it must never be the
/// part that gets dropped.
#[test]
fn the_newest_lines_are_always_the_ones_kept() {
    let Some(b) = buffer() else { return };
    fill(&b, 500, 50);
    let t = text(&b);
    for i in 490..500 {
        assert!(t.contains(&format!("line {i}")), "recent line {i} missing");
    }
}

#[test]
fn a_nonpositive_cap_disables_trimming() {
    let Some(b) = buffer() else { return };
    fill(&b, 20, 0);
    assert!(text(&b).contains("line 0"), "trimmed despite a disabled cap");
}

#[test]
fn the_shipped_cap_is_a_sane_size() {
    assert!(
        (500..=100_000).contains(&MAX_BUFFER_LINES),
        "cap of {MAX_BUFFER_LINES} is outside anything reasonable"
    );
}
