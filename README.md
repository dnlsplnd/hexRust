<<<<<<< HEAD
# hexrust (multi-server) v0.3.3
=======
# hexrust (multi-server) v0.5.0
>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)

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


<<<<<<< HEAD
## v0.3.3 change
- User list is now sorted by rank (~ & @ % +), then alphabetically.
- NAMES parsing keeps prefixes to reflect operator status.

## v0.3.3 fix
- Fixed a Rust borrow-lifetime issue in the AddUser handler (E0716).
- Removed an unused import warning in irc.rs.
=======
## v0.5.0 change
- User list is now sorted by rank (~ & @ % +), then alphabetically.
- NAMES parsing keeps prefixes to reflect operator status.

## v0.5.0 fix
- Fixed a Rust borrow-lifetime issue in the AddUser handler (E0716).
- Removed an unused import warning in irc.rs.


## Profiles + SASL (v0.5.0)

hexRust now supports saved connection profiles and optional SASL authentication (PLAIN).

Profiles are stored at:

- `~/.config/hexrust/profiles.toml` (on Fedora/Linux)

### Using Profiles

- Click **Profiles…** in the top bar
- Add or edit profiles
- Select a profile and click **Connect**

### SASL Notes

- SASL is optional. If you fill `sasl_username` and `sasl_password`, hexRust will attempt CAP/SASL auth on connect.
- Currently supported mechanism: **PLAIN**
- Passwords are stored in plain text in `profiles.toml`. Use at your own risk.

## v0.5.0 fix
- Fixed GTK closure ownership/lifetime issues in the Profiles manager.


## ZNC support (v0.5.0)

Profiles can now store ZNC login parts:

- ZNC username
- ZNC network (optional)
- ZNC server password

hexRust will automatically build an IRC PASS string:

- `username:password` (if network is empty)
- `username/network:password` (if network is set)

This matches common ZNC client login formats. See: https://wiki.znc.in/Connecting_to_ZNC

## v0.5.0
- Profiles UI: Regular and ZNC profiles are shown in separate tabs.
- Fixed Save button: errors are now surfaced and saves are not silently ignored.

## v0.5.0
- Fixed missing GTK import for Frame in Profiles UI.

## v0.5.0
- Theme: enforced pure green (#00ff00) on black (#000000) everywhere, monospace 9pt.

## v0.5.0
- Theme: fixed GTK slider sizing warnings by setting explicit min sizes for scrollbar/scale sliders.


## v0.5.0

Major step: **persistent logs + buffer search**

- Persistent per-buffer logs stored under:
  - `~/.local/share/hexrust/logs/<server>/<buffer>.log`
- **Find**: use the Find box (or Ctrl+F) and click **Find Next** to jump through matches.
- **Load Log**: replaces the current buffer view with the on-disk log (useful after restarts).

## v0.5.2
- Fixed RefCell borrow lifetime issues in Find/Load Log helpers.
>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)
