use crate::backend;
use crate::model::{BackendCmd, BufferKey, ConnMeta, UiEvent};
use crate::theme;
use crate::util::ts_prefix;

use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::{
<<<<<<< HEAD
    Application, ApplicationWindow, Button, Entry, Label, ListBox, ListBoxRow, Notebook, Orientation,
=======
    Application, ApplicationWindow, Button, Entry, Frame, Label, ListBox, ListBoxRow, Notebook, Orientation,
>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)
    Paned, ScrolledWindow, TextBuffer, TextView, WrapMode,
};

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone)]
struct Tab {
    page_num: u32,
    view: TextView,
    buffer: TextBuffer,
    tab_label: Label,
    base_label: String,
}

#[derive(Default)]
struct History {
    items: Vec<String>,
    pos: usize,
    scratch: String,
}

#[derive(Clone)]
enum SidebarItemKind {
    Header { conn_id: u64, title: String },
    Buffer { key: BufferKey, display: String, indent: u8 },
}

pub fn build_ui(app: &Application) {
    let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();

    let backend_tx = backend::start_backend(ui_tx.clone());
    let backend_tx: Rc<RefCell<Option<mpsc::Sender<BackendCmd>>>> =
        Rc::new(RefCell::new(Some(backend_tx)));

    // Connection counter
    let next_conn_id: Rc<RefCell<u64>> = Rc::new(RefCell::new(1));

    // UI state
    let conn_meta: Rc<RefCell<HashMap<u64, ConnMeta>>> = Rc::new(RefCell::new(HashMap::new()));
    let current_buf: Rc<RefCell<BufferKey>> = Rc::new(RefCell::new(BufferKey::new(0, "Status")));

    // Tabs and mapping
    let tabs: Rc<RefCell<HashMap<String, Tab>>> = Rc::new(RefCell::new(HashMap::new()));
    let page_to_buf: Rc<RefCell<HashMap<u32, String>>> = Rc::new(RefCell::new(HashMap::new()));

    // Unread + highlight counters
    let unread: Rc<RefCell<HashMap<String, u32>>> = Rc::new(RefCell::new(HashMap::new()));
    let highlights: Rc<RefCell<HashMap<String, u32>>> = Rc::new(RefCell::new(HashMap::new()));

    // Channel user state: (conn_id, channel) -> set
    let users: Rc<RefCell<HashMap<String, BTreeSet<String>>>> =
        Rc::new(RefCell::new(HashMap::new()));

    // Sidebar model
    let sidebar_items: Rc<RefCell<Vec<SidebarItemKind>>> = Rc::new(RefCell::new(Vec::new()));

    // Input history
    let history: Rc<RefCell<History>> = Rc::new(RefCell::new(History::default()));

    let win = ApplicationWindow::builder()
        .application(app)
        .title("hexrust (multi-server)")
        .default_width(1280)
        .default_height(760)
        .build();

    let root = gtk::Box::new(Orientation::Vertical, 8);
    root.set_margin_top(10);
    root.set_margin_bottom(10);
    root.set_margin_start(10);
    root.set_margin_end(10);

    // Top bar: connect controls + command palette button
    let top = gtk::Box::new(Orientation::Horizontal, 8);

    let server_entry = Entry::builder()
        .placeholder_text("Server (e.g. irc.libera.chat)")
        .hexpand(true)
        .text("irc.libera.chat")
        .build();

    let port_entry = Entry::builder()
        .placeholder_text("Port")
        .text("6697")
        .width_chars(6)
        .build();

    let nick_entry = Entry::builder()
        .placeholder_text("Nick")
        .text("hexrust")
        .width_chars(12)
        .build();

    let channel_entry = Entry::builder()
        .placeholder_text("Channel (e.g. #rust)")
        .text("#hexrust")
        .width_chars(18)
        .build();

    let connect_btn = Button::with_label("Add Connection");
    let palette_btn = Button::with_label("Command Palette (Ctrl+P)");
<<<<<<< HEAD
=======
    let profiles_btn = Button::with_label("Profiles…");
    let find_entry = Entry::builder().placeholder_text("Find… (Ctrl+F)").build();
    let find_next_btn = Button::with_label("Find Next");
    let load_log_btn = Button::with_label("Load Log");
>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)

    top.append(&server_entry);
    top.append(&port_entry);
    top.append(&nick_entry);
    top.append(&channel_entry);
    top.append(&connect_btn);
    top.append(&palette_btn);
<<<<<<< HEAD
=======
    top.append(&profiles_btn);
    top.append(&find_entry);
    top.append(&find_next_btn);
    top.append(&load_log_btn);
>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)

    // Left: sidebar tree (connections + buffers)
    let sidebar = gtk::Box::new(Orientation::Vertical, 8);
    let sidebar_title = Label::new(Some("Servers / Buffers"));
    sidebar_title.set_xalign(0.0);

    let sidebar_list = ListBox::new();
    sidebar_list.set_vexpand(true);
    sidebar_list.set_hexpand(true);

    let sidebar_scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&sidebar_list)
        .build();

    sidebar.append(&sidebar_title);
    sidebar.append(&sidebar_scroll);

    // Middle: notebook
    let notebook = Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);

    // Right: user list
    let users_panel = gtk::Box::new(Orientation::Vertical, 8);
    let users_title = Label::new(Some("Users"));
    users_title.set_xalign(0.0);

    let users_list = ListBox::new();
    users_list.set_vexpand(true);
    users_list.set_hexpand(true);

    let users_scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&users_list)
        .build();

    users_panel.append(&users_title);
    users_panel.append(&users_scroll);

    // Paned: sidebar | notebook | users
    let paned_outer = Paned::new(Orientation::Horizontal);
    paned_outer.set_hexpand(true);
    paned_outer.set_vexpand(true);

    let paned_inner = Paned::new(Orientation::Horizontal);
    paned_inner.set_hexpand(true);
    paned_inner.set_vexpand(true);

    paned_outer.set_start_child(Some(&sidebar));
    paned_outer.set_end_child(Some(&paned_inner));
    paned_outer.set_position(300);

    paned_inner.set_start_child(Some(&notebook));
    paned_inner.set_end_child(Some(&users_panel));
    paned_inner.set_position(980);

    // Input row
    let input_row = gtk::Box::new(Orientation::Horizontal, 8);
    let input_entry = Entry::builder()
        .placeholder_text("Message or /join /msg /me /nick /server /switch /raw …")
        .hexpand(true)
        .build();
    let send_btn = Button::with_label("Send");
    input_row.append(&input_entry);
    input_row.append(&send_btn);

    root.append(&top);
    root.append(&paned_outer);
    root.append(&input_row);

    win.set_child(Some(&root));

    theme::apply_terminal_theme();

<<<<<<< HEAD
=======
    // Load and seed profiles (config file).
    let profiles_path = crate::profiles::default_path().expect("profiles path");
    crate::profiles::seed_example_if_empty(&profiles_path).ok();

