// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! The raw-wire hostile injector (M9-D6 / F-F / C4 — Checkpoint #3 surface).
//!
//! A **test-only** wire-client. The client binary's `ops::*` cannot *build* a
//! forged event, and the validation boundary rejects unsigned/malformed events —
//! so wire-attacks need a minimal client that speaks the transport directly to a
//! Node's `Server`, crafting frames with `xgen-core` builders and then
//! deliberately violating one invariant per attack. The node stays a separate
//! process (M9-D3); this just connects over the real `ws://` transport like any
//! peer/client would. **Never ships in a binary.**
//!
//! ## Grounded rejection boundary (Checkpoint #3 correction to the design)
//! The design cites `ingest_event` (runtime.rs:481) as the rejection point — but
//! `ingest_event` is the **no-validation direct-insert** (`"No 13-step
//! validation — caller is responsible"`); its only guard is `None => return` for
//! an unsigned event. The real validation boundary on the inbound path is
//! **`validate_event`** (the F-4 13-step core, `exchange.rs`) inside
//! `dispatch_event`, called by the node's `process_inbound`. A forged-signature
//! event is rejected at **step 12** (`verify_event_signature`), never reaching
//! `ingest_event`. The per-attack rejection points are recorded by the C4 test;
//! the headline is below in [`Attack`].
//!
//! ## Per-attack rejection points (grounded; live-confirmed where noted)
//! - **ForgedSignature** → `validate_event` step 12 (signature). *Live-confirmed*
//!   (no member context needed — the sig check precedes membership).
//! - **Malformed** → transport `recv` frame-parse (never deserializes into an
//!   `Event`, so it cannot reach `validate_event`). *Code-grounded; not
//!   live-confirmed here* — pushing a raw garbage frame needs sub-`Connection`
//!   socket access (`ws`/`send_bytes`/`encode_frame` are private), which would
//!   require an xgen-core change (out of scope) or a hand-rolled
//!   `connect_async` + framing. Recorded; a `pub` raw-send seam is the
//!   ergonomic option (Joe-lock) for the Multiparty-tests runs.
//! - **DuplicateId** → DAG dedup (`graph.add_event`), **after** validation — so
//!   isolating it needs an otherwise-valid (member-context) event; recorded,
//!   live-exercised in the member-context Multiparty-tests runs.
//! - **ClockSkew** → **CLOSED by M9.1 (F1 / gap G6):** `validate_event` now carries
//!   a Step 8.5 future-skew bound (`MAX_FUTURE_SKEW_SECS` = 300; wire 3046), so a
//!   far-future event (this injector stamps 2099) is rejected at admission, not
//!   silently accepted. Origin-blind (runs on both origins). The fix and its
//!   sensitivity witness live in xgen-core; this injector remains the adversarial
//!   input for the member-context Multiparty-tests runs (expect rejection + absence).
//! - **Equivocation** → not a rejection: two individually-valid conflicting
//!   events both apply and the M8 resolution converges on one winner (no fork).
//!   Outcome is *convergence-on-winner* (MP-A-06, R2), not absence.
//! - **ForgedInvite** → `validate_event` (membership / missing-predecessor) or
//!   HeldPending-then-timeout; member-context dependent, recorded.

use std::time::Duration;

use ed25519_dalek::SigningKey;
use xgen_common::wire::{Event, EventType};
use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};
use xgen_core::identity::keypair;
use xgen_core::space::state::sign_event;
use xgen_core::transport::client::connect_url;
use xgen_core::transport::connection::Inbound;
use xgen_core::wire::types::TransportMessage;

use crate::Result;
use anyhow::Context;

/// The catalogue of wire-attacks (M9-D6). Each violates exactly one invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attack {
    /// Signature made by a key that does not match the claimed `sender`.
    ForgedSignature,
    /// A truncated/garbage frame that cannot deserialize into an `Event`.
    Malformed,
    /// The same `event_id` submitted twice.
    DuplicateId,
    /// A valid event with a far-future/past timestamp (no member context here).
    ClockSkew,
    /// Conflicting events presented to two nodes at one frontier.
    Equivocation,
    /// An event referencing a non-existent invite/space.
    ForgedInvite,
}

/// The result of one injection attempt.
#[derive(Debug, Clone)]
pub struct InjectionResult {
    /// The identity the injector authenticated as (proves it got past WS/auth —
    /// so a later absence is a *validation* rejection, not an auth bounce).
    pub authed_as: Option<String>,
    /// The crafted event's `event_id` (the oracle checks this is absent).
    pub event_id: Option<String>,
    /// Any `Error` reply the node sent back (code, string). Often `None`: the
    /// event-acceptance reject paths log + drop rather than emit an `Error`
    /// (J-081; M6 envelope `event_id` on reject is PENDING) — absence from
    /// converged state is then the rejection signal.
    pub error_reply: Option<(u32, String)>,
}

