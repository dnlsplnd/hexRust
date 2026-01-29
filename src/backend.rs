use crate::irc;
use crate::model::{BackendCmd, IrcConfig, UiEvent};
use crate::util::ts_prefix;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Clone)]
struct ConnHandle {
    raw_tx: tokio::sync::mpsc::UnboundedSender<String>,
}

#[derive(Clone)]
struct BackendState {
    conns: Arc<Mutex<HashMap<u64, ConnHandle>>>,
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
            // Best-effort: send QUIT and drop the sender handle.
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
    // Create a per-connection raw channel.
    let (raw_tx, raw_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Store handle early so UI can send immediately.
    {
        let mut conns = state.conns.lock().await;
        conns.insert(conn_id, ConnHandle { raw_tx: raw_tx.clone() });
    }

    let _ = ui_tx.send(UiEvent::ConnectionUp {
        conn_id,
        server: cfg.server.clone(),
        nick: cfg.nick.clone(),
        initial_channel: cfg.initial_channel.clone(),
    });

    let ui_tx_task = ui_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = irc::irc_run(conn_id, cfg, ui_tx_task.clone(), raw_rx, raw_tx.clone()).await {
            let _ = ui_tx_task.send(UiEvent::ConnectionDown {
                conn_id,
                reason: format!("{e:#}"),
            });
        }
    });

    Ok(())
}