>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)
    // Ensure a global Status buffer (conn_id=0) for backend logs.
    ensure_buffer(
        BufferKey::new(0, "Status"),
        "Status".to_string(),
        true,
        &notebook,
        &tabs,
        &page_to_buf,
        &unread,
        &highlights,
    );
    append_to(
        &BufferKey::new(0, "Status"),
        &format!("{} *** Ready.", ts_prefix()),
        &tabs,
    );

    // Sidebar behavior: activate buffer.
    {
        let notebook = notebook.clone();
        let tabs = tabs.clone();
        let current_buf = current_buf.clone();
        let unread = unread.clone();
        let highlights = highlights.clone();

        sidebar_list.connect_row_activated(move |_list, row| {
            // We store the BufferKey as the widget name for safe retrieval (no unsafe set_data/data).
            let key_str = row.widget_name().to_string();
            if key_str.is_empty() {
                return;
            }
            if let Some(key) = BufferKey::from_string(&key_str) {
                *current_buf.borrow_mut() = key.clone();

                // Clear unread/highlights on visit.
                unread.borrow_mut().insert(key.as_string(), 0);
                highlights.borrow_mut().insert(key.as_string(), 0);
                update_tab_label(&key, &tabs, &unread, &highlights);

                select_buffer(&key, &notebook, &tabs);
            }
        });
    }

    // Notebook switch: keep sidebar counters consistent + refresh user list.
    {
        let page_to_buf = page_to_buf.clone();
        let current_buf = current_buf.clone();
        let unread = unread.clone();
        let highlights = highlights.clone();
        let tabs = tabs.clone();

        let users = users.clone();
        let users_list = users_list.clone();
        let users_title = users_title.clone();

        notebook.connect_switch_page(move |_nb, _page, page_num| {
            if let Some(key_str) = page_to_buf.borrow().get(&page_num).cloned() {
                if let Some(key) = BufferKey::from_string(&key_str) {
                    *current_buf.borrow_mut() = key.clone();
                    unread.borrow_mut().insert(key.as_string(), 0);
                    highlights.borrow_mut().insert(key.as_string(), 0);
                    update_tab_label(&key, &tabs, &unread, &highlights);

                    refresh_user_list(&users_list, &users_title, &key, &users.borrow());
                }
            }
        });
    }

    // Clicking a user opens a PM buffer for the current connection.
    {
        let notebook = notebook.clone();
        let tabs = tabs.clone();
        let page_to_buf = page_to_buf.clone();
        let current_buf = current_buf.clone();

        let unread = unread.clone();
        let highlights = highlights.clone();

        let conn_meta = conn_meta.clone();
        let sidebar_items = sidebar_items.clone();
        let sidebar_list = sidebar_list.clone();

        let input_entry = input_entry.clone();

        users_list.connect_row_activated(move |_list, row| {
            let Some(child) = row.child() else { return };
            let Some(label) = child.downcast_ref::<Label>() else { return };
            let nick = label.text().to_string();
            if nick.is_empty() || nick.starts_with("Waiting") || nick.starts_with("Select") {
                return;
            }

            let conn_id = current_buf.borrow().conn_id;
            if conn_id == 0 {
                return;
            }

            let key = BufferKey::new(conn_id, nick.clone());
            let display = display_for_buffer(&key, &conn_meta.borrow());
            ensure_buffer(
                key.clone(),
                display.clone(),
                true,
                &notebook,
                &tabs,
                &page_to_buf,
                &unread,
                &highlights,
            );
            add_sidebar_buffer(
                &key,
                &display,
                &sidebar_items,
                &sidebar_list,
                &conn_meta.borrow(),
            );
            select_buffer(&key, &notebook, &tabs);
            input_entry.grab_focus();
        });
    }

    // Input history: Up/Down arrows.
    {
        let hist = history.clone();
        let entry = input_entry.clone();

        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_c, key, _code, _state| {
            if key == gdk::Key::Up {
                history_up(&entry, &hist);
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::Down {
                history_down(&entry, &hist);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        input_entry.add_controller(controller);
    }

    // Command palette (Ctrl+P) + button.
    {
        let win = win.clone();
        let palette = create_command_palette(
            &win,
            &backend_tx,
            &next_conn_id,
            &conn_meta,
            &notebook,
            &tabs,
            &page_to_buf,
            &unread,
            &highlights,
            &sidebar_items,
            &sidebar_list,
            &current_buf,
        );

        let palette_clone = palette.clone();
        palette_btn.connect_clicked(move |_| {
            palette_clone.present();
        });

<<<<<<< HEAD
=======
        // Profiles manager
        let profiles_mgr = create_profiles_manager(
            &win,
            profiles_path.clone(),
            &backend_tx,
            &next_conn_id,
            &conn_meta,
            &notebook,
            &tabs,
            &page_to_buf,
            &unread,
            &highlights,
            &sidebar_items,
            &sidebar_list,
            &current_buf,
        );

        let profiles_mgr_clone = profiles_mgr.clone();
        profiles_btn.connect_clicked(move |_| {
            profiles_mgr_clone.present();
        });

        // Find Next
        {
            let tabs = tabs.clone();
            let current_buf = current_buf.clone();
            let find_entry = find_entry.clone();
            find_next_btn.connect_clicked(move |_| {
                find_next_in_current_buffer(&find_entry.text(), &current_buf, &tabs);
            });
        }

        // Load Log
        {
            let tabs = tabs.clone();
            let current_buf = current_buf.clone();
            let conn_meta = conn_meta.clone();
            load_log_btn.connect_clicked(move |_| {
                load_log_into_current_buffer(&current_buf, &conn_meta, &tabs);
            });
        }

>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)
        let palette_clone2 = palette.clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_c, key, _code, state| {
            let ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
            if ctrl && key == gdk::Key::p {
                palette_clone2.present();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        win.add_controller(controller);
    }

    // Connect button: add a new connection.
    {
        let backend_tx = backend_tx.clone();
        let next_conn_id = next_conn_id.clone();
        let conn_meta = conn_meta.clone();

        let notebook = notebook.clone();
        let tabs = tabs.clone();
        let page_to_buf = page_to_buf.clone();
        let unread = unread.clone();
        let highlights = highlights.clone();

        let sidebar_items = sidebar_items.clone();
        let sidebar_list = sidebar_list.clone();

        let current_buf = current_buf.clone();

        let server_entry = server_entry.clone();
        let port_entry = port_entry.clone();
        let nick_entry = nick_entry.clone();
        let channel_entry = channel_entry.clone();

        connect_btn.connect_clicked(move |_| {
            let Some(tx) = backend_tx.borrow().clone() else { return };

            let server = server_entry.text().trim().to_string();
            if server.is_empty() {
                append_to(
                    &BufferKey::new(0, "Status"),
                    &format!("{} *** Enter a server.", ts_prefix()),
                    &tabs,
                );
                return;
            }

            let port: u16 = port_entry.text().trim().parse().unwrap_or(6697);
            let nick = nick_entry.text().trim().to_string();
            let channel = channel_entry.text().trim().to_string();

            let nick = if nick.is_empty() { "hexrust".to_string() } else { nick };
            let channel = if channel.is_empty() { "#hexrust".to_string() } else { channel };

            let conn_id = {
                let mut n = next_conn_id.borrow_mut();
                let id = *n;
                *n += 1;
                id
            };

            // Update meta immediately (UI-first).
            conn_meta.borrow_mut().insert(
                conn_id,
                ConnMeta {
                    server: server.clone(),
                    nick: nick.clone(),
                    default_target: channel.clone(),
                },
            );

            // Add connection header + status + initial channel to sidebar and tabs.
            add_sidebar_connection(conn_id, &server, &sidebar_items, &sidebar_list);

            let status_key = BufferKey::new(conn_id, "Status");
            let status_display = display_for_buffer(&status_key, &conn_meta.borrow());
            ensure_buffer(
                status_key.clone(),
                status_display.clone(),
                false,
                &notebook,
                &tabs,
                &page_to_buf,
                &unread,
                &highlights,
            );
            add_sidebar_buffer(
                &status_key,
                &status_display,
                &sidebar_items,
                &sidebar_list,
                &conn_meta.borrow(),
            );

            let chan_key = BufferKey::new(conn_id, channel.clone());
            let chan_display = display_for_buffer(&chan_key, &conn_meta.borrow());
            ensure_buffer(
                chan_key.clone(),
                chan_display.clone(),
                true,
                &notebook,
                &tabs,
                &page_to_buf,
                &unread,
                &highlights,
            );
            add_sidebar_buffer(
                &chan_key,
                &chan_display,
                &sidebar_items,
                &sidebar_list,
                &conn_meta.borrow(),
            );

            *current_buf.borrow_mut() = chan_key.clone();
            select_buffer(&chan_key, &notebook, &tabs);

            // Tell backend to connect.
<<<<<<< HEAD
            let cfg = crate::model::IrcConfig {
                server,
                port,
                tls: true,
                nick,
                initial_channel: channel,
            };
=======
            let cfg = crate::model::IrcConfig { server, port, tls: true, nick, initial_channel: channel, server_password: None, sasl_username: None, sasl_password: None };
>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)
            let _ = tx.send(BackendCmd::Connect { conn_id, cfg });

            append_to(
                &BufferKey::new(conn_id, "Status"),
                &format!("{} *** Connection requested.", ts_prefix()),
                &tabs,
            );
        });
    }

    // Send handler: sends to current buffer and supports multi-server commands.
    {
        let backend_tx = backend_tx.clone();
        let conn_meta = conn_meta.clone();
        let current_buf = current_buf.clone();

        let notebook = notebook.clone();
        let tabs = tabs.clone();
        let page_to_buf = page_to_buf.clone();
        let unread = unread.clone();
        let highlights = highlights.clone();

        let sidebar_items = sidebar_items.clone();
        let sidebar_list = sidebar_list.clone();

        let history_for_send = history.clone();

        let input_entry_for_send = input_entry.clone();

        let send_now = move || {
            let Some(tx) = backend_tx.borrow().clone() else { return };
            let raw = input_entry_for_send.text().to_string();
            input_entry_for_send.set_text("");

            let msg = raw.trim();
            if msg.is_empty() {
                return;
            }

            // Record history.
            history_push(&history_for_send, msg.to_string());

            let key = current_buf.borrow().clone();
            if key.conn_id == 0 {
                append_to(
                    &key,
                    &format!("{} *** (no connection) {msg}", ts_prefix()),
                    &tabs,
                );
                return;
            }

            // Slash commands.
            if msg.starts_with('/') {
                let cmdline = msg[1..].trim();
                handle_slash_command(
                    &tx,
                    &key,
                    cmdline,
                    &conn_meta,
                    &notebook,
                    &tabs,
                    &page_to_buf,
                    &unread,
                    &highlights,
                    &sidebar_items,
                    &sidebar_list,
                );
                return;
            }

            // Normal text: PRIVMSG to the active buffer target.
            let target = if key.name == "Status" {
                conn_meta
                    .borrow()
                    .get(&key.conn_id)
                    .map(|m| m.default_target.clone())
                    .unwrap_or_else(|| "#hexrust".to_string())
            } else {
                key.name.clone()
            };

            // Ensure buffer exists locally.
            let target_key = BufferKey::new(key.conn_id, target.clone());
            let display = display_for_buffer(&target_key, &conn_meta.borrow());
            ensure_buffer(
                target_key.clone(),
                display.clone(),
                false,
                &notebook,
                &tabs,
                &page_to_buf,
                &unread,
                &highlights,
            );
            add_sidebar_buffer(
                &target_key,
                &display,
                &sidebar_items,
                &sidebar_list,
                &conn_meta.borrow(),
            );

            let _ = tx.send(BackendCmd::SendRaw {
                conn_id: key.conn_id,
                line: format!("PRIVMSG {target} :{msg}"),
            });

            // Echo locally.
            let nick = conn_meta
                .borrow()
                .get(&key.conn_id)
                .map(|m| m.nick.clone())
                .unwrap_or_else(|| "me".to_string());

            append_to(
                &target_key,
                &format!("{} <{}> {}", ts_prefix(), nick, msg),
                &tabs,
            );
        };

        let send_now_btn = send_now.clone();
        send_btn.connect_clicked(move |_| send_now_btn());
        input_entry.connect_activate(move |_| send_now());
    }

    // Pump UiEvents into GTK.
    {
        let notebook = notebook.clone();
        let tabs = tabs.clone();
        let page_to_buf = page_to_buf.clone();
        let current_buf = current_buf.clone();

        let unread = unread.clone();
        let highlights = highlights.clone();

        let conn_meta = conn_meta.clone();

        let sidebar_items = sidebar_items.clone();
        let sidebar_list = sidebar_list.clone();

        let users = users.clone();
        let users_list = users_list.clone();
        let users_title = users_title.clone();

        glib::timeout_add_local(Duration::from_millis(25), move || {
            while let Ok(ev) = ui_rx.try_recv() {
                match ev {
                    UiEvent::ConnectionUp {
                        conn_id,
                        server,
                        nick,
                        initial_channel,
                    } => {
                        // Ensure meta exists (backend-confirm). UI may have inserted it already.
                        conn_meta.borrow_mut().entry(conn_id).or_insert(ConnMeta {
                            server,
                            nick,
                            default_target: initial_channel,
                        });
                    }
                    UiEvent::ConnectionDown { conn_id, reason } => {
                        append_to(
                            &BufferKey::new(conn_id, "Status"),
                            &format!("{} *** DISCONNECTED: {reason}", ts_prefix()),
                            &tabs,
                        );
                    }
                    UiEvent::EnsureBuffer {
                        conn_id,
                        buffer,
                        make_current,
                    } => {
                        let key = BufferKey::new(conn_id, buffer.clone());
                        let display = display_for_buffer(&key, &conn_meta.borrow());
                        ensure_buffer(
                            key.clone(),
                            display.clone(),
                            make_current,
                            &notebook,
                            &tabs,
                            &page_to_buf,
                            &unread,
                            &highlights,
                        );
                        add_sidebar_buffer(
                            &key,
                            &display,
                            &sidebar_items,
                            &sidebar_list,
                            &conn_meta.borrow(),
                        );
                        if make_current {
                            select_buffer(&key, &notebook, &tabs);
                            *current_buf.borrow_mut() = key.clone();
                        }
                    }
                    UiEvent::Append {
                        conn_id,
                        buffer,
                        line,
                        bump_unread,
                        bump_highlight,
                    } => {
                        let key = BufferKey::new(conn_id, buffer.clone());
                        let display = display_for_buffer(&key, &conn_meta.borrow());
                        ensure_buffer(
                            key.clone(),
                            display.clone(),
                            false,
                            &notebook,
                            &tabs,
                            &page_to_buf,
                            &unread,
                            &highlights,
                        );
                        add_sidebar_buffer(
                            &key,
                            &display,
                            &sidebar_items,
                            &sidebar_list,
                            &conn_meta.borrow(),
                        );

                        append_to(&key, &line, &tabs);

                        // Bump counters if not the active buffer.
                        let active = current_buf.borrow().clone();
                        if bump_unread && active != key {
                            let k = key.as_string();
                            let v = unread.borrow().get(&k).copied().unwrap_or(0);
                            unread
                                .borrow_mut()
                                .insert(k.clone(), v.saturating_add(1));

                            if bump_highlight {
                                let h = highlights.borrow().get(&k).copied().unwrap_or(0);
                                highlights
                                    .borrow_mut()
                                    .insert(k.clone(), h.saturating_add(1));
                            }

                            update_tab_label(&key, &tabs, &unread, &highlights);
                        }
                    }
                    UiEvent::SetMyNick { conn_id, nick } => {
                        if let Some(m) = conn_meta.borrow_mut().get_mut(&conn_id) {
                            m.nick = nick;
                        }
                    }
                    UiEvent::SetUsers { conn_id, channel, users: list } => {
                        let k = chan_users_key(conn_id, &channel);

                        // Deduplicate by display nick, keep the highest-ranked prefix if multiple variants exist.
                        let mut best: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                        for u in list {
                            let disp = crate::util::nick_display(&u).to_lowercase();
                            let rank_u = crate::util::nick_rank(&u);
                            if let Some(prev) = best.get(&disp) {
                                let rank_prev = crate::util::nick_rank(prev);
                                if rank_u > rank_prev {
                                    best.insert(disp, u);
                                }
                            } else {
                                best.insert(disp, u);
                            }
                        }

                        let mut set = BTreeSet::new();
                        for (_k, v) in best {
                            set.insert(v);
                        }

                        users.borrow_mut().insert(k, set);
                        refresh_user_list(
                            &users_list,
                            &users_title,
                            &current_buf.borrow(),
                            &users.borrow(),
                        );
                    }
                    UiEvent::AddUser { conn_id, channel, nick } => {
                        // Normalize: remove any existing variants of this nick with status prefixes.
                        let key = chan_users_key(conn_id, &channel);
                        let disp = crate::util::nick_display(&nick).to_string();
                        let mut users_mut = users.borrow_mut();
                        let set = users_mut.entry(key).or_default();
                        set.retain(|x| crate::util::nick_display(x) != disp);
                        set.insert(nick);
                        refresh_user_list(
                            &users_list,
                            &users_title,
                            &current_buf.borrow(),
                            &users.borrow(),
                        );
                    }
                    UiEvent::RemoveUser { conn_id, channel, nick } => {
                        if let Some(set) = users
                            .borrow_mut()
                            .get_mut(&chan_users_key(conn_id, &channel))
                        {
                            let disp = crate::util::nick_display(&nick).to_string();
                            set.retain(|x| crate::util::nick_display(x) != disp);
                        }
                        refresh_user_list(
                            &users_list,
                            &users_title,
                            &current_buf.borrow(),
                            &users.borrow(),
                        );
                    }
                    UiEvent::RemoveUserEverywhere { conn_id, nick } => {
                        for (k, set) in users.borrow_mut().iter_mut() {
                            if k.starts_with(&format!("{conn_id}|")) {
                                set.remove(&nick);
                            }
                        }
                        refresh_user_list(
                            &users_list,
                            &users_title,
                            &current_buf.borrow(),
                            &users.borrow(),
                        );
                    }
                    UiEvent::RenameUserEverywhere { conn_id, old, new_ } => {
                        for (k, set) in users.borrow_mut().iter_mut() {
                            if k.starts_with(&format!("{conn_id}|")) {
                                let old_disp = crate::util::nick_display(&old).to_string();
                                let mut removed = false;
                                set.retain(|x| {
                                    let keep = crate::util::nick_display(x) != old_disp;
                                    if !keep {
                                        removed = true;
                                    }
                                    keep
                                });
                                if removed {
                                    set.insert(new_.clone());
                                }
                            }
                        }
                        refresh_user_list(
                            &users_list,
                            &users_title,
                            &current_buf.borrow(),
                            &users.borrow(),
                        );
                    }
                }
            }

            glib::ControlFlow::Continue
        });
    }

    // Initial sidebar rebuild.
    rebuild_sidebar(&sidebar_items, &sidebar_list);

    // Initial user list.
    refresh_user_list(&users_list, &users_title, &current_buf.borrow(), &users.borrow());

    win.present();
}

fn chan_users_key(conn_id: u64, channel: &str) -> String {
    format!("{conn_id}|{channel}")
}

fn display_for_buffer(key: &BufferKey, meta: &HashMap<u64, ConnMeta>) -> String {
    if key.conn_id == 0 {
        return "Status".to_string();
    }
    let server = meta
        .get(&key.conn_id)
        .map(|m| m.server.as_str())
        .unwrap_or("server");
    if key.name == "Status" {
        format!("Status @ {server}")
    } else {
        format!("{} @ {server}", key.name)
    }
}

fn ensure_buffer(
    key: BufferKey,
    display: String,
    make_current: bool,
    notebook: &Notebook,
    tabs: &Rc<RefCell<HashMap<String, Tab>>>,
    page_to_buf: &Rc<RefCell<HashMap<u32, String>>>,
    unread: &Rc<RefCell<HashMap<String, u32>>>,
    highlights: &Rc<RefCell<HashMap<String, u32>>>,
) {
    let k = key.as_string();
    if tabs.borrow().contains_key(&k) {
        if make_current {
            notebook.set_current_page(Some(tabs.borrow()[&k].page_num));
        }
        return;
    }

    let buf = TextBuffer::new(None);
    let view = TextView::with_buffer(&buf);
    view.set_editable(false);
    view.set_cursor_visible(false);
    view.set_monospace(true);
    view.set_wrap_mode(WrapMode::WordChar);
    view.set_vexpand(true);
    view.set_hexpand(true);

    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&view)
        .build();

    let tab_label = Label::new(Some(&display));
    tab_label.set_xalign(0.0);

    let page_num = notebook.append_page(&scrolled, Some(&tab_label));

    let tab = Tab {
        page_num,
        view,
        buffer: buf,
        tab_label,
        base_label: display,
    };

    tabs.borrow_mut().insert(k.clone(), tab);
    page_to_buf.borrow_mut().insert(page_num, k.clone());

    unread.borrow_mut().entry(k.clone()).or_insert(0);
    highlights.borrow_mut().entry(k.clone()).or_insert(0);

    update_tab_label(&key, tabs, unread, highlights);

    if make_current {
        notebook.set_current_page(Some(page_num));
        unread.borrow_mut().insert(k.clone(), 0);
        highlights.borrow_mut().insert(k.clone(), 0);
        update_tab_label(&key, tabs, unread, highlights);
    }
}

fn update_tab_label(
    key: &BufferKey,
    tabs: &Rc<RefCell<HashMap<String, Tab>>>,
    unread: &Rc<RefCell<HashMap<String, u32>>>,
    highlights: &Rc<RefCell<HashMap<String, u32>>>,
) {
    let k = key.as_string();
    let u = unread.borrow().get(&k).copied().unwrap_or(0);
    let h = highlights.borrow().get(&k).copied().unwrap_or(0);

    if let Some(tab) = tabs.borrow().get(&k) {
        let base = tab.base_label.clone();
        let label = if h > 0 && u > 0 {
            format!("{base} (!{h}, {u})")
        } else if h > 0 {
            format!("{base} (!{h})")
        } else if u > 0 {
            format!("{base} ({u})")
        } else {
            base
        };
        tab.tab_label.set_text(&label);
    }
}

fn select_buffer(key: &BufferKey, notebook: &Notebook, tabs: &Rc<RefCell<HashMap<String, Tab>>>) {
    if let Some(tab) = tabs.borrow().get(&key.as_string()) {
        notebook.set_current_page(Some(tab.page_num));
    }
}

fn append_to(key: &BufferKey, line: &str, tabs: &Rc<RefCell<HashMap<String, Tab>>>) {
    let k = key.as_string();
    let Some(tab) = tabs.borrow().get(&k).cloned() else { return };

    let mut end = tab.buffer.end_iter();
    tab.buffer.insert(&mut end, line);
    tab.buffer.insert(&mut end, "\n");

    let mut end2 = tab.buffer.end_iter();
    tab.view.scroll_to_iter(&mut end2, 0.0, false, 0.0, 0.0);
}

fn handle_slash_command(
    tx: &mpsc::Sender<BackendCmd>,
    current: &BufferKey,
    cmdline: &str,
    conn_meta: &Rc<RefCell<HashMap<u64, ConnMeta>>>,
    notebook: &Notebook,
    tabs: &Rc<RefCell<HashMap<String, Tab>>>,
    page_to_buf: &Rc<RefCell<HashMap<u32, String>>>,
    unread: &Rc<RefCell<HashMap<String, u32>>>,
    highlights: &Rc<RefCell<HashMap<String, u32>>>,
    sidebar_items: &Rc<RefCell<Vec<SidebarItemKind>>>,
    sidebar_list: &ListBox,
) {
    let mut parts = cmdline.splitn(3, ' ');
    let cmd = parts.next().unwrap_or("").to_lowercase();
    let a1 = parts.next().unwrap_or("").trim().to_string();
    let a2 = parts.next().unwrap_or("").trim().to_string();

    match cmd.as_str() {
        "join" => {
            if a1.is_empty() {
                append_to(current, &format!("{} *** Usage: /join #chan", ts_prefix()), tabs);
                return;
            }
            let _ = tx.send(BackendCmd::SendRaw {
                conn_id: current.conn_id,
                line: format!("JOIN {a1}"),
            });
            let _ = tx.send(BackendCmd::SendRaw {
                conn_id: current.conn_id,
                line: format!("NAMES {a1}"),
            });

            let key = BufferKey::new(current.conn_id, a1.clone());
            let display = display_for_buffer(&key, &conn_meta.borrow());
            ensure_buffer(
                key.clone(),
                display.clone(),
                true,
                notebook,
                tabs,
                page_to_buf,
                unread,
                highlights,
            );
            add_sidebar_buffer(&key, &display, sidebar_items, sidebar_list, &conn_meta.borrow());
            select_buffer(&key, notebook, tabs);
        }
        "msg" => {
            if a1.is_empty() || a2.is_empty() {
                append_to(current, &format!("{} *** Usage: /msg nick text", ts_prefix()), tabs);
                return;
            }
            let _ = tx.send(BackendCmd::SendRaw {
                conn_id: current.conn_id,
                line: format!("PRIVMSG {a1} :{a2}"),
            });

            let key = BufferKey::new(current.conn_id, a1.clone());
            let display = display_for_buffer(&key, &conn_meta.borrow());
            ensure_buffer(
                key.clone(),
                display.clone(),
                true,
                notebook,
                tabs,
                page_to_buf,
                unread,
                highlights,
            );
            add_sidebar_buffer(&key, &display, sidebar_items, sidebar_list, &conn_meta.borrow());
            select_buffer(&key, notebook, tabs);

            let nick = conn_meta
                .borrow()
                .get(&current.conn_id)
                .map(|m| m.nick.clone())
                .unwrap_or_else(|| "me".to_string());
            append_to(&key, &format!("{} <{}> {}", ts_prefix(), nick, a2), tabs);
        }
        "me" => {
            let action = format!("{a1} {a2}").trim().to_string();
            if action.is_empty() {
                return;
            }
            let target = if current.name == "Status" {
                conn_meta
                    .borrow()
                    .get(&current.conn_id)
                    .map(|m| m.default_target.clone())
                    .unwrap_or_else(|| "#hexrust".to_string())
            } else {
                current.name.clone()
            };
            let ctcp = format!("\u{0001}ACTION {action}\u{0001}");
            let _ = tx.send(BackendCmd::SendRaw {
                conn_id: current.conn_id,
                line: format!("PRIVMSG {target} :{ctcp}"),
            });

            let nick = conn_meta
                .borrow()
                .get(&current.conn_id)
                .map(|m| m.nick.clone())
                .unwrap_or_else(|| "me".to_string());
            append_to(
                &BufferKey::new(current.conn_id, target),
                &format!("{} * {} {}", ts_prefix(), nick, action),
                tabs,
            );
        }
        "nick" => {
            if a1.is_empty() {
                return;
            }
            let _ = tx.send(BackendCmd::SendRaw {
                conn_id: current.conn_id,
                line: format!("NICK {a1}"),
            });
            if let Some(m) = conn_meta.borrow_mut().get_mut(&current.conn_id) {
                m.nick = a1.clone();
            }
            append_to(current, &format!("{} *** NICK {a1}", ts_prefix()), tabs);
        }
        "raw" => {
            let raw = format!("{a1} {a2}").trim().to_string();
            if raw.is_empty() {
                return;
            }
            let _ = tx.send(BackendCmd::SendRaw {
                conn_id: current.conn_id,
                line: raw.clone(),
            });
            append_to(current, &format!("{} => {raw}", ts_prefix()), tabs);
        }
        "switch" => {
            if a1.is_empty() {
                append_to(current, &format!("{} *** Usage: /switch <buffer>", ts_prefix()), tabs);
                return;
            }
            let query = a1.to_lowercase();
            let mut best: Option<BufferKey> = None;
            for k in tabs.borrow().keys() {
                if let Some(bk) = BufferKey::from_string(k) {
                    let s = format!("{} {}", bk.conn_id, bk.name).to_lowercase();
                    if s.contains(&query) {
                        best = Some(bk);
                        break;
                    }
                }
            }
            if let Some(bk) = best {
                select_buffer(&bk, notebook, tabs);
            } else {
                append_to(current, &format!("{} *** Not found: {a1}", ts_prefix()), tabs);
            }
        }
        "server" => {
            if a1.is_empty() {
                append_to(current, &format!("{} *** Usage: /server <id> <raw>", ts_prefix()), tabs);
                return;
            }
            let Ok(id) = a1.parse::<u64>() else {
                append_to(current, &format!("{} *** Invalid server id: {a1}", ts_prefix()), tabs);
                return;
            };
            let raw = a2.trim().to_string();
            if raw.is_empty() {
                append_to(current, &format!("{} *** Usage: /server <id> <raw>", ts_prefix()), tabs);
                return;
            }
            let _ = tx.send(BackendCmd::SendRaw { conn_id: id, line: raw.clone() });
            append_to(current, &format!("{} => (server {id}) {raw}", ts_prefix()), tabs);
        }
        _ => {
            let raw = cmdline.trim().to_string();
            let _ = tx.send(BackendCmd::SendRaw {
                conn_id: current.conn_id,
                line: raw.clone(),
            });
            append_to(current, &format!("{} => {raw}", ts_prefix()), tabs);
        }
    }
}

fn create_command_palette(
    parent: &ApplicationWindow,
    backend_tx: &Rc<RefCell<Option<mpsc::Sender<BackendCmd>>>>,
    next_conn_id: &Rc<RefCell<u64>>,
    conn_meta: &Rc<RefCell<HashMap<u64, ConnMeta>>>,
    notebook: &Notebook,
    tabs: &Rc<RefCell<HashMap<String, Tab>>>,
    page_to_buf: &Rc<RefCell<HashMap<u32, String>>>,
    unread: &Rc<RefCell<HashMap<String, u32>>>,
    highlights: &Rc<RefCell<HashMap<String, u32>>>,
    sidebar_items: &Rc<RefCell<Vec<SidebarItemKind>>>,
    sidebar_list: &ListBox,
    current_buf: &Rc<RefCell<BufferKey>>,
) -> ApplicationWindow {
    let dialog = ApplicationWindow::builder()
        .transient_for(parent)
        .modal(true)
        .title("Command Palette")
        .default_width(680)
        .default_height(140)
        .build();

    let root = gtk::Box::new(Orientation::Vertical, 8);
    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);

    let hint = Label::new(Some("Examples:  connect irc.libera.chat 6697 nick #rust   |   join #rust   |   switch #chan   |   raw WHOIS nick"));
    hint.set_xalign(0.0);

    let entry = Entry::builder()
        .placeholder_text("Command…")
        .hexpand(true)
        .build();

    root.append(&hint);
    root.append(&entry);

    dialog.set_child(Some(&root));

    {
        let dialog = dialog.clone();
        let backend_tx = backend_tx.clone();
        let next_conn_id = next_conn_id.clone();
        let conn_meta = conn_meta.clone();

        let notebook = notebook.clone();
        let tabs = tabs.clone();
        let page_to_buf = page_to_buf.clone();
        let unread = unread.clone();
        let highlights = highlights.clone();

        let sidebar_items = sidebar_items.clone();
        let sidebar_list = sidebar_list.clone();

        let current_buf = current_buf.clone();

        entry.connect_activate(move |e| {
            let input = e.text().trim().to_string();
            e.set_text("");
            if input.is_empty() {
                return;
            }

            let Some(tx) = backend_tx.borrow().clone() else {
                return;
            };

            let mut parts = input.split_whitespace();
            let cmd = parts.next().unwrap_or("").to_lowercase();

            match cmd.as_str() {
                "connect" => {
                    let server = parts.next().unwrap_or("").to_string();
                    let port: u16 = parts.next().unwrap_or("6697").parse().unwrap_or(6697);
                    let nick = parts.next().unwrap_or("hexrust").to_string();
                    let chan = parts.next().unwrap_or("#hexrust").to_string();

                    if server.is_empty() {
                        append_to(&BufferKey::new(0, "Status"), &format!("{} *** palette: missing server", ts_prefix()), &tabs);
                        return;
                    }

                    let conn_id = {
                        let mut n = next_conn_id.borrow_mut();
                        let id = *n;
                        *n += 1;
                        id
                    };

                    conn_meta.borrow_mut().insert(conn_id, ConnMeta {
                        server: server.clone(),
                        nick: nick.clone(),
                        default_target: chan.clone(),
                    });

                    add_sidebar_connection(conn_id, &server, &sidebar_items, &sidebar_list);

                    let status_key = BufferKey::new(conn_id, "Status");
                    let status_disp = display_for_buffer(&status_key, &conn_meta.borrow());
                    ensure_buffer(status_key.clone(), status_disp.clone(), false, &notebook, &tabs, &page_to_buf, &unread, &highlights);
                    add_sidebar_buffer(&status_key, &status_disp, &sidebar_items, &sidebar_list, &conn_meta.borrow());

                    let chan_key = BufferKey::new(conn_id, chan.clone());
                    let chan_disp = display_for_buffer(&chan_key, &conn_meta.borrow());
                    ensure_buffer(chan_key.clone(), chan_disp.clone(), true, &notebook, &tabs, &page_to_buf, &unread, &highlights);
                    add_sidebar_buffer(&chan_key, &chan_disp, &sidebar_items, &sidebar_list, &conn_meta.borrow());

                    *current_buf.borrow_mut() = chan_key.clone();
                    select_buffer(&chan_key, &notebook, &tabs);

<<<<<<< HEAD
                    let cfg = crate::model::IrcConfig {
                        server,
                        port,
                        tls: true,
                        nick,
                        initial_channel: chan,
                    };
=======
                    let cfg = crate::model::IrcConfig { server, port, tls: true, nick, initial_channel: chan, server_password: None, sasl_username: None, sasl_password: None };
>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)
                    let _ = tx.send(BackendCmd::Connect { conn_id, cfg });

                    dialog.close();
                }
                "join" => {
                    let chan = parts.next().unwrap_or("").to_string();
                    if chan.is_empty() {
                        return;
                    }
                    let key = current_buf.borrow().clone();
                    let _ = tx.send(BackendCmd::SendRaw {
                        conn_id: key.conn_id,
                        line: format!("JOIN {chan}"),
                    });
                    let _ = tx.send(BackendCmd::SendRaw {
                        conn_id: key.conn_id,
                        line: format!("NAMES {chan}"),
                    });
                    dialog.close();
                }
                "switch" => {
                    let query = parts.next().unwrap_or("").to_string();
                    if query.is_empty() {
                        return;
                    }
                    let q = query.to_lowercase();
                    let mut best: Option<BufferKey> = None;
                    for k in tabs.borrow().keys() {
                        if let Some(bk) = BufferKey::from_string(k) {
                            let s = format!("{} {}", bk.conn_id, bk.name).to_lowercase();
                            if s.contains(&q) {
                                best = Some(bk);
                                break;
                            }
                        }
                    }
                    if let Some(bk) = best {
                        select_buffer(&bk, &notebook, &tabs);
                        *current_buf.borrow_mut() = bk;
                    }
                    dialog.close();
                }
                "raw" => {
                    let key = current_buf.borrow().clone();
                    let rest = input.strip_prefix("raw").unwrap_or("").trim().to_string();
                    if !rest.is_empty() {
                        let _ = tx.send(BackendCmd::SendRaw {
                            conn_id: key.conn_id,
                            line: rest,
                        });
                    }
                    dialog.close();
                }
                _ => {
                    let key = current_buf.borrow().clone();
                    let _ = tx.send(BackendCmd::SendRaw {
                        conn_id: key.conn_id,
                        line: input,
                    });
                    dialog.close();
                }
            }
        });
    }

    dialog
}

