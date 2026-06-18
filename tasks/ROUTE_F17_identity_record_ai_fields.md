# Route F17 — `identity.record` omits `is_ai` / `ai_capabilities` (suspected code gap)
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-18  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## 0. What this is
A Joe-routed finding from the AFI audit AI sub-pass (J-398). Per the AFI ground rule (Q4: code is truth; suspected **code** bugs are routed, not doc-patched), this was held rather than reconciled in the doc. Seat: **Clair** (code) after Joe confirms the fix direction. Chat Claude owns the doc follow-up once the code lands.

## 1. The finding (AI-F17)
The wire message `IdentityMessage::Record` (`xgen-core/src/wire/types.rs`, `"identity.record"`) carries only:
`protocol_version, identity_id, display_name?, registered_at, devices, home_node`.

It **omits `is_ai` and `ai_capabilities`** — yet:
- The full `IdentityRecord` runtime struct (`xgen-core/src/identity/registry.rs`) carries both.
- `identity.register` (the request) carries both.
- The replication path (`IdentityReplicateMessage::Replicate.identity_record`) round-trips both (test `identity_record_round_trip_preserves_ai_fields`).
- §3.6.10 makes the AI declaration a **transparency** requirement — a public identity lookup that hides `is_ai` defeats that intent.

So a peer/client doing an `identity.get` lookup cannot learn whether the Identity is an AI from the `identity.record` response. Appendix I §IV.1 documents `is_ai`/`ai_capabilities` on `identity.record` — i.e. the **doc currently describes the intended shape, the wire message lags it**.

## 2. Direction — Joe-LOCKED: code fix (J-400)
Joe locked the **code fix** at J-400. The doc-fix alternative (§2.1) is retained for context only — NOT the chosen path.

**Code fix** — add `is_ai` and `ai_capabilities` to `IdentityMessage::Record`, populated from the stored `IdentityRecord` when the Node answers `identity.get`. Mirror the established serde discipline already used on `identity.register` / `IdentityRecord`:
- `is_ai: bool` with `#[serde(default, skip_serializing_if = "is_false")]` (omitted when `false` — keeps human-record responses byte-identical to today).
- `ai_capabilities: Option<AiCapabilities>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` (present iff `is_ai`).
This is additive + backward-compatible (Ch3 §3.0.3): old nodes omit, old clients ignore. Appendix I §IV.1 then needs **no change** — it already documents the target shape.

**Alternative (doc fix) — NOT chosen (J-400).** If the lookup response were deliberately minimal, trim the `is_ai`/`ai_capabilities` rows from Appendix I §IV.1 instead. Rejected — conflicts with §3.6.10 transparency.

## 3. Acceptance criteria (if code fix)
- `IdentityMessage::Record` gains `is_ai` + `ai_capabilities` with the serde attrs above.
- The `identity.get` responder populates both from the stored `IdentityRecord`.
- Round-trip test: an AI Identity's `identity.record` response carries `is_ai = true` + capabilities; a human's omits both.
- Back-compat test: a legacy `identity.record` JSON without the fields still deserialises (human record).
- In-suite baseline stays green.
- No Appendix I edit needed (doc already matches the target); add a one-line JOURNAL note that F17 closed code-side.

## 4. Provenance
AFI audit AI sub-pass — JOURNAL J-398; consolidated ledger in `tasks/HANDOFF_AFI_AUDIT.md` §9 (AI-F17 row).
