use std::fmt;

#[derive(Debug, Clone)]
pub struct IrcConfig {
    pub server: String,
    pub port: u16,
    pub tls: bool,
    pub nick: String,
    pub initial_channel: String,
}

#[derive(Debug, Clone)]
pub enum BackendCmd {
    Connect { conn_id: u64, cfg: IrcConfig },
    SendRaw { conn_id: u64, line: String },
    Disconnect { conn_id: u64, reason: String },
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    // Connection lifecycle
    ConnectionUp {
        conn_id: u64,
        server: String,
        nick: String,
        initial_channel: String,
    },
    ConnectionDown {
        conn_id: u64,
        reason: String,
    },

    // Buffer operations
    EnsureBuffer {
        conn_id: u64,
        buffer: String,
        make_current: bool,
    },
    Append {
        conn_id: u64,
        buffer: String,
        line: String,
        bump_unread: bool,
        bump_highlight: bool,
    },
    SetMyNick {
        conn_id: u64,
        nick: String,
    },

    // User list per channel
    SetUsers {
        conn_id: u64,
        channel: String,
        users: Vec<String>,
    },
    AddUser {
        conn_id: u64,
        channel: String,
        nick: String,
    },
    RemoveUser {
        conn_id: u64,
        channel: String,
        nick: String,
    },
    RemoveUserEverywhere {
        conn_id: u64,
        nick: String,
    },
    RenameUserEverywhere {
        conn_id: u64,
        old: String,
        new_: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferKey {
    pub conn_id: u64,
    pub name: String,
}

impl BufferKey {
    pub fn new(conn_id: u64, name: impl Into<String>) -> Self {
        Self {
            conn_id,
            name: name.into(),
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}|{}", self.conn_id, self.name)
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let (a, b) = s.split_once('|')?;
        let conn_id: u64 = a.parse().ok()?;
        Some(Self {
            conn_id,
            name: b.to_string(),
        })
    }
}

impl fmt::Display for BufferKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}", self.conn_id, self.name)
    }
}

#[derive(Debug, Clone)]
pub struct ConnMeta {
    pub server: String,
    pub nick: String,
    pub default_target: String,
}