fn add_sidebar_connection(
    conn_id: u64,
    server: &str,
    sidebar_items: &Rc<RefCell<Vec<SidebarItemKind>>>,
    sidebar_list: &ListBox,
) {
    sidebar_items.borrow_mut().push(SidebarItemKind::Header {
        conn_id,
        title: format!("[{conn_id}] {server}"),
    });
    rebuild_sidebar(sidebar_items, sidebar_list);
}

fn add_sidebar_buffer(
    key: &BufferKey,
    display: &str,
    sidebar_items: &Rc<RefCell<Vec<SidebarItemKind>>>,
    sidebar_list: &ListBox,
    _conn_meta: &HashMap<u64, ConnMeta>,
) {
    let exists = sidebar_items.borrow().iter().any(|it| match it {
        SidebarItemKind::Buffer { key: k, .. } => k == key,
        _ => false,
    });
    if exists {
        return;
    }

    let indent = if key.name == "Status" { 1 } else { 2 };
    sidebar_items.borrow_mut().push(SidebarItemKind::Buffer {
        key: key.clone(),
        display: display.to_string(),
        indent,
    });

    rebuild_sidebar(sidebar_items, sidebar_list);
}

fn rebuild_sidebar(sidebar_items: &Rc<RefCell<Vec<SidebarItemKind>>>, sidebar_list: &ListBox) {
    while let Some(child) = sidebar_list.first_child() {
        sidebar_list.remove(&child);
    }

    let mut items = sidebar_items.borrow().clone();
    items.sort_by(|a, b| match (a, b) {
        (
            SidebarItemKind::Header { conn_id: ca, .. },
            SidebarItemKind::Header { conn_id: cb, .. },
        ) => ca.cmp(cb),
        (SidebarItemKind::Header { .. }, SidebarItemKind::Buffer { .. }) => {
            std::cmp::Ordering::Less
        }
        (SidebarItemKind::Buffer { .. }, SidebarItemKind::Header { .. }) => {
            std::cmp::Ordering::Greater
        }
        (SidebarItemKind::Buffer { key: ka, .. }, SidebarItemKind::Buffer { key: kb, .. }) => {
            (ka.conn_id, &ka.name).cmp(&(kb.conn_id, &kb.name))
        }
    });

    for it in items.iter() {
        match it {
            SidebarItemKind::Header { title, .. } => {
                let row = ListBoxRow::new();
                let label = Label::new(Some(title));
                label.set_xalign(0.0);
                label.set_opacity(0.85);
                row.set_child(Some(&label));
                sidebar_list.append(&row);
            }
            SidebarItemKind::Buffer {
                key,
                display,
                indent,
            } => {
                let row = ListBoxRow::new();
                let pad = "  ".repeat(*indent as usize);
                let label = Label::new(Some(&format!("{pad}{display}")));
                label.set_xalign(0.0);
                row.set_child(Some(&label));

                // Store the BufferKey in a safe place: widget_name.
                row.set_widget_name(&key.as_string());

                sidebar_list.append(&row);
            }
        }
    }
}

