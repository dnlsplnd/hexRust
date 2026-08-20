use crate::irc;
use crate::model::{BackendCmd, IrcConfig, UiEvent};
use crate::util::ts_prefix;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

/// Longest gap between reconnection attempts. Attempts back off 1, 2, 4, 8,
/// 16 seconds and then hold here, which is roughly two attempts a minute --
/// persistent enough to recover unattended, sparse enough not to hammer a
/// server that is refusing us.
const MAX_RECONNECT_DELAY: u64 = 30;

#[derive(Clone)]
struct ConnHandle {
    raw_tx: tokio::sync::mpsc::UnboundedSender<String>,
}

#[derive(Clone)]
struct BackendState {
    /// Live connections. A connection is absent while it is waiting to
    /// reconnect, so sends during that window are reported rather than
    /// dropped into a channel nothing is reading.
    conns: Arc<Mutex<HashMap<u64, ConnHandle>>>,
    /// Set when the user asked to disconnect. Kept separately from `conns`
    /// because it has to outlive the connection itself: it is what stops the
    /// reconnect loop, which runs precisely when there is no connection.
    quits: Arc<Mutex<HashMap<u64, Arc<AtomicBool>>>>,
}

pub fn start_backend(ui_tx: mpsc::Sender<UiEvent>) -> mpsc::Sender<BackendCmd> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<BackendCmd>();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime");

        rt.block_on(async move {
            let (tcmd_tx, mut tcmd_rx) = tokio::sync::mpsc::unbounded_channel::<BackendCmd>();

            // Bridge std::mpsc -> tokio mpsc.
            let bridge_tx = tcmd_tx.clone();
            std::thread::spawn(move || {
                while let Ok(cmd) = cmd_rx.recv() {
                    let _ = bridge_tx.send(cmd);
                }
            });

            let state = BackendState {
                conns: Arc::new(Mutex::new(HashMap::new())),
                quits: Arc::new(Mutex::new(HashMap::new())),
            };

            while let Some(cmd) = tcmd_rx.recv().await {
                if let Err(e) = handle_cmd(cmd, &state, &ui_tx).await {
                    let _ = ui_tx.send(UiEvent::Append {
                        conn_id: 0,
                        buffer: "Status".to_string(),
                        line: format!("{} *** backend error: {e:#}", ts_prefix()),
                        bump_unread: true,
                        bump_highlight: false,
                    });
                }
            }
        });
    });

    cmd_tx
}

async fn handle_cmd(cmd: BackendCmd, state: &BackendState, ui_tx: &mpsc::Sender<UiEvent>) -> Result<()> {
    match cmd {
        BackendCmd::Connect { conn_id, cfg } => {
            connect(conn_id, cfg, state, ui_tx).await?;
        }
        BackendCmd::SendRaw { conn_id, line } => {
            let conns = state.conns.lock().await;
            if let Some(h) = conns.get(&conn_id) {
                let _ = h.raw_tx.send(line);
            } else {
                let _ = ui_tx.send(UiEvent::Append {
                    conn_id,
                    buffer: "Status".to_string(),
                    line: format!("{} *** Not connected (conn_id={conn_id}).", ts_prefix()),
                    bump_unread: true,
                    bump_highlight: false,
                });
            }
        }
        BackendCmd::Disconnect { conn_id, reason } => {
            // Mark the intent first: the reconnect loop checks this as soon as
            // the connection ends, so it must be set before we cause the end.
            if let Some(flag) = state.quits.lock().await.get(&conn_id) {
                flag.store(true, Ordering::SeqCst);
            }
            let mut conns = state.conns.lock().await;
            if let Some(h) = conns.remove(&conn_id) {
                let _ = h.raw_tx.send(format!("QUIT :{reason}"));
            }
            let _ = ui_tx.send(UiEvent::ConnectionDown { conn_id, reason });
        }
    }
    Ok(())
}

async fn connect(conn_id: u64, cfg: IrcConfig, state: &BackendState, ui_tx: &mpsc::Sender<UiEvent>) -> Result<()> {
    let quit = Arc::new(AtomicBool::new(false));
    state.quits.lock().await.insert(conn_id, quit.clone());

    let _ = ui_tx.send(UiEvent::ConnectionUp {
        conn_id,
        server: cfg.server.clone(),
        nick: cfg.nick.clone(),
        initial_channel: cfg.initial_channel.clone(),
    });

    let ui_tx_task = ui_tx.clone();
    let conns = state.conns.clone();
    let quits = state.quits.clone();

    tokio::spawn(async move {
        let mut attempt: u32 = 0;

        loop {
            // A fresh channel per attempt: the previous receiver was consumed
            // by the run that just ended.
            let (raw_tx, raw_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            conns
                .lock()
                .await
                .insert(conn_id, ConnHandle { raw_tx: raw_tx.clone() });

            let result = irc::irc_run(
                conn_id,
                cfg.clone(),
                ui_tx_task.clone(),
                raw_rx,
                raw_tx.clone(),
            )
            .await;

            // Not connected from here until the next attempt succeeds.
            conns.lock().await.remove(&conn_id);

            if quit.load(Ordering::SeqCst) {
                break;
            }

            // A clean end of stream is a disconnect too. Reporting only the
            // error case left the UI believing it was still connected.
            let reason = match result {
                Ok(()) => "connection closed by server".to_string(),
                Err(e) => format!("{e:#}"),
            };
            let _ = ui_tx_task.send(UiEvent::ConnectionDown {
                conn_id,
                reason: reason.clone(),
            });

            attempt += 1;
            let delay = reconnect_delay(attempt);
            let _ = ui_tx_task.send(UiEvent::Append {
                conn_id,
                buffer: "Status".to_string(),
                line: format!(
                    "{} *** {reason}. Reconnecting in {delay}s (attempt {attempt})…",
                    ts_prefix()
                ),
                bump_unread: true,
                bump_highlight: false,
            });

            // Wake early if the user disconnects while we are waiting, rather
            // than making them sit out the whole backoff.
            let mut waited = 0;
            while waited < delay {
                tokio::time::sleep(Duration::from_secs(1)).await;
                if quit.load(Ordering::SeqCst) {
                    break;
                }
                waited += 1;
            }
            if quit.load(Ordering::SeqCst) {
                break;
            }
        }

        quits.lock().await.remove(&conn_id);
    });

    Ok(())
}

/// Seconds to wait before attempt `n`: 1, 2, 4, 8, 16, then held at the cap.
///
/// Public so the backoff policy can be asserted directly, rather than
/// inferred from wall-clock timing in an integration test.
pub fn reconnect_delay(attempt: u32) -> u64 {
    let shift = attempt.saturating_sub(1).min(16);
    (1u64 << shift).min(MAX_RECONNECT_DELAY)
}
