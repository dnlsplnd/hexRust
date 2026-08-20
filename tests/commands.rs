//! Slash-command planning. Pure, so no GTK display is involved.

use hexrust::commands::{is_channel, needs_channel, plan, Ctx, Out};

/// Typed while a channel buffer is active.
fn in_chan() -> Ctx<'static> {
    Ctx { current_buffer: "#rust", default_target: "#rust" }
}

/// Typed in the Status buffer, with a channel known for the connection.
fn in_status() -> Ctx<'static> {
    Ctx { current_buffer: "Status", default_target: "#rust" }
}

/// Typed in Status with no channel anywhere.
fn nowhere() -> Ctx<'static> {
    Ctx { current_buffer: "Status", default_target: "" }
}

fn raws(cmd: &str, args: &str, ctx: &Ctx) -> Vec<String> {
    plan(cmd, args, ctx)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|o| match o {
            Out::Raw(s) => Some(s),
            Out::Info(_) => None,
        })
        .collect()
}

fn one(cmd: &str, args: &str, ctx: &Ctx) -> String {
    let r = raws(cmd, args, ctx);
    assert_eq!(r.len(), 1, "expected exactly one raw line for /{cmd} {args}, got {r:?}");
    r.into_iter().next().unwrap()
}

#[test]
fn channel_prefixes_follow_rfc2811() {
    for s in ["#rust", "&local", "+modeless", "!12345chan"] {
        assert!(is_channel(s), "{s} should be a channel");
    }
    for s in ["nick", "", "Status", "@op"] {
        assert!(!is_channel(s), "{s} should not be a channel");
    }
}

#[test]
fn part_defaults_to_the_current_channel() {
    assert_eq!(one("part", "", &in_chan()), "PART #rust");
    assert_eq!(one("part", "#other", &in_chan()), "PART #other");
}

/// A bare reason must not be mistaken for a channel name.
#[test]
fn part_treats_a_non_channel_first_word_as_the_reason() {
    assert_eq!(one("part", "going home", &in_chan()), "PART #rust :going home");
    assert_eq!(one("part", "#other bye now", &in_chan()), "PART #other :bye now");
}

#[test]
fn quit_always_sends_a_reason() {
    assert_eq!(one("quit", "", &in_status()), "QUIT :hexrust");
    assert_eq!(one("quit", "see ya", &in_status()), "QUIT :see ya");
}

#[test]
fn topic_queries_when_empty_and_sets_when_given() {
    assert_eq!(one("topic", "", &in_chan()), "TOPIC #rust");
    assert_eq!(one("topic", "hello world", &in_chan()), "TOPIC #rust :hello world");
    assert_eq!(one("topic", "#other a b", &in_chan()), "TOPIC #other :a b");
}

/// The trailing parameter must be prefixed with ':' or multi-word topics and
/// reasons are truncated at the first space by the server.
#[test]
fn multiword_trailing_parameters_are_colon_prefixed() {
    assert!(one("topic", "two words", &in_chan()).contains(" :two words"));
    assert!(one("kick", "bob being noisy", &in_chan()).contains(" :being noisy"));
    assert!(one("quit", "two words", &in_chan()).contains(" :two words"));
    assert!(one("part", "two words", &in_chan()).contains(" :two words"));
}

#[test]
fn kick_resolves_channel_and_victim() {
    assert_eq!(one("kick", "bob", &in_chan()), "KICK #rust bob");
    assert_eq!(one("kick", "bob spam", &in_chan()), "KICK #rust bob :spam");
    assert_eq!(one("kick", "#other bob spam", &in_status()), "KICK #other bob :spam");
}

#[test]
fn mode_infers_the_channel_only_when_the_target_is_missing() {
    assert_eq!(one("mode", "", &in_chan()), "MODE #rust");
    assert_eq!(one("mode", "+m", &in_chan()), "MODE #rust +m");
    // An explicit target is left alone, including a nick.
    assert_eq!(one("mode", "#other +m", &in_chan()), "MODE #other +m");
    assert_eq!(one("mode", "alice +i", &in_status()), "MODE alice +i");
}

#[test]
fn op_and_friends_expand_to_mode_flags() {
    assert_eq!(one("op", "alice", &in_chan()), "MODE #rust +o alice");
    assert_eq!(one("deop", "alice", &in_chan()), "MODE #rust -o alice");
    assert_eq!(one("voice", "alice", &in_chan()), "MODE #rust +v alice");
}

/// One flag letter per nick, or the server applies the mode to the first only.
#[test]
fn op_repeats_the_flag_for_each_nick() {
    assert_eq!(one("op", "alice bob carol", &in_chan()), "MODE #rust +ooo alice bob carol");
    assert_eq!(one("devoice", "alice bob", &in_chan()), "MODE #rust -vv alice bob");
}

