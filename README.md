# hexrust (multi-server) v0.3.3

This build fixes GTK ListBoxRow metadata handling:
- No more `row.set_data()` / `row.data()` (those are unsafe / pointer-based in gtk4-rs)
- Buffer keys are stored in `row.set_widget_name(...)` and read back via `row.widget_name()`

Major features:
- Multiple servers at once (each connection has its own conn_id)
- Left sidebar "tree" of servers + buffers (click to switch)
- Per-buffer unread + highlight counters: `(!h, u)`
- Highlights when your nick is mentioned (or any direct PM)
- Command palette (Ctrl+P) with quick `connect/join/switch/raw`
- Terminal theme: black background + green text everywhere
- UTF-8 safe decoding for incoming bytes

## Fedora 43 dependencies

```bash
sudo dnf install -y gtk4-devel openssl-devel pkgconf-pkg-config
```

## Run

```bash
cargo run
```


## v0.3.3 change
- User list is now sorted by rank (~ & @ % +), then alphabetically.
- NAMES parsing keeps prefixes to reflect operator status.

## v0.3.3 fix
- Fixed a Rust borrow-lifetime issue in the AddUser handler (E0716).
- Removed an unused import warning in irc.rs.