/// Build a forged-signature event: `sender` claims `victim`, but the signature
/// is produced by `attacker_key` (≠ victim). Targets `space_id`/`room_id`
/// (synthetic is fine — the sig check at step 12 precedes space/membership).
pub fn build_forged_signature_event(
    victim_key: &SigningKey,
    attacker_key: &SigningKey,
    space_id: &str,
    room_id: &str,
) -> Event {
    let victim = IdentityXgid::from_pubkey(&victim_key.verifying_key());
    let ev = Event::new(
        EventType::MessageText,
        victim,
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        vec![EventXgid::from_xgid(Xgid::new(space_id.to_string()))],
        now_rfc3339(),
        serde_json::json!({ "text": "forged-signature injection (MP-A-05)" }),
    );
    // sign_event computes a valid event_id hash + signs with attacker_key; the
    // sender still claims victim → verify_event_signature fails at step 12.
    sign_event(ev, attacker_key)
}

/// Build a valid event signed by its own sender but with a far-future
/// timestamp (the ClockSkew attack body — used in member-context runs).
pub fn build_clock_skew_event(
    sender_key: &SigningKey,
    space_id: &str,
    room_id: &str,
    prev: Vec<&str>,
) -> Event {
    let sender = IdentityXgid::from_pubkey(&sender_key.verifying_key());
    let ev = Event::new(
        EventType::MessageText,
        sender,
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev.into_iter()
            .map(|p| EventXgid::from_xgid(Xgid::new(p.to_string())))
            .collect(),
        // Year 2099 — far outside any sane skew window.
        "2099-01-01T00:00:00.000Z".to_string(),
        serde_json::json!({ "text": "clock-skew injection" }),
    );
    sign_event(ev, sender_key)
}

/// Generate a fresh signing key (a throwaway injector identity).
pub fn fresh_key() -> SigningKey {
    keypair::generate()
}

/// Connect to `node_url`, authenticate as `auth_key`, send `event`, and briefly
/// listen for an `Error` reply. Returns what was observed. The crafted event's
/// fate (applied vs rejected) is determined by the oracle (absence), not here.
pub async fn inject_event(
    node_url: &str,
    auth_key: &SigningKey,
    event: &Event,
) -> Result<InjectionResult> {
    let mut conn = connect_url(node_url)
        .await
        .with_context(|| format!("injector connect {node_url}"))?;
    let authed_as = conn
        .client_authenticate(auth_key)
        .await
        .with_context(|| format!("injector authenticate to {node_url}"))?;

    conn.send_event(event)
        .await
        .context("injector send_event")?;

    let event_id = event.event_id.as_ref().map(|e| e.as_str().to_string());
    let error_reply = recv_error_within(&mut conn, Duration::from_millis(500)).await;
    Ok(InjectionResult {
        authed_as: Some(authed_as),
        event_id,
        error_reply,
    })
}

/// Listen for up to `dur` for a `TransportMessage::Error`; return `(code, str)`
/// if one arrives, else `None` (timeout / other message / closed).
async fn recv_error_within<S>(
    conn: &mut xgen_core::transport::connection::Connection<S>,
    dur: Duration,
) -> Option<(u32, String)>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    match tokio::time::timeout(dur, conn.recv()).await {
        Ok(Ok(Inbound::Transport(TransportMessage::Error {
            error_code,
            error_string,
            ..
        }))) => Some((error_code, error_string)),
        _ => None,
    }
}

fn now_rfc3339() -> String {
    use chrono::SecondsFormat;
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgen_core::space::state::verify_event_signature;

    #[test]
    fn forged_signature_event_fails_verification() {
        // The crux of MP-A-05: the crafted event's signature does NOT verify
        // against its claimed sender — exactly what validate_event step 12 catches.
        let victim = fresh_key();
        let attacker = fresh_key();
        let ev = build_forged_signature_event(
            &victim,
            &attacker,
            "xgen://hash/sha256:SPACE",
            "xgen://hash/sha256:ROOM",
        );
        assert!(ev.event_id.is_some(), "forged event still has a valid id hash");
        assert!(ev.signature.is_some(), "forged event is signed (by the wrong key)");
        assert!(
            !verify_event_signature(&ev),
            "forged-signature event MUST fail verification (claimed sender ≠ signer)"
        );
    }

    #[test]
    fn a_correctly_signed_event_verifies() {
        // Control: signing with the sender's own key verifies — proves the
        // forgery test isn't a false positive.
        let k = fresh_key();
        let sender = IdentityXgid::from_pubkey(&k.verifying_key());
        let ev = Event::new(
            EventType::MessageText,
            sender,
            RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:ROOM".to_string())),
            SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:SPACE".to_string())),
            vec![EventXgid::from_xgid(Xgid::new("xgen://hash/sha256:SPACE".to_string()))],
            now_rfc3339(),
            serde_json::json!({ "text": "honest" }),
        );
        let signed = sign_event(ev, &k);
        assert!(verify_event_signature(&signed));
    }

    #[test]
    fn clock_skew_event_is_well_formed_and_verifies() {
        // The skew attack body is a fully VALID event (correct sig, valid id) —
        // its only violation is the far-future (2099) timestamp. Post-M9.1,
        // validate_event's Step 8.5 future-skew bound rejects it at admission
        // (wire 3046); this test asserts the injector still produces the
        // well-formed adversarial input that the bound is meant to catch.
        let k = fresh_key();
        let ev = build_clock_skew_event(
            &k,
            "xgen://hash/sha256:SPACE",
            "xgen://hash/sha256:ROOM",
            vec!["xgen://hash/sha256:SPACE"],
        );
        assert!(verify_event_signature(&ev), "skew event is correctly signed");
        assert_eq!(ev.timestamp, "2099-01-01T00:00:00.000Z");
    }
}