fn refresh_user_list(
    users_list: &ListBox,
    users_title: &Label,
    active: &BufferKey,
    users: &HashMap<String, BTreeSet<String>>,
) {
    while let Some(child) = users_list.first_child() {
        users_list.remove(&child);
    }

    if active.conn_id == 0 || !active.name.starts_with('#') {
        users_title.set_text("Users");
        users_list.append(&placeholder_row("Select a channel tab to show users"));
        return;
    }

    let key = chan_users_key(active.conn_id, &active.name);
    let set_opt = users.get(&key);

    let count = set_opt.map(|s| s.len()).unwrap_or(0);
    users_title.set_text(&format!("Users [{}] {}", count, active.name));

    let Some(set) = set_opt else {
        users_list.append(&placeholder_row("Waiting for NAMES (353)…"));
        return;
    };

    if set.is_empty() {
        users_list.append(&placeholder_row("No users (yet)"));
        return;
    }

    // Sort: by prefix rank (~ & @ % +), then alphabetically (case-insensitive) on display nick.
    let mut vec: Vec<String> = set.iter().cloned().collect();
    vec.sort_by(|a, b| {
        let ra = crate::util::nick_rank(a);
        let rb = crate::util::nick_rank(b);
        if ra != rb {
            return rb.cmp(&ra); // higher rank first
        }
        let da = crate::util::nick_display(a).to_lowercase();
        let db = crate::util::nick_display(b).to_lowercase();
        da.cmp(&db)
    });

    for nick in vec.iter() {
        let row = ListBoxRow::new();
        // Display without prefix, but keep it visible as a leading char for context.
        let prefix = nick.chars().next().unwrap_or(' ');
        let disp = crate::util::nick_display(nick);
        let shown = if matches!(prefix, '~' | '&' | '@' | '%' | '+') {
            format!("{prefix}{disp}")
        } else {
            disp.to_string()
        };

        let label = Label::new(Some(&shown));
        label.set_xalign(0.0);
        row.set_child(Some(&label));
        users_list.append(&row);
    }
}

