# XGen Protocol — Forward-Compatibility (Unknown-Event Relay) Audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

**CLOSED 2026-06-03 (J-236).** PG-09 shipped end-to-end — C1 `9bf57d1` (xgen-common type layer) + C2 `e0a1972` (ingest/relay/match arms). Suite 1035/0/2, clippy clean both feature sets. Two as-built deltas vs this audit (detailed in `FORWARD_COMPAT_DESIGN.md` close): FC-D6 chokepoint is `state.rs:450 apply_event`, not `exchange.rs:300` (which is a `_ => Ok()` classifier); validation step-6 reject was test-only — the production inbound gate is `connection.rs:203 from_value::<Event>` (C1's tolerant Deserialize).

---

## §1 — Purpose + locked fork

Closes **PG-09** (PROTOCOL_GAP_AUDIT §3). Realises the spec's forward-compatibility guarantee — ch3 §3.2 L648: *"A Node receiving an Event with an unrecognised `type` value MUST store the Event in the log and propagate it to peers"* + ch2 L381 (unknown types stored, forwarded, ignored). Phase 0 for the **Unknown-Event Forward-Compat** arc (Wave 1 / Arc B).

**Fork locked (Joe, 2026-06-03): Fork 2 — relay-unknown wins.** Unknown-type events are stored + relayed but not applied; the closed-set statement (Appendix I L75) is the outlier and is corrected to match. (Fork 1 — keep closed enum, reject unknown, delete the design intent — was rejected: it kills extensibility and partitions federation across versions.)

This is the hard multiparty (M8/M9) prerequisite: version/feature skew across federated Nodes is intrinsic, so a Node MUST relay event types it does not understand.

---

## §2 — As-is: the gap is three layers deep

The audit assumed PG-09 lived at validation step 6. Grounding found it is **deeper** — an unknown-type event is rejected at the **serde layer**, before validation ever runs:

| Layer | Site | Behaviour today | Effect |
|---|---|---|---|
| **L1 — deserialization** | `wire.rs` `Event.event_type: EventType` (closed enum, `#[derive(Deserialize)]`, `#[serde(rename="type")]`) | serde has **no catch-all** for the `type` field → deserializing an Event whose `type` is unknown **fails outright** | the event never becomes an `Event` value — *the real blocker* |
| **L2 — structural validation** | `validation.rs:112` `ValidationError::UnknownEventType` (step 6) | even if L1 were relaxed, step 6 **rejects** unknown types | second rejection point |
| **L3 — representation** | `wire.rs:24` `EventType` (closed enum) + `from_str → None` (`:256`) | no variant can hold an unknown type string | nothing to store even if L1/L2 passed |

**Already-correct machinery (Fork 2 rides on it):**
- **Type-agnostic id + signature** — `canonical_event_bytes(&Value)` excludes id+signature (`canonical.rs:40/131`); an unknown event's `event_id` and signature verify **without** knowing the type. Structural validation (steps 1–5,7) and signature (step 12) are type-blind.
- **Apply-skip exists** — `state.rs:476 _ => Ok()` already no-ops unknown types at apply (today unreachable; becomes the correct path).
- **Relay is event-driven** — fan-out (`runtime.rs:677` `FanoutRequest`, `apply_fanout`) forwards the event value, no per-type logic needed to relay.

**Internal spec contradiction to fix (same arc):** Appendix I L75 ("type MUST be one of the known strings") contradicts ch3 §3.2 L648. Fork 2 makes ch3 §3.2 + ch2 L381 authoritative; Appendix I L75 is corrected to "type is an open namespace; unknown types are stored, relayed, and ignored, not rejected."

---

## §3 — Target behaviour (Fork 2)

An Event whose `type` is not in the known set MUST:
1. **Deserialize** into an `Event` value, preserving the raw type string (L1 fix).
2. Pass **structural validation** (steps 1–5, 7) and **signature** verification (step 12) — unchanged, type-blind. Step 6 (L2) flips from *reject* → *accept-as-opaque*.
3. Be **stored** in the Event log (real event, real `event_id`, referenceable in `prev_events`).
4. Be **relayed** through fan-out + included in sync/replay batches.
5. **Not be applied** to Space/Room state (the `_ => Ok()` skip, L3) — it changes no state.
6. Be **ignored** by clients/drivers that do not understand it (surfaced via the events pipe; a `*` filter matches it, a named-type filter cannot name it).

Known event types: **behaviour unchanged**. Vanilla path byte-identical for known events.

---

## §4 — The shape decision (next Joe-lock)

Both shapes deliver §3; they differ in *where* "unknown" is represented.

**Shape A — `EventType::Unknown(String)` variant.** Add a catch-all variant holding the raw type string. Needs a **custom `Deserialize`** for `EventType` (serde `#[serde(other)]` only supports a *unit* variant — it would drop the string), plus `as_str`/`from_str`/`Display` arms. Ripple: up to ~440 `EventType::` sites; Rust exhaustiveness flags every exhaustive `match` at compile time → **safe, mechanical labour**, but touches many files.
- **Pro:** `Event.event_type` stays a single typed field; the type is always an `EventType`; minimal change to call sites that only *compare* (`== EventType::Foo` still works).
- **Con:** every exhaustive `match` on `EventType` must add an `Unknown(_)` arm; custom Deserialize is a subtle hand-rolled impl.

**Shape B — raw type-string on `Event`, enum stays closed.** `Event` carries the wire type as a `String` (the truth); known-type logic resolves lazily via `EventType::from_str(&s) -> Option<EventType>`. Unknown → `None` → stored/relayed, never applied.
- **Pro:** `EventType` enum + its exhaustive matches are **untouched** (no Unknown arm anywhere); "unknown = `None`" is a natural, honest representation; deser never fails (it's just a String).
- **Con:** changes the `Event` struct's `event_type` field shape; every site that reads `event.event_type` expecting an `EventType` must call `from_str` (a *different* but comparably large ripple); two sources of truth (raw string + resolved enum) need a single disciplined accessor to avoid drift.

**Recommendation: Shape A.** Rationale: (1) it keeps `event.event_type` a typed `EventType` everywhere, so the hundreds of comparison/known-match sites are unchanged — only the ~dozens of *exhaustive* matches need an `Unknown` arm, all compiler-flagged; (2) "unknown" stays inside the type system rather than as a stringly-typed side channel, matching how the codebase already models types; (3) the custom Deserialize is localized to one place. Shape B's "raw string + lazy resolve" spreads `from_str` calls across every read site and invites raw/resolved drift (the exact D-077 sustainability smell). Shape A's ripple is larger in count but safer in kind (exhaustiveness-checked, one-time).

*Confirm-at-pickup for whichever shape:* the M7-events subscription filter `matches` predicate (EventType-based) — a `*` / family-wildcard filter must match an unknown type; a named-type filter cannot name one. Light.

---

## §5 — Design decisions (resolve at design-lock, FC-D# arc-local per D-069)

- **FC-D1** shape A vs B — **pending Joe-lock** (rec: A).
- **FC-D2** spec reconcile — Appendix I L75 → open-namespace wording; ch3 §3.2 L648 + ch2 L381 confirmed authoritative; ch0/Appendix C unaffected. Ships same commit as code (D-074).
- **FC-D3** sync/replay survival — unknown events must round-trip store → `range`/replay → re-serialize byte-identically (the raw type string is the canonical-bytes truth; confirm replay does not re-validate-by-enum and drop them).
- **FC-D4** DAG referenceability — an unknown event has a valid `event_id` and may appear in a later known event's `prev_events`; the `HeldPending`/`graph.rs` path treats it as a normal node (no special-casing; confirm).
- **FC-D5** filter semantics — `*` and `family.*` wildcards match unknown types; an exact unknown type cannot be named in a filter (stays `BAD_ARGUMENT`, EV-D4).
- **FC-D6** no apply, no side effects — unknown events never mutate state, never trigger membership/permission/temperature logic (the `_ => Ok()` skip is the single chokepoint; confirm no other exhaustive match silently does work).

---

## §6 — Touch-surface (Shape A, estimate)

`xgen-common/wire.rs` (EventType +Unknown variant, custom Deserialize, as_str/from_str/Display arms) · `xgen-core` validation step 6 (`validation.rs` accept-as-opaque) + every exhaustive `EventType` match (+Unknown arm, compiler-listed) · apply path confirm (`state.rs:476` already correct) · sync/replay confirm (FC-D3) · filter confirm (FC-D5) · docs: Appendix I L75 + a §3.2 as-built note. Tests: unknown-type round-trips deser→validate→store→relay→replay; unknown-type not applied; known types unchanged; `*`-filter matches unknown.

---

## §7 — Next

**Joe-lock FC-D1 (shape A vs B)** → write design + runbook (FC-D2…D6 locked) → **Clair implements** (single atomic-ish arc; known-type behaviour must stay green throughout) → close (D-074: PROTOCOL_GAP_AUDIT §5 PG-09 → DONE; Appendix I + §3.2 reconcile; DECISIONS call on whether any FC-D# promotes). Clair stands down until the shape locks.
