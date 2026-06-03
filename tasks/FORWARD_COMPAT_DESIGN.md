# XGen Protocol — Forward-Compatibility (Unknown-Event Relay) Design + Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Locked decisions (FC-D#, arc-local per D-069)

Closes **PG-09**. Fork 2 (relay-unknown) locked at audit. Shape A locked (Joe, 2026-06-03).

- **FC-D1 — Shape A.** `EventType::Unknown(String)` catch-all variant holds the raw wire type string. "Unknown" lives inside the type system; `event.event_type` stays a typed `EventType` everywhere.
- **FC-D2 — spec reconcile (ships same commit, D-074).** ch3 §3.2 L648 + ch2 L381 are authoritative. Appendix I L75 corrected: *"`type` is an open namespace; a Node stores, relays, and ignores unknown types — it does not reject them."*
- **FC-D3 — sync/replay survival.** Unknown events round-trip store → `range`/replay → re-serialize byte-identically; the raw type string is the canonical-bytes truth. Replay deserializes via the now-tolerant `Event` and does not drop them.
- **FC-D4 — DAG referenceability.** An unknown event has a valid `event_id` and may appear in a later known event's `prev_events`; `graph.rs`/`HeldPending` treat it as a normal node — no special-casing.
- **FC-D5 — filter semantics (free).** `from_str` **stays strict** (`None` on unknown), so a subscription filter can only name known types (unknown exact name → `BAD_ARGUMENT`, EV-D4 unchanged); `*` / `family.*` wildcards match unknown types via the event's stored type string.
- **FC-D6 — no apply, no side effects.** Unknown events never mutate state. The apply dispatch's `Unknown` arm is a no-op; no membership/permission/temperature/audit logic fires.

**Key invariant (the whole point of Shape A's two entry points):** *deserialization is tolerant* (unknown → `Unknown(s)`), *`from_str` is strict* (unknown → `None`). They do different jobs and must not be merged.

---

## §2 — Shape-A mechanism

**`EventType` (`xgen-common/wire.rs`):**
1. Add variant `Unknown(String)` (holds the exact wire type string).
2. **Custom `Deserialize`** (replaces the derive): deserialize the field as `String`; `EventType::from_str(&s)` → the known variant; else `EventType::Unknown(s)`. Tolerant — never errors on an unrecognised type.
3. **Custom `Serialize`** (replaces the derive): emit `self.as_str()` as a plain string. Round-trips byte-identically for known types (same wire strings) and for unknown (emits the stored string).
4. **`as_str(&self) -> &str`** (was `-> &'static str`): known arms return their literal; `Unknown(s) => s`. *Signature change — see confirm-at-pickup.* `Display` already delegates to `as_str`, unchanged.
5. **`from_str(s: &str) -> Option<Self>`** — **unchanged, stays strict** (known → `Some`, unknown → `None`). Do **not** add an `Unknown` arm here.

**Validation (`xgen-core` `validation.rs` step 6):** the `UnknownEventType` rejection becomes accept-as-opaque — an unknown type is structurally valid (it has a valid id + signature, verified type-blind). Remove/relax the step-6 error path; keep all other structural checks. (Confirm whether step 6 currently keys off `from_str` on raw bytes or off the deserialized enum — the fix sits wherever the rejection is.)

**Apply dispatch (`exchange.rs:300 match event.event_type`):** add `EventType::Unknown(_) => { /* store + relay only; no state mutation */ }`. This is the chokepoint for FC-D6.

**Every other exhaustive `match` on `EventType`:** add an `Unknown(_)` arm. **The compiler enumerates them** (exhaustiveness) — Clair adds the variant first, then fixes each compile error per these rules:
- pure classification (e.g. "is this a state event?", priority, audit-category) → `Unknown(_)` takes the **inert/default** branch (not a state event, lowest priority, not audited as a known category).
- any arm that would *do work* → `Unknown(_)` must **no-op** (FC-D6).
- never `unreachable!()` / `panic!()` on `Unknown` — it is reachable from the wire.

---

## §3 — Commit plan (runbook for Clair)

Single bounded arc; **known-type behaviour stays green throughout** (vanilla byte-identical for known events). Suggested two commits:

**C1 — `xgen-common` type layer.** `EventType::Unknown(String)` + custom `Serialize`/`Deserialize` + `as_str → &str` + `Display` unchanged + `from_str` unchanged (strict). Unit tests: known type round-trips byte-identically (serialize→deserialize→serialize); `"bogus.type"` deserializes to `Unknown("bogus.type")` and re-serializes to `"bogus.type"`; `from_str("bogus.type") == None`. `cargo test -p xgen-common` + build + clippy.

**C2 — `xgen-core`/`xgen-node` ingest + relay + the match arms.** Relax validation step 6 (accept-as-opaque); add the `Unknown(_)` apply-dispatch no-op arm; fix every compiler-listed exhaustive match per §2 rules; confirm sync/replay (FC-D3) and the filter (FC-D5) need no change beyond the compiler arms. Integration tests: a signed unknown-type event → deserialize → validate (pass) → store → appears in fan-out + sync batch → **not applied** (Space/Room state unchanged) → replay round-trips it; a `*`-filter matches it; a known-type-named filter does not; known-type flows unchanged. `cargo test --workspace` + build all-targets + clippy `-D warnings`.

**Close (D-074 atomic, doc-only after C2):** PROTOCOL_GAP_AUDIT §5 PG-09 → ✅ DONE; FC-D2 spec reconcile (Appendix I L75 + a §3.2 as-built note in ch3); this design + the audit → COMPLETED; DECISIONS call on whether any FC-D# promotes (default arc-local, D-069 — likely stays arc-local). Joe pushes.

Per-commit: `cargo test --workspace` + build all-targets + clippy `-D warnings`; baseline 1024/0/2. `Filesystem:*` for E:\ writes; never `create_file`. Node `--batch`/`pipe.rs` untouched unless an arm sits there.

---

## §4 — Confirm-at-pickup (D-078)

1. **`as_str` signature change** `&'static str → &str` — find callers that bind it to `&'static str` specifically (most use `&str`/`Display`); adjust. Likely small.
2. **Where L1 deserialization happens** in the ingest path vs the raw-bytes structural validation — confirm the custom `Deserialize` is the actual gate an inbound federated event passes through (and that step 6's rejection is the only other one). The audit located both; Clair confirms the exact call order.
3. **`exchange.rs:300` dispatch** — confirm it is *the* apply/validate-by-type dispatch and that `Unknown(_) => no-op` is the complete FC-D6 chokepoint (no second state-mutating match elsewhere).
4. **Serialize byte-identity** for known types must not regress (custom Serialize must emit exactly the prior rename strings) — the C1 round-trip test guards this; double-check canonical-bytes/event-id stability for known events.

---

## §5 — Out of scope / non-goals

- No new event types are added — this is purely about *relaying* types a Node does not know.
- Unknown events are never applied, never trigger side effects, never surface in known-type-specific UI (FC-D6).
- Does not touch version negotiation (major/minor, §3.1) — orthogonal; this is within-major-version forward-compat.
- Clair stands down until Joe kicks off C1.
