# hexrust

A multi-server IRC client in Rust and GTK4.

- Several servers at once, each with its own buffers
- Sidebar tree of servers and buffers, with per-buffer unread and highlight
  counters shown as `(!h, u)`
- Highlights on your nick and on any direct message
- Command palette (Ctrl+P) for quick `connect` / `join` / `switch` / `raw`
- Saved connection profiles, with SASL (PLAIN) and ZNC support
- Tab completion for nicks and commands, and a right-click menu on the user list
- Persistent per-buffer logs, plus in-buffer search
- TRON theme: cyan on black, amber for the active tab

## Build and run

Fedora dependencies:

```bash
sudo dnf install -y rust cargo gtk4-devel openssl-devel pkgconf-pkg-config
```

```bash
cargo run
```

## Commands

Anything not listed here is passed straight to the server, so `/WALLOPS hi`
works without hexrust needing to know the command.

| Command | Effect |
| --- | --- |
| `/join #chan` | Join a channel |
| `/part [#chan] [reason]` | Leave a channel, defaulting to the current one |
| `/query <nick> [text]` | Open a private buffer, optionally sending a line |
| `/msg <nick> <text>` | Send a private message |
| `/me <action>` | Send a CTCP ACTION |
| `/notice <target> <text>` | Send a notice |
| `/nick <newnick>` | Change nick |
| `/topic [#chan] [text]` | Show or set the topic |
| `/names [#chan]` | Refresh the user list |
| `/whois <nick>`, `/whowas <nick>` | Look someone up |
| `/kick [#chan] <nick> [reason]` | Kick a user |
| `/ban <mask>`, `/unban <mask>` | Set or clear a ban |
| `/op`, `/deop`, `/voice`, `/devoice` | Channel modes, one or more nicks at a time |
| `/mode [target] <modes>` | Set modes; the target defaults to the current channel |
| `/invite <nick> [#chan]` | Invite someone |
| `/away [reason]`, `/back` | Set or clear away status |
| `/ctcp <target> <cmd>`, `/ping <nick>`, `/version [nick]` | CTCP requests |
| `/list [params]`, `/motd` | Server queries |
| `/oper <user> <pass>` | Operator login (never echoed to a buffer) |
| `/switch <buffer>` | Jump to a buffer by substring |
| `/server <id> <raw>` | Send a raw line to a specific connection |
| `/raw <line>` | Send a raw line to the current connection |
| `/quit [reason]` | Disconnect |
| `/help` | List the above in the client |

Commands that need a channel (`/part`, `/topic`, `/kick`, `/op`, …) use the
active buffer, falling back to the connection's default channel when typed in
Status. If neither is available they say so rather than sending a broken line.

## Tab completion

Tab completes the word before the cursor. A nick opening the line is completed
as `nick: `, matching what other clients and most highlight rules expect;
anywhere else it gets a plain space. A word starting with `/` at the very
start of the line completes against the command list instead.

Press Tab again to cycle through the candidates. Typing anything starts a
fresh completion rather than continuing the old cycle.

## User list menu

Right-click a nick in the user list for Whois, Query, Op, Deop, Voice,
Devoice, Kick and Ban. Each entry runs the same command the keyboard would, so
there is one implementation behind both.

## Profiles

Stored at `~/.config/hexrust/profiles.toml`. Click **Profiles…** in the top
bar to add, edit, or connect.

Both SASL and ZNC credentials live in that file **in plain text**, so treat it
accordingly.

### SASL

Fill `sasl_username` and `sasl_password` and hexrust negotiates CAP and
authenticates on connect. Only the PLAIN mechanism is supported. If the server
does not offer SASL, or refuses it, the client says so and continues
unauthenticated rather than stalling.

### ZNC

Profiles can hold a ZNC username, network, and password. These are combined
into the IRC `PASS` string ZNC expects:

- `username:password` when no network is set
- `username/network:password` when one is

See <https://wiki.znc.in/Connecting_to_ZNC>.

## Logs

Per-buffer logs are written to
`~/.local/share/hexrust/logs/<server>/<buffer>.log`.

- **Find** (or Ctrl+F) with **Find Next** steps through matches in the buffer
- **Load Log** replaces the buffer view with the on-disk log, which is how you
  read back history from before a restart

## Tests

```bash
cargo test                                        # offline, deterministic
cargo test --test live_connect -- --ignored       # connects to Libera.Chat
```

The offline tests drive the client against a scripted local IRC server, so
they cover registration, SASL, and message routing without a network. The live
test skips itself if the network refuses the connection on policy grounds
(some addresses are required to use SASL).