#[test]
fn away_and_back_toggle_correctly() {
    assert_eq!(one("away", "lunch", &in_status()), "AWAY :lunch");
    assert_eq!(one("away", "", &in_status()), "AWAY");
    assert_eq!(one("back", "", &in_status()), "AWAY");
}

#[test]
fn ctcp_payloads_are_wrapped_and_uppercased() {
    assert_eq!(
        one("ctcp", "alice version", &in_status()),
        "PRIVMSG alice :\u{0001}VERSION\u{0001}"
    );
    assert_eq!(one("ping", "alice", &in_status()), "PRIVMSG alice :\u{0001}PING\u{0001}");
    // Bare /version asks the server, not a user.
    assert_eq!(one("version", "", &in_status()), "VERSION");
}

#[test]
fn notice_echoes_locally_since_the_server_does_not_reflect_it() {
    let outs = plan("notice", "alice hello", &in_status()).unwrap();
    assert!(outs.contains(&Out::Raw("NOTICE alice :hello".into())));
    assert!(
        outs.iter().any(|o| matches!(o, Out::Info(t) if t.contains("hello"))),
        "no local echo: {outs:?}"
    );
}

/// A password must never be echoed into a buffer that gets written to disk.
#[test]
fn oper_is_never_echoed() {
    let outs = plan("oper", "root hunter2", &in_status()).unwrap();
    assert_eq!(outs, vec![Out::Raw("OPER root hunter2".into())]);
    assert!(
        !outs.iter().any(|o| matches!(o, Out::Info(t) if t.contains("hunter2"))),
        "password leaked into an echoed line"
    );
}

#[test]
fn missing_arguments_report_usage_and_send_nothing() {
    for (cmd, args) in [("whois", ""), ("kick", ""), ("notice", "alice"), ("invite", "")] {
        let outs = plan(cmd, args, &in_chan()).unwrap();
        assert!(
            outs.iter().all(|o| matches!(o, Out::Info(_))),
            "/{cmd} {args:?} sent something despite bad arguments: {outs:?}"
        );
        assert!(
            outs.iter().any(|o| matches!(o, Out::Info(t) if t.contains("Usage"))),
            "/{cmd} {args:?} gave no usage hint"
        );
    }
}

/// Commands needing a channel must decline rather than send a malformed line.
#[test]
fn channel_commands_decline_outside_a_channel() {
    // Args chosen so each case genuinely has to resolve a channel: /mode with
    // an explicit target (/mode alice +i) is valid anywhere, so it is exercised
    // here in the target-less form only.
    for (cmd, args) in [
        ("part", ""),
        ("topic", "hi"),
        ("kick", "bob"),
        ("names", ""),
        ("op", "alice"),
        ("ban", "*!*@host"),
        ("invite", "alice"),
        ("mode", "+m"),
    ] {
        assert!(
            plan(cmd, args, &nowhere()).is_none(),
            "/{cmd} {args:?} produced a line with no channel available"
        );
        assert!(needs_channel(cmd), "/{cmd} should be flagged as needing a channel");
    }
}

/// A nick target keeps working outside a channel; only the implied-channel
/// forms depend on one.
#[test]
fn mode_with_an_explicit_target_works_anywhere() {
    assert_eq!(one("mode", "alice +i", &nowhere()), "MODE alice +i");
}

/// Status-buffer use still works when the connection knows a default channel.
#[test]
fn channel_commands_fall_back_to_the_connection_default() {
    assert_eq!(one("part", "", &in_status()), "PART #rust");
    assert_eq!(one("names", "", &in_status()), "NAMES #rust");
}

#[test]
fn unknown_commands_are_not_owned_here() {
    for cmd in ["join", "msg", "me", "nick", "raw", "switch", "server", "query", "wallops"] {
        assert!(plan(cmd, "x", &in_chan()).is_none(), "/{cmd} should fall through");
        assert!(!needs_channel(cmd), "/{cmd} should not be flagged as channel-only");
    }
}

#[test]
fn help_lists_commands_without_touching_the_network() {
    let outs = plan("help", "", &in_status()).unwrap();
    assert!(outs.iter().all(|o| matches!(o, Out::Info(_))), "/help sent traffic");
    let text = outs
        .iter()
        .map(|o| match o {
            Out::Info(t) => t.clone(),
            Out::Raw(t) => t.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n");
    for expect in ["/join", "/part", "/whois", "/kick", "/topic", "/quit"] {
        assert!(text.contains(expect), "/help never mentions {expect}");
    }
}
