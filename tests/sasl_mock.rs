//! SASL / capability-negotiation tests against a scripted local IRC server.
//!
//! These are offline and deterministic, so they run in a normal `cargo test`.

use hexrust::model::{IrcConfig, UiEvent};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// A one-shot IRC server that replies according to `reply`.
///
/// `reply` receives each line the client sent and returns lines to send back.
/// Every line the client sent is recorded for assertions.
async fn mock_server(
    reply: impl Fn(&str) -> Vec<String> + Send + 'static,
) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let log2 = Arc::clone(&log);

    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 1];
        let mut cur = Vec::<u8>::new();
        loop {
            match sock.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            if buf[0] != b'\n' {
                if buf[0] != b'\r' {
                    cur.push(buf[0]);
                }
                continue;
            }
            let line = String::from_utf8_lossy(&cur).to_string();
            cur.clear();
            if line.is_empty() {
                continue;
            }
            log2.lock().unwrap().push(line.clone());
            for out in reply(&line) {
                if sock.write_all(format!("{out}\r\n").as_bytes()).await.is_err() {
                    return;
                }
            }
        }
    });

    (port, log)
}

fn cfg_with_sasl(port: u16) -> IrcConfig {
    IrcConfig {
        server: "127.0.0.1".into(),
        port,
        tls: false,
        nick: "n".into(),
        initial_channel: "#c".into(),
        server_password: None,
        sasl_username: Some("u".into()),
        sasl_password: Some("p".into()),
    }
}

/// Runs the client against the mock for `secs`, returning the UI lines.
async fn drive(cfg: IrcConfig, secs: u64) -> Vec<String> {
    let (ui_tx, ui_rx) = std::sync::mpsc::channel::<UiEvent>();
    let (raw_tx, raw_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let run = tokio::spawn(hexrust::irc::irc_run(1, cfg, ui_tx, raw_rx, raw_tx));

    let lines = tokio::task::spawn_blocking(move || {
        let mut lines = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(secs);
        while let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) {
            match ui_rx.recv_timeout(left) {
                Ok(UiEvent::Append { line, .. }) => lines.push(line),
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

/// Happy path: server offers SASL, client authenticates and registers.
#[tokio::test(flavor = "multi_thread")]
async fn sasl_handshake_completes_and_sends_correct_payload() {
    let (port, log) = mock_server(|line| match line {
        l if l.starts_with("CAP LS") => vec![":srv CAP * LS :multi-prefix sasl=PLAIN".into()],
        l if l.starts_with("CAP REQ") => vec![":srv CAP * ACK :sasl".into()],
        "AUTHENTICATE PLAIN" => vec!["AUTHENTICATE +".into()],
        l if l.starts_with("AUTHENTICATE ") => {
            vec![":srv 903 n :SASL authentication successful".into()]
        }
        "CAP END" => vec![":srv 001 n :Welcome".into()],
        _ => vec![],
    })
    .await;

    let lines = drive(cfg_with_sasl(port), 5).await;
    let sent = log.lock().unwrap().clone();

    assert!(
        sent.iter().any(|l| l == "AUTHENTICATE dQB1AHA="),
        "wrong or missing SASL PLAIN payload; sent: {sent:?}"
    );
    assert!(sent.iter().any(|l| l == "CAP END"), "never ended CAP: {sent:?}");
    assert!(
        lines.iter().any(|l| l.contains("SASL: success")),
        "no success reported: {lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.contains("Joining")),
        "never registered after SASL: {lines:?}"
    );
}

/// Server does NOT offer SASL. The client must still finish capability
/// negotiation with CAP END, or registration stalls forever.
#[tokio::test(flavor = "multi_thread")]
async fn sasl_unavailable_still_ends_cap_negotiation() {
    let (port, log) = mock_server(|line| match line {
        // Note: no "sasl" in the advertised capabilities.
        l if l.starts_with("CAP LS") => vec![":srv CAP * LS :multi-prefix account-notify".into()],
        "CAP END" => vec![":srv 001 n :Welcome".into()],
        _ => vec![],
    })
    .await;

    let lines = drive(cfg_with_sasl(port), 5).await;
    let sent = log.lock().unwrap().clone();

    assert!(
        sent.iter().any(|l| l == "CAP END"),
        "client never sent CAP END when SASL was unavailable, so the server \
         holds registration open forever; sent: {sent:?}"
    );
    assert!(
        lines.iter().any(|l| l.contains("Joining")),
        "never registered: {lines:?}"
    );
}

/// The server refuses the capability we requested. Same requirement: the
/// client has to end negotiation rather than wait forever.
#[tokio::test(flavor = "multi_thread")]
async fn sasl_nak_still_ends_cap_negotiation() {
    let (port, log) = mock_server(|line| match line {
        l if l.starts_with("CAP LS") => vec![":srv CAP * LS :sasl=PLAIN".into()],
        l if l.starts_with("CAP REQ") => vec![":srv CAP * NAK :sasl".into()],
        "CAP END" => vec![":srv 001 n :Welcome".into()],
        _ => vec![],
    })
    .await;

    let lines = drive(cfg_with_sasl(port), 5).await;
    let sent = log.lock().unwrap().clone();

    assert!(sent.iter().any(|l| l == "CAP END"), "no CAP END after NAK: {sent:?}");
    assert!(
        lines.iter().any(|l| l.contains("Joining")),
        "never registered after NAK: {lines:?}"
    );
}

/// CAP LS 302 may split the capability list across lines, with "*" marking
/// "more to come". SASL arriving in a later line must still be picked up,
/// and the client must not give up after the first line.
#[tokio::test(flavor = "multi_thread")]
async fn sasl_advertised_in_later_multiline_cap_ls() {
    let (port, log) = mock_server(|line| match line {
        l if l.starts_with("CAP LS") => vec![
            ":srv CAP * LS * :multi-prefix account-notify".into(),
            ":srv CAP * LS :away-notify sasl=PLAIN".into(),
        ],
        l if l.starts_with("CAP REQ") => vec![":srv CAP * ACK :sasl".into()],
        "AUTHENTICATE PLAIN" => vec!["AUTHENTICATE +".into()],
        l if l.starts_with("AUTHENTICATE ") => {
            vec![":srv 903 n :SASL authentication successful".into()]
        }
        "CAP END" => vec![":srv 001 n :Welcome".into()],
        _ => vec![],
    })
    .await;

    let lines = drive(cfg_with_sasl(port), 5).await;
    let sent = log.lock().unwrap().clone();

    assert!(
        sent.iter().any(|l| l == "CAP REQ :sasl"),
        "missed sasl in the continuation line: {sent:?}"
    );
    assert!(
        lines.iter().any(|l| l.contains("SASL: success")),
        "handshake did not complete: {lines:?}"
    );
}
