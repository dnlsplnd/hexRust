//! Reconnection behaviour, driven through the real backend.

use hexrust::backend::start_backend;
use hexrust::model::{BackendCmd, IrcConfig, UiEvent};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Accepts connections forever, running `handle` on each, and counts them.
fn mock_server(handle: impl Fn(&mut TcpStream) + Send + Sync + 'static) -> (u16, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let count = Arc::new(AtomicUsize::new(0));
    let count2 = Arc::clone(&count);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { break };
            count2.fetch_add(1, Ordering::SeqCst);
            let h = &handle;
            h(&mut s);
        }
    });

    (port, count)
}

/// Reads until the client finishes registration, so replies are well timed.
fn wait_for_user(s: &mut TcpStream) {
    let peek = s.try_clone().unwrap();
    let mut r = BufReader::new(peek);
    let mut line = String::new();
    while r.read_line(&mut line).unwrap_or(0) > 0 {
        if line.starts_with("USER ") {
            return;
        }
        line.clear();
    }
}

fn cfg(port: u16) -> IrcConfig {
    IrcConfig {
        server: "127.0.0.1".into(),
        port,
        tls: false,
        nick: "n".into(),
        initial_channel: "#c".into(),
        server_password: None,
        sasl_username: None,
        sasl_password: None,
    }
}

/// Collects UI lines in the background so the channel never fills.
fn collect(rx: mpsc::Receiver<UiEvent>) -> Arc<std::sync::Mutex<Vec<String>>> {
    let out = Arc::new(std::sync::Mutex::new(Vec::new()));
    let out2 = Arc::clone(&out);
    std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() {
            if let UiEvent::Append { line, .. } = ev {
                out2.lock().unwrap().push(line);
            }
        }
    });
    out
}

fn wait_until(deadline: Duration, mut cond: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < deadline {
        if cond() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

/// A dropped connection must come back on its own; otherwise the client is
/// dead until the user notices and reconnects by hand.
#[test]
fn a_dropped_connection_is_retried() {
    let (port, count) = mock_server(|s| {
        wait_for_user(s);
        let _ = s.write_all(b":srv 001 n :Welcome\r\n");
        // Drop immediately, as a server or a network blip would.
        let _ = s.shutdown(std::net::Shutdown::Both);
    });

    let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
    let lines = collect(ui_rx);
    let tx = start_backend(ui_tx);
    tx.send(BackendCmd::Connect { conn_id: 1, cfg: cfg(port) }).unwrap();

    assert!(
        wait_until(Duration::from_secs(8), || count.load(Ordering::SeqCst) >= 2),
        "never reconnected; connections seen: {}",
        count.load(Ordering::SeqCst)
    );

    let l = lines.lock().unwrap().clone();
    assert!(
        l.iter().any(|x| x.contains("Reconnecting in")),
        "reconnection was not reported to the user: {l:?}"
    );
}

/// Backoff must grow, or a server that is down gets hammered.
#[test]
fn repeated_failures_back_off() {
    let (port, count) = mock_server(|s| {
        let _ = s.shutdown(std::net::Shutdown::Both);
    });

    let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
    let lines = collect(ui_rx);
    let tx = start_backend(ui_tx);
    tx.send(BackendCmd::Connect { conn_id: 1, cfg: cfg(port) }).unwrap();

    // Long enough for several attempts at 1s, 2s, 4s.
    std::thread::sleep(Duration::from_secs(9));

    let l = lines.lock().unwrap().clone();
    let delays: Vec<u64> = l
        .iter()
        .filter_map(|x| {
            let i = x.find("Reconnecting in ")? + "Reconnecting in ".len();
            let rest = &x[i..];
            let j = rest.find('s')?;
            rest[..j].parse::<u64>().ok()
        })
        .collect();

    assert!(delays.len() >= 3, "too few attempts to judge backoff: {delays:?}");
    assert!(
        delays.windows(2).all(|w| w[1] >= w[0]),
        "delay did not grow monotonically: {delays:?}"
    );
    assert!(delays[0] < delays[delays.len() - 1], "no backoff at all: {delays:?}");

    let seen = count.load(Ordering::SeqCst);
    assert!(seen < 20, "hammered the server with {seen} attempts in 9s");
}

/// Disconnecting on purpose must not be undone by the reconnect loop.
#[test]
fn an_explicit_disconnect_is_not_retried() {
    let (port, count) = mock_server(|s| {
        wait_for_user(s);
        let _ = s.write_all(b":srv 001 n :Welcome\r\n");
        // Stay open until the client goes away.
        let peek = s.try_clone().unwrap();
        let mut r = BufReader::new(peek);
        let mut line = String::new();
        while r.read_line(&mut line).unwrap_or(0) > 0 {
            if line.starts_with("QUIT") {
                break;
            }
            line.clear();
        }
        let _ = s.shutdown(std::net::Shutdown::Both);
    });

    let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
    let _lines = collect(ui_rx);
    let tx = start_backend(ui_tx);
    tx.send(BackendCmd::Connect { conn_id: 1, cfg: cfg(port) }).unwrap();

    assert!(
        wait_until(Duration::from_secs(5), || count.load(Ordering::SeqCst) >= 1),
        "never connected in the first place"
    );

    tx.send(BackendCmd::Disconnect { conn_id: 1, reason: "bye".into() }).unwrap();

    // Well past the first backoff steps.
    std::thread::sleep(Duration::from_secs(6));
    assert_eq!(
        count.load(Ordering::SeqCst),
        1,
        "reconnected after the user asked to disconnect"
    );
}
