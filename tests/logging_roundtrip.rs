//! Log persistence round-trip.
//!
//! These write into the real data dir under a throwaway server name so they
//! don't collide with a user's actual logs.

use hexrust::logging;

fn unique_server() -> String {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test.invalid.{n}")
}

#[test]
fn appended_lines_read_back_in_order() {
    let server = unique_server();
    logging::append_line(&server, "#chan", "first").unwrap();
    logging::append_line(&server, "#chan", "second").unwrap();

    let text = logging::read_all(&server, "#chan").unwrap();
    assert_eq!(text, "first\nsecond\n");
    assert!(logging::log_exists(&server, "#chan").unwrap());

    let _ = std::fs::remove_dir_all(logging::log_path(&server, "#chan").unwrap().parent().unwrap());
}

#[test]
fn separate_buffers_do_not_share_a_file() {
    let server = unique_server();
    logging::append_line(&server, "#a", "in-a").unwrap();
    logging::append_line(&server, "#b", "in-b").unwrap();

    assert_eq!(logging::read_all(&server, "#a").unwrap(), "in-a\n");
    assert_eq!(logging::read_all(&server, "#b").unwrap(), "in-b\n");

    let _ = std::fs::remove_dir_all(logging::log_path(&server, "#a").unwrap().parent().unwrap());
}

#[test]
fn reading_a_buffer_that_was_never_logged_is_empty_not_an_error() {
    let server = unique_server();
    assert_eq!(logging::read_all(&server, "#nope").unwrap(), "");
    assert!(!logging::log_exists(&server, "#nope").unwrap());
}

/// Buffer names arrive straight from the network, so path separators and
/// traversal must not escape the log directory.
#[test]
fn hostile_buffer_names_stay_inside_the_log_dir() {
    let base = logging::logs_base_dir().unwrap();
    for nasty in ["../../etc/passwd", "a/b", "..", "/abs"] {
        let p = logging::log_path("srv", nasty).unwrap();
        assert!(
            p.starts_with(&base),
            "buffer name {nasty:?} escaped the log dir: {}",
            p.display()
        );
    }
}
