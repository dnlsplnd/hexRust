//! Live network test against a real IRC server.
//!
//! Ignored by default so ordinary `cargo test` stays offline. Run with:
//!   cargo test --test live_connect -- --ignored --nocapture

use hexrust::model::{IrcConfig, UiEvent};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn nick() -> String {
    // Keep it unique-ish so repeat runs don't collide on the server.
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 100000;
    format!("hexr{n}")
}

/// Drives irc_run against a server and returns every line that reached the UI.
async fn collect(cfg: IrcConfig, secs: u64) -> Vec<String> {
    let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
    let (raw_tx, raw_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let run = tokio::spawn(hexrust::irc::irc_run(1, cfg, ui_tx, raw_rx, raw_tx));

    // ui_rx is a blocking std channel, so drain it off the async runtime.
    let lines = tokio::task::spawn_blocking(move || {
        let mut lines = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(secs);
        while let Some(left) = deadline.checked_duration_since(Instant::now()) {
            match ui_rx.recv_timeout(left) {
                Ok(UiEvent::Append { line, .. }) => {
                    println!("  {line}");
                    lines.push(line);
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        lines
    })
    .await
    .unwrap();

    run.abort();
    lines
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn connects_and_registers_without_sasl() {
    let cfg = IrcConfig {
        server: "irc.libera.chat".into(),
        port: 6697,
        tls: true,
        nick: nick(),
        initial_channel: "#hexrust-test".into(),
        server_password: None,
        sasl_username: None,
        sasl_password: None,
    };

    let lines = collect(cfg, 30).await;
    assert!(!lines.is_empty(), "no output at all from the server");

    let joined = lines.iter().any(|l| l.contains("Joining"));
    assert!(joined, "never reached the 001 welcome / join stage");
}