fn placeholder_row(text: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    let label = Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_opacity(0.75);
    row.set_child(Some(&label));
    row
}

fn history_push(hist: &Rc<RefCell<History>>, item: String) {
    let mut h = hist.borrow_mut();
    if h.items.last().map(|s| s == &item).unwrap_or(false) {
        h.pos = h.items.len();
        h.scratch.clear();
        return;
    }
    h.items.push(item);
    h.pos = h.items.len();
    h.scratch.clear();
}

fn history_up(entry: &Entry, hist: &Rc<RefCell<History>>) {
    let mut h = hist.borrow_mut();
    if h.items.is_empty() {
        return;
    }
    if h.pos == h.items.len() {
        h.scratch = entry.text().to_string();
    }
    if h.pos > 0 {
        h.pos -= 1;
        entry.set_text(&h.items[h.pos]);
        entry.set_position(-1);
    }
}

fn history_down(entry: &Entry, hist: &Rc<RefCell<History>>) {
    let mut h = hist.borrow_mut();
    if h.items.is_empty() {
        return;
    }
    if h.pos < h.items.len() {
        h.pos += 1;
        if h.pos == h.items.len() {
            entry.set_text(&h.scratch);
        } else {
            entry.set_text(&h.items[h.pos]);
        }
        entry.set_position(-1);
    }
}
<<<<<<< HEAD
=======




