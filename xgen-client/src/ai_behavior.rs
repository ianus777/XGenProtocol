// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! AI Client plugin trait + reference implementation (M4, spec 3.6.10).
//!
//! The AI-mode resident's *behaviour* is a plugin. The runtime loop owns
//! WebSocket I/O, pacing, mute enforcement, and state replay; the plugin
//! decides — given the protocol context — whether to reply and what to say.
//!
//! M4 ships exactly one plugin: [`EchoPlugin`]. Its job is to prove the loop,
//! not to be useful. Future milestones add real plugins (LLM hookups, dialog
//! managers) as additional `AiBehavior` impls.
//!
//! ## Plugin contract (locked at M4 task-file v0.3)
//!
//! - The plugin is invoked once per inbound Event the AI receives.
//! - The plugin returns `Some(text)` to reply with `text` as `message.text`,
//!   or `None` for "no reply." The runtime takes it from there.
//! - The plugin MAY mutate its own state but MUST NOT block (no I/O, no
//!   long compute). Long-running work is a future-plugin concern.
//! - The plugin sees no SpaceState beyond what's in [`EventContext`] — mute
//!   handling, operator resolution, and pacing decisions are runtime
//!   concerns, not plugin concerns.
//!
//! ## Mention detection (locked at M4 v0.3 §6)
//!
//! Two-rail, OR'd, case-sensitive:
//!
//! 1. Substring match for the AI's full `identity_id` URI in `content.text`.
//! 2. Substring match for the optional `mention_token` (e.g. `"@bob"`) read
//!    from `[ai.behavior]`. Default `None` — Rail 1 only.
//!
//! If either rail matches, it counts. The implementation MUST NOT sequence
//! the rails (i.e. "fall through to optional if always-rail misses"); both
//! evaluate independently.

use xgen_common::xgid::IdentityXgid;
use xgen_core::wire::types::{Event, EventType};

/// Context passed to a plugin on each inbound Event. Carries the
/// information the plugin needs to make a reply decision; the runtime
/// holds the rest.
pub struct EventContext<'a> {
    /// The inbound Event being delivered to the AI.
    pub event: &'a Event,
    /// This AI's `identity_id` URI. Used by mention-detection plugins.
    pub ai_identity_id: &'a IdentityXgid,
    /// Optional mention token from `[ai.behavior] mention_token`. `None`
    /// means "no token configured; identity_id substring is the only rail."
    pub mention_token: Option<&'a str>,
}

/// Plugin trait. Implementations decide, given an inbound Event, whether
/// to emit a reply (returning `Some(text)`) or stay silent (`None`).
///
/// Plugins are `Send` so the runtime can move them across `tokio::spawn`
/// boundaries; they are not required to be `Sync` (one runtime thread
/// owns the plugin at a time).
pub trait AiBehavior: Send {
    /// Decide whether to reply to this Event. Implementations should be
    /// fast and non-blocking.
    fn on_event(&mut self, ctx: &EventContext) -> Option<String>;

    /// Plugin name, for logs and `__HEALTH__` output. Stable for the
    /// plugin's lifetime.
    fn name(&self) -> &'static str;
}

// ── Reference plugin: echo-on-mention ────────────────────────────────────────

/// The M4 reference plugin (locked at v0.3 §3). Replies to `message.text`
/// events that mention the AI with the deterministic line
/// `[echo-plugin] received mention from <sender_id_short>`, where
/// `sender_id_short` is the last 12 characters of the mentioning Identity's
/// `identity_id`. No configurable reply text — the format is fixed for
/// grep-ability in smoke tests and for unambiguity in early demos.
pub struct EchoPlugin;

impl EchoPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EchoPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AiBehavior for EchoPlugin {
    fn on_event(&mut self, ctx: &EventContext) -> Option<String> {
        // Only message.text triggers a reply. Other event types are ignored.
        if !matches!(ctx.event.event_type, EventType::MessageText) {
            return None;
        }

        // Don't reply to our own events.
        if ctx.event.sender.as_str() == ctx.ai_identity_id.as_str() {
            return None;
        }

        let text = ctx.event.content["text"].as_str().unwrap_or("");
        let mentioned = detect_mention(text, ctx.ai_identity_id.as_str(), ctx.mention_token);
        if !mentioned {
            return None;
        }

        let sender_short = short_id_suffix(ctx.event.sender.as_str());
        Some(format!("[echo-plugin] received mention from {sender_short}"))
    }

    fn name(&self) -> &'static str {
        "echo"
    }
}

/// Two-rail mention detection (locked at v0.3 §6). Both rails evaluated
/// independently; either match counts. Case-sensitive.
fn detect_mention(text: &str, ai_identity_id: &str, mention_token: Option<&str>) -> bool {
    let id_match = text.contains(ai_identity_id);
    let token_match = match mention_token {
        Some(tok) if !tok.is_empty() => text.contains(tok),
        _ => false,
    };
    id_match || token_match
}

/// Return the last 12 characters of an `identity_id` URI as a short suffix
/// suitable for log lines and the echo reply. Falls back to the full
/// string if it's shorter than 12 characters.
fn short_id_suffix(identity_id: &str) -> &str {
    let len = identity_id.len();
    if len <= 12 {
        identity_id
    } else {
        &identity_id[len - 12..]
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use xgen_common::xgid::{RoomXgid, SpaceXgid, Xgid};

    const AI_ID: &str = "xgen://pubkey/ed25519:BOTxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    const ALICE_ID: &str = "xgen://pubkey/ed25519:ALICExxxxxxxxxxxxxxxxxxxxxxxxxx";
    const ROOM: &str = "xgen://hash/sha256:room";
    const SPACE: &str = "xgen://hash/sha256:space";

    fn make_text_event(sender: &str, text: &str) -> Event {
        Event::new(
            EventType::MessageText,
            IdentityXgid::from_xgid(Xgid::new(sender.to_string())),
            RoomXgid::from_xgid(Xgid::new(ROOM.to_string())),
            SpaceXgid::from_xgid(Xgid::new(SPACE.to_string())),
            vec![],
            "2026-05-17T10:00:00.000Z".to_string(),
            json!({ "text": text }),
        )
    }

    #[test]
    fn mention_via_identity_id_triggers_reply() {
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let ev = make_text_event(ALICE_ID, &format!("hey {AI_ID} you up?"));
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai,
            mention_token: None,
        };
        let reply = plugin.on_event(&ctx).expect("expected reply");
        assert!(reply.starts_with("[echo-plugin] received mention from "));
        // sender_short is last 12 chars of ALICE_ID
        assert!(reply.ends_with("xxxxxxxxxxxx"));
    }

    #[test]
    fn mention_via_token_triggers_reply() {
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let ev = make_text_event(ALICE_ID, "hey @bob you up?");
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai,
            mention_token: Some("@bob"),
        };
        let reply = plugin.on_event(&ctx).expect("expected reply via token");
        assert!(reply.starts_with("[echo-plugin] received mention from "));
    }

    #[test]
    fn no_mention_no_reply() {
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let ev = make_text_event(ALICE_ID, "just talking about the weather");
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai,
            mention_token: Some("@bob"),
        };
        assert!(plugin.on_event(&ctx).is_none());
    }

    #[test]
    fn rails_are_ord_not_sequenced() {
        // Both rails evaluated independently. Rail B match alone is enough
        // — implementation must not fall through only when Rail A misses.
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let ev = make_text_event(ALICE_ID, "@bob standalone");
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai,
            mention_token: Some("@bob"),
        };
        assert!(plugin.on_event(&ctx).is_some(), "Rail B alone must trigger");

        // And Rail A alone must also trigger when token is set.
        let ev2 = make_text_event(ALICE_ID, &format!("hi {AI_ID}"));
        let ctx2 = EventContext {
            event: &ev2,
            ai_identity_id: &ai,
            mention_token: Some("@bob"),
        };
        assert!(plugin.on_event(&ctx2).is_some(), "Rail A alone must trigger");
    }

    #[test]
    fn case_sensitive_token_match() {
        // "@Bob" (capital B) must NOT match when token is "@bob".
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let ev = make_text_event(ALICE_ID, "hey @Bob");
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai,
            mention_token: Some("@bob"),
        };
        assert!(plugin.on_event(&ctx).is_none());
    }

    #[test]
    fn self_mentions_dont_trigger_reply() {
        // If the AI's own event somehow loops back (or the AI mentions its
        // own identity_id in a non-reply event), the plugin must not reply.
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let ev = make_text_event(AI_ID, &format!("test mentioning {AI_ID}"));
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai,
            mention_token: None,
        };
        assert!(plugin.on_event(&ctx).is_none());
    }

    #[test]
    fn non_text_events_ignored() {
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let mut ev = make_text_event(ALICE_ID, &format!("ping {AI_ID}"));
        ev.event_type = EventType::StateRoomCreate;
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai,
            mention_token: None,
        };
        assert!(plugin.on_event(&ctx).is_none());
    }

    #[test]
    fn empty_mention_token_treated_as_unset() {
        // Defensive: `mention_token: Some("")` is treated as "no token";
        // every text would match an empty substring otherwise.
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let ev = make_text_event(ALICE_ID, "nothing relevant here");
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai,
            mention_token: Some(""),
        };
        assert!(plugin.on_event(&ctx).is_none());
    }

    /// T12 — `EventContext.ai_identity_id` is `&IdentityXgid` (design doc
    /// §4.6.c / Q4). Mention detection reads it via `.as_str()`; a mention by
    /// full XGID triggers a reply.
    #[test]
    fn ai_behavior_event_context_ai_identity_id_typed() {
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let ev = make_text_event(ALICE_ID, &format!("hey {AI_ID} ping"));
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai, // &IdentityXgid, not &str
            mention_token: None,
        };
        assert!(
            plugin.on_event(&ctx).is_some(),
            "mention by typed ai_identity_id triggers reply"
        );
    }

    #[test]
    fn plugin_name_is_echo() {
        assert_eq!(EchoPlugin::new().name(), "echo");
    }

    #[test]
    fn reply_text_format_matches_spec() {
        // §3 contract: exactly "[echo-plugin] received mention from <hash>"
        // where <hash> is last 12 chars of sender identity_id.
        let mut plugin = EchoPlugin::new();
        let ai = IdentityXgid::from_xgid(Xgid::new(AI_ID.to_string()));
        let ev = make_text_event(ALICE_ID, &format!("@{AI_ID}"));
        let ctx = EventContext {
            event: &ev,
            ai_identity_id: &ai,
            mention_token: None,
        };
        let reply = plugin.on_event(&ctx).unwrap();
        let expected_suffix = &ALICE_ID[ALICE_ID.len() - 12..];
        assert_eq!(reply, format!("[echo-plugin] received mention from {expected_suffix}"));
    }
}