fn create_profiles_manager(
    parent: &ApplicationWindow,
    profiles_path: std::path::PathBuf,
    backend_tx: &Rc<RefCell<Option<mpsc::Sender<BackendCmd>>>>,
    next_conn_id: &Rc<RefCell<u64>>,
    conn_meta: &Rc<RefCell<HashMap<u64, ConnMeta>>>,
    notebook: &Notebook,
    tabs: &Rc<RefCell<HashMap<String, Tab>>>,
    page_to_buf: &Rc<RefCell<HashMap<u32, String>>>,
    unread: &Rc<RefCell<HashMap<String, u32>>>,
    highlights: &Rc<RefCell<HashMap<String, u32>>>,
    sidebar_items: &Rc<RefCell<Vec<SidebarItemKind>>>,
    sidebar_list: &ListBox,
    current_buf: &Rc<RefCell<BufferKey>>,
) -> ApplicationWindow {
    // Separate Regular vs ZNC profiles and fix "Save" by showing errors instead of silently ignoring them.
    let win = ApplicationWindow::builder()
        .transient_for(parent)
        .modal(true)
        .title("Profiles")
        .default_width(820)
        .default_height(560)
        .build();

    let profiles_path = Rc::new(profiles_path);

    let root = gtk::Box::new(Orientation::Vertical, 10);
    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);

    let header = Label::new(Some(&format!(
        "Profiles file: {}\nRegular and ZNC profiles are shown separately. ZNC uses IRC PASS: username/network:password.",
        profiles_path.display()
    )));
    header.set_xalign(0.0);

    let tabs_nb = Notebook::new();
    tabs_nb.set_hexpand(true);
    tabs_nb.set_vexpand(true);

    let list_regular = ListBox::new();
    list_regular.set_vexpand(true);

    let list_znc = ListBox::new();
    list_znc.set_vexpand(true);

    let scroll_regular = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&list_regular)
        .build();

    let scroll_znc = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&list_znc)
        .build();

    tabs_nb.append_page(&scroll_regular, Some(&Label::new(Some("Regular"))));
    tabs_nb.append_page(&scroll_znc, Some(&Label::new(Some("ZNC"))));

    // Bottom status label (for save/load errors).
    let status = Label::new(Some(""));
    status.set_xalign(0.0);

    // Buttons
    let buttons = gtk::Box::new(Orientation::Horizontal, 10);
    let refresh_btn = Button::with_label("Refresh");
    let add_regular_btn = Button::with_label("Add Regular");
    let add_znc_btn = Button::with_label("Add ZNC");
    let edit_btn = Button::with_label("Edit");
    let del_btn = Button::with_label("Delete");
    let connect_btn = Button::with_label("Connect");

    buttons.append(&refresh_btn);
    buttons.append(&add_regular_btn);
    buttons.append(&add_znc_btn);
    buttons.append(&edit_btn);
    buttons.append(&del_btn);
    buttons.append(&connect_btn);

    root.append(&header);
    root.append(&tabs_nb);
    root.append(&buttons);
    root.append(&status);

    win.set_child(Some(&root));

    // Helper: clear listbox.
    let clear_list = |list: &ListBox| {
        loop {
            let Some(child) = list.first_child() else { break };
            list.remove(&child);
        }
    };

    // Populate both lists from file.
    let populate: Rc<dyn Fn()> = {
        let list_regular = list_regular.clone();
        let list_znc = list_znc.clone();
        let profiles_path = profiles_path.clone();
        let status = status.clone();

        Rc::new(move || {
            clear_list(&list_regular);
            clear_list(&list_znc);

            let pf = match crate::profiles::load(&profiles_path) {
                Ok(pf) => pf,
                Err(e) => {
                    status.set_text(&format!("Load failed: {e}"));
                    return;
                }
            };

            status.set_text("");

            for p in pf.profiles.iter() {
                let is_znc = p.znc_username.is_some() && p.znc_password.is_some();

                let row = ListBoxRow::new();
                row.set_widget_name(&p.name);

                let text = if is_znc {
                    format!(
                        "{}  →  {}:{}  {}  nick={}  chan={}  znc_user={}  net={}",
                        p.name,
                        p.server,
                        p.port,
                        if p.tls { "TLS" } else { "PLAINTEXT" },
                        p.nick,
                        p.initial_channel,
                        p.znc_username.clone().unwrap_or_default(),
                        p.znc_network.clone().unwrap_or_default(),
                    )
                } else {
                    format!(
                        "{}  →  {}:{}  {}  nick={}  chan={}  sasl={}",
                        p.name,
                        p.server,
                        p.port,
                        if p.tls { "TLS" } else { "PLAINTEXT" },
                        p.nick,
                        p.initial_channel,
                        if p.sasl_username.is_some() { "yes" } else { "no" }
                    )
                };

                let label = Label::new(Some(&text));
                label.set_xalign(0.0);
                row.set_child(Some(&label));

                if is_znc {
                    list_znc.append(&row);
                } else {
                    list_regular.append(&row);
                }
            }
        })
    };

    // Initial populate.
    (populate)();

    // Refresh
    {
        let populate = populate.clone();
        refresh_btn.connect_clicked(move |_| (populate)());
    }

    // Editor dialog (shared for both Regular and ZNC).
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum EditorKind {
        Regular,
        Znc,
    }

    let open_editor: Rc<dyn Fn(EditorKind, Option<crate::profiles::Profile>)> = {
        let parent2 = win.clone();
        let profiles_path = profiles_path.clone();
        let populate = populate.clone();
        let status_global = status.clone();

        Rc::new(move |kind: EditorKind, existing: Option<crate::profiles::Profile>| {
            let is_edit = existing.is_some();
            let dialog = ApplicationWindow::builder()
                .transient_for(&parent2)
                .modal(true)
                .title(if is_edit { "Edit Profile" } else { "Add Profile" })
                .default_width(620)
                .default_height(620)
                .build();

            let root = gtk::Box::new(Orientation::Vertical, 10);
            root.set_margin_top(12);
            root.set_margin_bottom(12);
            root.set_margin_start(12);
            root.set_margin_end(12);

            let hint = Label::new(Some(
                "Regular profiles: normal server connection.\nZNC profiles: fill ZNC fields and the client will send PASS username/network:password.",
            ));
            hint.set_xalign(0.0);

            // SERVER section
            let server_frame = Frame::new(Some("Server"));
            let server_box = gtk::Box::new(Orientation::Vertical, 8);
            server_box.set_margin_top(8);
            server_box.set_margin_bottom(8);
            server_box.set_margin_start(8);
            server_box.set_margin_end(8);

            let name = Entry::builder().placeholder_text("Profile name").build();
            let server = Entry::builder().placeholder_text("Server host").build();
            let port = Entry::builder().placeholder_text("Port").text("6697").build();
            let tls = Entry::builder().placeholder_text("TLS (true/false)").text("true").build();
            let nick = Entry::builder().placeholder_text("Nick").build();
            let chan = Entry::builder().placeholder_text("Initial channel").build();

            server_box.append(&Label::new(Some("Name")));
            server_box.append(&name);
            server_box.append(&Label::new(Some("Server")));
            server_box.append(&server);
            server_box.append(&Label::new(Some("Port")));
            server_box.append(&port);
            server_box.append(&Label::new(Some("TLS")));
            server_box.append(&tls);
            server_box.append(&Label::new(Some("Nick")));
            server_box.append(&nick);
            server_box.append(&Label::new(Some("Initial channel")));
            server_box.append(&chan);

            server_frame.set_child(Some(&server_box));

            // ZNC section (separate)
            let znc_frame = Frame::new(Some("ZNC (optional)"));
            let znc_box = gtk::Box::new(Orientation::Vertical, 8);
            znc_box.set_margin_top(8);
            znc_box.set_margin_bottom(8);
            znc_box.set_margin_start(8);
            znc_box.set_margin_end(8);

            let znc_user = Entry::builder().placeholder_text("ZNC username").build();
            let znc_net = Entry::builder().placeholder_text("ZNC network (optional)").build();
            let znc_pass = Entry::builder().placeholder_text("ZNC server password").build();

            znc_box.append(&Label::new(Some("ZNC username")));
            znc_box.append(&znc_user);
            znc_box.append(&Label::new(Some("ZNC network (optional)")));
            znc_box.append(&znc_net);
            znc_box.append(&Label::new(Some("ZNC server password")));
            znc_box.append(&znc_pass);

            let znc_note = Label::new(Some(
                "PASS format sent to server:\n  username:password\n  username/network:password",
            ));
            znc_note.set_xalign(0.0);
            znc_box.append(&znc_note);

            znc_frame.set_child(Some(&znc_box));

            // SASL section (optional)
            let sasl_frame = Frame::new(Some("SASL (optional)"));
            let sasl_box = gtk::Box::new(Orientation::Vertical, 8);
            sasl_box.set_margin_top(8);
            sasl_box.set_margin_bottom(8);
            sasl_box.set_margin_start(8);
            sasl_box.set_margin_end(8);

            let sasl_user = Entry::builder().placeholder_text("SASL username").build();
            let sasl_pass = Entry::builder().placeholder_text("SASL password").build();

            sasl_box.append(&Label::new(Some("SASL username")));
            sasl_box.append(&sasl_user);
            sasl_box.append(&Label::new(Some("SASL password")));
            sasl_box.append(&sasl_pass);

            sasl_frame.set_child(Some(&sasl_box));

            // Dialog status label
            let dialog_status = Label::new(Some(""));
            dialog_status.set_xalign(0.0);

            // Buttons
            let btn_row = gtk::Box::new(Orientation::Horizontal, 10);
            let save_btn = Button::with_label("Save");
            let cancel_btn = Button::with_label("Cancel");
            btn_row.append(&save_btn);
            btn_row.append(&cancel_btn);

            // Prefill from existing or defaults
            if let Some(p) = existing.clone() {
                name.set_text(&p.name);
                server.set_text(&p.server);
                port.set_text(&p.port.to_string());
                tls.set_text(if p.tls { "true" } else { "false" });
                nick.set_text(&p.nick);
                chan.set_text(&p.initial_channel);

                if let Some(u) = p.znc_username {
                    znc_user.set_text(&u);
                }
                if let Some(n) = p.znc_network {
                    znc_net.set_text(&n);
                }
                if let Some(pw) = p.znc_password {
                    znc_pass.set_text(&pw);
                }

                if let Some(u) = p.sasl_username {
                    sasl_user.set_text(&u);
                }
                if let Some(pw) = p.sasl_password {
                    sasl_pass.set_text(&pw);
                }
            } else if kind == EditorKind::Znc {
                // Small hint defaults for ZNC adds.
                port.set_text("6697");
                tls.set_text("true");
            }

            // Build UI
            root.append(&hint);
            root.append(&server_frame);
            root.append(&znc_frame);
            root.append(&sasl_frame);
            root.append(&btn_row);
            root.append(&dialog_status);

            dialog.set_child(Some(&root));

            cancel_btn.connect_clicked({
                let dialog = dialog.clone();
                move |_| dialog.close()
            });

            save_btn.connect_clicked({
                let dialog = dialog.clone();
                let profiles_path = profiles_path.clone();
                let populate = populate.clone();
                let dialog_status = dialog_status.clone();
                let status_global = status_global.clone();

                let name = name.clone();
                let server = server.clone();
                let port = port.clone();
                let tls = tls.clone();
                let nick = nick.clone();
                let chan = chan.clone();

                let znc_user = znc_user.clone();
                let znc_net = znc_net.clone();
                let znc_pass = znc_pass.clone();

                let sasl_user = sasl_user.clone();
                let sasl_pass = sasl_pass.clone();

                move |_| {
                    dialog_status.set_text("");

                    let name_v = name.text().trim().to_string();
                    let server_v = server.text().trim().to_string();
                    let port_v: u16 = port.text().trim().parse().unwrap_or(6697);
                    let tls_v = tls.text().trim().eq_ignore_ascii_case("true");
                    let nick_v = nick.text().trim().to_string();
                    let chan_v = chan.text().trim().to_string();

                    if name_v.is_empty() || server_v.is_empty() || nick_v.is_empty() || chan_v.is_empty() {
                        dialog_status.set_text("Missing required fields: name, server, nick, initial channel.");
                        return;
                    }

                    let znc_u = znc_user.text().trim().to_string();
                    let znc_n = znc_net.text().trim().to_string();
                    let znc_p = znc_pass.text().trim().to_string();

                    let znc_username = if znc_u.is_empty() { None } else { Some(znc_u) };
                    let znc_network = if znc_n.is_empty() { None } else { Some(znc_n) };
                    let znc_password = if znc_p.is_empty() { None } else { Some(znc_p) };

                    let sasl_u = sasl_user.text().trim().to_string();
                    let sasl_p = sasl_pass.text().trim().to_string();
                    let sasl_username = if sasl_u.is_empty() { None } else { Some(sasl_u) };
                    let sasl_password = if sasl_p.is_empty() { None } else { Some(sasl_p) };

                    let new_prof = crate::profiles::Profile {
                        name: name_v.clone(),
                        server: server_v,
                        port: port_v,
                        tls: tls_v,
                        nick: nick_v,
                        initial_channel: chan_v,
                        znc_username,
                        znc_network,
                        znc_password,
                        sasl_username,
                        sasl_password,
                    };

                    let mut pf = match crate::profiles::load(&profiles_path) {
                        Ok(pf) => pf,
                        Err(e) => {
                            dialog_status.set_text(&format!("Load failed: {e}"));
                            return;
                        }
                    };

                    // Upsert by name.
                    if let Some(existing) = pf.profiles.iter_mut().find(|p| p.name == name_v) {
                        *existing = new_prof;
                    } else {
                        pf.profiles.push(new_prof);
                    }

                    pf.profiles
                        .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                    if let Err(e) = crate::profiles::save(&profiles_path, &pf) {
                        dialog_status.set_text(&format!("Save failed: {e}"));
                        status_global.set_text(&format!("Save failed: {e}"));
                        return;
                    }

                    status_global.set_text("Saved profiles.toml");
                    (populate)();
                    dialog.close();
                }
            });

            dialog.present();
        })
    };

    // Add Regular
    {
        let open_editor = open_editor.clone();
        add_regular_btn.connect_clicked(move |_| (open_editor)(EditorKind::Regular, None));
    }

    // Add ZNC
    {
        let open_editor = open_editor.clone();
        add_znc_btn.connect_clicked(move |_| (open_editor)(EditorKind::Znc, None));
    }

    // Helper: get currently selected profile name depending on active tab.
    let get_selected_name: Rc<dyn Fn() -> Option<String>> = {
        let tabs_nb = tabs_nb.clone();
        let list_regular = list_regular.clone();
        let list_znc = list_znc.clone();

        Rc::new(move || {
            let page = tabs_nb.current_page().unwrap_or(0);
            let list = if page == 1 { &list_znc } else { &list_regular };
            let row = list.selected_row()?;
            let name = row.widget_name().to_string();
            if name.is_empty() { None } else { Some(name) }
        })
    };

    // Edit selected
    {
        let profiles_path = profiles_path.clone();
        let get_selected_name = get_selected_name.clone();
        let open_editor = open_editor.clone();
        let tabs_nb = tabs_nb.clone();

        edit_btn.connect_clicked(move |_| {
            let Some(name) = (get_selected_name)() else { return };

            let pf = match crate::profiles::load(&profiles_path) {
                Ok(pf) => pf,
                Err(_) => return,
            };

            let Some(p) = pf.profiles.into_iter().find(|p| p.name == name) else { return };

            // Choose editor kind based on current tab.
            let page = tabs_nb.current_page().unwrap_or(0);
            let kind = if page == 1 { EditorKind::Znc } else { EditorKind::Regular };
            (open_editor)(kind, Some(p));
        });
    }

    // Delete selected
    {
        let profiles_path = profiles_path.clone();
        let get_selected_name = get_selected_name.clone();
        let populate = populate.clone();
        let status = status.clone();

        del_btn.connect_clicked(move |_| {
            let Some(name) = (get_selected_name)() else { return };

            let mut pf = match crate::profiles::load(&profiles_path) {
                Ok(pf) => pf,
                Err(e) => {
                    status.set_text(&format!("Load failed: {e}"));
                    return;
                }
            };
            pf.profiles.retain(|p| p.name != name);

            if let Err(e) = crate::profiles::save(&profiles_path, &pf) {
                status.set_text(&format!("Save failed: {e}"));
                return;
            }

            status.set_text("Deleted profile");
            (populate)();
        });
    }

    // Connect selected
    {
        let profiles_path = profiles_path.clone();
        let get_selected_name = get_selected_name.clone();
        let backend_tx = backend_tx.clone();
        let next_conn_id = next_conn_id.clone();
        let conn_meta = conn_meta.clone();

        let notebook = notebook.clone();
        let tabs = tabs.clone();
        let page_to_buf = page_to_buf.clone();
        let unread = unread.clone();
        let highlights = highlights.clone();

        let sidebar_items = sidebar_items.clone();
        let sidebar_list = sidebar_list.clone();

        let current_buf = current_buf.clone();
        let win_close = win.clone();
        let status = status.clone();

        connect_btn.connect_clicked(move |_| {
            let Some(name) = (get_selected_name)() else { return };

            let Some(tx) = backend_tx.borrow().clone() else { return };

            let pf = match crate::profiles::load(&profiles_path) {
                Ok(pf) => pf,
                Err(e) => {
                    status.set_text(&format!("Load failed: {e}"));
                    return;
                }
            };

            let Some(p) = pf.profiles.into_iter().find(|p| p.name == name) else { return };

            let conn_id = {
                let mut n = next_conn_id.borrow_mut();
                let id = *n;
                *n += 1;
                id
            };

            conn_meta.borrow_mut().insert(
                conn_id,
                ConnMeta {
                    server: p.server.clone(),
                    nick: p.nick.clone(),
                    default_target: p.initial_channel.clone(),
                },
            );

            add_sidebar_connection(conn_id, &p.server, &sidebar_items, &sidebar_list);

            let status_key = BufferKey::new(conn_id, "Status");
            let status_disp = display_for_buffer(&status_key, &conn_meta.borrow());
            ensure_buffer(
                status_key.clone(),
                status_disp.clone(),
                false,
                &notebook,
                &tabs,
                &page_to_buf,
                &unread,
                &highlights,
            );
            add_sidebar_buffer(
                &status_key,
                &status_disp,
                &sidebar_items,
                &sidebar_list,
                &conn_meta.borrow(),
            );

            let chan_key = BufferKey::new(conn_id, p.initial_channel.clone());
            let chan_disp = display_for_buffer(&chan_key, &conn_meta.borrow());
            ensure_buffer(
                chan_key.clone(),
                chan_disp.clone(),
                true,
                &notebook,
                &tabs,
                &page_to_buf,
                &unread,
                &highlights,
            );
            add_sidebar_buffer(
                &chan_key,
                &chan_disp,
                &sidebar_items,
                &sidebar_list,
                &conn_meta.borrow(),
            );

            *current_buf.borrow_mut() = chan_key.clone();
            select_buffer(&chan_key, &notebook, &tabs);

            let cfg = p.to_irc_config();
            let _ = tx.send(BackendCmd::Connect { conn_id, cfg });

            status.set_text("Connecting…");
            win_close.close();
        });
    }

    win.present();
    win
}


fn find_next_in_current_buffer(
    query: &str,
    current_buf: &Rc<RefCell<BufferKey>>,
    tabs: &Rc<RefCell<HashMap<String, Tab>>>,
) {
    let q = query.trim();
    if q.is_empty() {
        return;
    }

    let key = current_buf.borrow().as_string();
    let (buf, view) = {
        let tabs_b = tabs.borrow();
        let Some(tab) = tabs_b.get(&key) else { return };
        (tab.buffer.clone(), tab.view.clone())
    };

    let buf: TextBuffer = buf;

    // Start from current selection end, otherwise from start.
    let mut start = buf.start_iter();
    let mut end = buf.end_iter();

    if let Some((_s, e)) = buf.selection_bounds() {
        start = e;
        end = buf.end_iter();
    }

    // Search forward, then wrap to start.
    let found = start.forward_search(q, gtk::TextSearchFlags::CASE_INSENSITIVE, Some(&end))
        .or_else(|| buf.start_iter().forward_search(q, gtk::TextSearchFlags::CASE_INSENSITIVE, Some(&buf.end_iter())));

    if let Some((mut m_start, m_end)) = found {
        buf.select_range(&m_start, &m_end);
        view.scroll_to_iter(&mut m_start, 0.0, false, 0.0, 0.0);
    }
}

fn load_log_into_current_buffer(
    current_buf: &Rc<RefCell<BufferKey>>,
    conn_meta: &Rc<RefCell<HashMap<u64, ConnMeta>>>,
    tabs: &Rc<RefCell<HashMap<String, Tab>>>,
) {
    let key = current_buf.borrow().clone();
    let conn_id = key.conn_id;
    let buffer_name = key.name.clone();

    let server = {
        let meta_b = conn_meta.borrow();
        let Some(meta) = meta_b.get(&conn_id) else { return };
        meta.server.clone()
    };

    let Ok(text) = crate::logging::read_all(&server, &buffer_name) else { return };

    let buf = {
        let tabs_b = tabs.borrow();
        let Some(tab) = tabs_b.get(&key.as_string()) else { return };
        tab.buffer.clone()
    };
    buf.set_text(&text);
}
>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)
