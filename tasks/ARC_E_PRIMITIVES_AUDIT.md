# XGen Protocol — Arc E (Primitive Completion) Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Method, sources, scope

### 0.1 Purpose

The **D-071 Phase-0 grounding** for Arc E, selected at the J-244 milestone-selection point (Arc D CLOSED, suite 1060/0/2). Arc E is the **primitive-completion** cluster from gap-audit §4-E: two catalogued S2 gaps — **PG-03** (`TrustAssertion` first-class primitive) + **PG-08** (`Thread` primitive). This audit grounds both against as-built code, settles scope + sequence, and feeds the design beat. Doc-only — no code; suite at J-244's **1060**/0/2, not re-run.

### 0.2 Scope decision (recorded, confirm at close)

- **IN:** PG-03 (TrustAssertion) + PG-08 (Thread).
- **OUT — full first-class Role object model.** Appendix C carries `class Role` (`id`, `name`, `position`, `created_at`) as a documented primitive, and J-244 flagged it as "clustering into arc E." Grounding overrides that convenience: the Role model is **Arc-D lineage** (it extends the PG-12-min per-Room override Arc D shipped, into custom roles / `permissions[]` / `position` / `Guest`), and it **depends on** the tier + assertion substrate landing first to be meaningful. It is spun out to a later privilege-model arc.
- **Sequence:** PG-03 first, PG-08 second (rationale in §3.1).

### 0.3 Sources

**Spec:** ch1 L210/215 (protocol owns the trust assertion); ch2 §"Thread Model" L598–735 + L1372/1390/1391 (TA uniform structure / expires_at / issuer) + L1463 (revoked module voids its assertions); ch3 §3.8 (Auth Module + Trust Assertion, full); Appendix C class diagrams (`TrustAssertion`, `Thread`, `Role`); Appendix J (XGID typed flavours, D-072/D-073).

**Code:** `xgen-common/src/xgid/flavours.rs` (TA flavour wrapper + Pass-2 deferral) · `xgen-core/src/identity/registration.rs` (the assertion seam) · `xgen-core/src/auth/tiers.rs` (the PG-13 gate) · `xgen-common/src/wire.rs` (EventType enum) · `xgen-core/src/space/state.rs` (applier pattern).

### 0.4 Verdict vocabulary

Reused from `PROTOCOL_GAP_AUDIT` §0.4: **NO-GAP · GAP-CONFIRMED · SPEC-DRIFT · NEEDS-DESIGN · N/A**. Finding IDs are **`AE-A##`** (audit findings, append-only).

---

## §1 — PG-03: TrustAssertion grounding

### 1.1 Findings

**AE-A1 — the struct is absent; the scaffolding around it already exists.** `flavours.rs:35` records that `TrustAssertionXgid::from_assertion` is **deferred to Pass 2** because the `TrustAssertion` struct does not exist in Rust. What *does* exist: the `TrustAssertionXgid` flavour wrapper itself (hash-anchored, `xgen://hash/sha256:`); the three registration error variants `TrustAssertionRequired` (3003) / `AssertionSignatureInvalid` (3004) / `AssertionExpired` (3005) (`registration.rs:67–70/92–94`); and the full wire schema in ch3 §3.8.4. **Verdict: GAP-CONFIRMED.** The closing cost is "fill in the documented, scaffolded shape," not "design from scratch."

**AE-A2 — registration steps 5–7 are DEAD CODE today; PG-03 is what activates them.** `registration.rs:232–238` runs only **step 4** (presence → 3003), then binds-and-drops the assertion: `let _assertion = trust_assertion.ok_or(...)?;` with the comment *"Steps 5–7 deferred to Phase 2."* The 3004/3005 error variants exist but are **never returned** anywhere in `xgen-core`/`xgen-node`. So ch3 §3.8.5's seven-check MUST ("all seven checks MUST pass") is **only 1/7 enforced**. Implementing the struct is precisely what makes steps 5 (signature verify) / 6 (`valid_until` future) / 7 (required claims) real. **This widens PG-03** from "a struct is missing" to "the registration validation pipeline's back half is unimplemented because the artefact it validates doesn't exist."

**AE-A3 — PG-03 is the keystone under Arc D's wired-but-no-op tier-gate.** Arc D (J-243) wired `verify_tier_assertion` onto the join path, with the joiner tier read by `assertion_tier_of(record)` (PM-D2) — which today reads `record["tier"]` **heuristically** (`None→1`, `Some(v)→v["tier"]`). A real `TrustAssertion` is the authoritative tier source. PG-03 is therefore what gives PG-13's honest Tier-1 no-op its **teeth** — the two gaps are a matched pair, and this is the strongest argument for PG-03-first.

**AE-A4 — TrustAssertion is a SignedPrimitive (1 of only 3).** Appendix C L115: only **Event, Node, TrustAssertion** are `SignedPrimitive`s — "entities whose authenticity must be independently verifiable by any recipient without trusting the source." The struct needs **canonical-form signing + verify**, reusing Event canonicalisation (§3.2.4, `canonical.rs`). Field order is **locked by §3.8.5**: `type, tier, issuer, identity_id, issued_at, valid_until, claims`. This is real crypto-adjacent work (Ed25519 verify against the `issuer` key), not a plain data struct.

**AE-A5 — SPEC-DRIFT between the wire schema and the class diagram (design must reconcile).** ch3 §3.8.4 (the **wire authority**): `{ type, tier, issuer, identity_id, issued_at, valid_until, claims, signature }`. Appendix C `class TrustAssertion`: `{ identity, tier, issued_at, expires_at, issuer, jurisdiction }`. Drifts: (a) **`valid_until`** (ch3 wire) vs **`expires_at`** (AppC + ch2 L1390) — the wire name wins, AppC is the drift; (b) AppC adds **`jurisdiction`** — absent from the ch3 wire schema (tendril to PG-04, see AE-A7); (c) AppC omits `claims`/`signature` — diagram abbreviation, not a real divergence. **Verdict: SPEC-DRIFT** — ch3 §3.8.4 is authoritative; the AppC class row gets reconciled at close.

**AE-A6 — the "Pass 2" framing question.** `flavours.rs` calls the deferred constructor "Pass 2" of the XGID Retrofit arc ("Pass 2 owns the auth-module surfaces"). Implementing the `TrustAssertion` struct **is** that deferred content. Design must decide: frame PG-03 as the formal **XGID Retrofit Pass 2**, or keep it Arc-E-local with a one-line cross-reference. Recommendation: arc-E-local, cross-referenced — the Retrofit-Pass framing was about typed-flavour retype (D-072/073), and the struct is a superset of that narrow constructor.

**AE-A7 — `jurisdiction` is a PG-04 tendril; note, do not fold.** AppC's `jurisdiction` field on TrustAssertion connects to PG-04 (federation jurisdictional namespacing). Design records whether the Rust struct carries an (optional) `jurisdiction` slot now (cheap forward-compat) or omits it until PG-04 (arc G). Do **not** pull PG-04 into Arc E.

**AE-A8 — scope line: protocol artefact vs live Auth Module.** §3.8 specifies the Auth Module *interface* (record, verify-request/confirm, validity-query endpoint, trusted-list registration §3.8.7). The honest Arc-E deliverable is the **`TrustAssertion` struct + canonical sign/verify + registration steps 5–7 + a synthetic test issuer** — i.e. the protocol-side validation logic. A **live Auth Module service** (real email/phone verification, the network query endpoint) is **Tier 2–4 institutional, out of core-team scope** (ch2 L1320–1334; only Tier 1 ships). Design must state this line explicitly so the close does not overclaim.

**AE-A9 — honesty posture (D-065), mirrors PG-13.** Even with the struct built and correct, Tier-1 **Local Node mode bypasses the Auth Module entirely** (§3.8.8 — no assertion required), and the system is Tier-1-only today (no real issuer). So PG-03 ships **real, tested validation logic exercised by synthetic assertions**, load-bearing only once a real Tier 2–4 Auth Module exists. This is the same honest-no-op shape Arc D adopted for the tier-gate — record it the same way at close.

### 1.2 PG-03 verdict

**GAP-CONFIRMED, sized as focused depth.** One SignedPrimitive struct + canonical Ed25519 sign/verify + activation of registration steps 5–7 (3004/3005 go live) + an `assertion_tier_of` rewire to read the real assertion + a synthetic test issuer. Conceptually the heaviest strand; mechanically contained.

---

## §2 — PG-08: Thread grounding

### 2.1 Findings

**AE-A10 — zero implementation, rich + stable spec.** Confirmed by grep: **no `Thread`/`thread.*` `EventType`, no `ThreadStatus` enum** anywhere in `xgen-common`/`xgen-core` (the only `thread` hits are `std::thread`/tokio prose). ch2 §"Thread Model" (L598–735) is one of the most fully-developed primitive specs in the document: anatomy, lifecycle, per-Room-type behaviour, notification model. **Verdict: GAP-CONFIRMED.**

**AE-A11 — the event set must be locked (spec lists one, lifecycle implies three).** ch2 L323's event table lists `thread.create`; the lifecycle (CREATED→OPEN→RESOLVED/ARCHIVED) implies **`thread.resolved`** + **`thread.archived`** as State Events. Design locks the full set — provisionally **3 events**: `thread.create` (origin, carries topic/creator/initial content) + `thread.resolved` + `thread.archived` (State Events) — as new `EventType` variants.

**AE-A12 — the Space/Room pattern it mirrors is well-worn.** Closing PG-08 follows the established primitive path: `EventType` variant(s) in `wire.rs` → state shape (`ThreadState`) in `space/state.rs` → applier arm(s) → validation → a `build_thread_*_event` builder → tests. M8 (J-238–241) just exercised this exact apply/state path, so the seam is fresh. Mechanical breadth, low conceptual surprise.

**AE-A13 — `auth_tier_min` couples Thread to PG-03 (reinforces PG-03-first).** A Thread "can require a higher auth tier than its parent Room … narrow it further but never widen it beyond the Room's own minimum" (ch2 L660). That participation gate wants the **same tier machinery** PG-13 wired and PG-03 makes real. A Thread `auth_tier_min` enforced *before* PG-03 lands is the same honest no-op as the join gate; enforced *after* it has teeth. Small but real coupling — argues PG-03 → PG-08.

**AE-A14 — state ownership + convergence.** `ThreadState` lives in `SpaceState` alongside rooms, keyed by `thread_id`, anchored to a `room_id` (mirrors `RoomState`). `thread.resolved`/`thread.archived` are **State Events** → they carry a state key `(EventType, thread_id)` and ride the M8 `derive_resolved` convergence machinery for free. Design confirms the appliers are convergence-clean (idempotent, order-independent) and that `state_key_for_event` gains the thread arms — otherwise concurrent resolve/archive would not converge.

**AE-A15 — Thread.id stays unflavoured (`xgen_uri`), consistent with AppC.** Appendix C marks `Thread.id` as conceptual `xgen_uri` — **no `ThreadXgid` flavour today** (Phase-3 work, per the AppC header note). Recommendation: stay conceptual (a plain hash-derived `xgen://thread/sha256:` string), do **not** add a flavour wrapper in Arc E — that keeps PG-08 off the XGID-retrofit surface and consistent with the documented stance. Design records this explicitly.

**AE-A16 — protocol stays thin (notification = client concern).** ch2 is explicit: the protocol provides the `status` field + event types; **clients implement notification logic**. So Arc E ships the primitive + lifecycle state; per-Room-type behaviour (forum-as-threads, announcement-reply threads, stage companion thread) and notifications are **client/UI-milestone** work. Design states the protocol/client line.

### 2.2 PG-08 verdict

**GAP-CONFIRMED, sized as mechanical breadth.** 3 event types + `ThreadStatus` enum + `ThreadState` in `SpaceState` + appliers (convergence-clean) + `state_key_for_event` arms + validation + builder + tests. Wider surface than PG-03, lower conceptual risk; rides the M8 state path.

---

## §3 — Cross-strand synthesis + scope

### 3.1 Sequence: PG-03 → PG-08 (confirmed by grounding)

Three independent grounding facts point the same way:
1. **AE-A3** — PG-03 is the keystone under Arc D's already-wired tier-gate; landing it closes a matched pair.
2. **AE-A13** — Thread's `auth_tier_min` participation gate is only meaningful once a real assertion tier exists; PG-03-first means PG-08's gate is honest, not decorative, on landing.
3. **Risk ordering** — PG-03 is the conceptually heavy strand (SignedPrimitive, crypto, dead-code activation); doing it first while the context is fresh is lower-risk than interleaving.

### 3.2 Role model spun out (AE confirms §0.2)

Appendix C `class Role` is a real documented primitive, but its closure (custom roles, `permissions[]`, `position`, `Guest`, the cascade) is **the continuation of Arc D's privilege model**, not primitive-identity completion, and it sits **downstream** of the tier/assertion substrate. Keep it out of Arc E; it earns its own arc.

### 3.3 Sizing summary

| Strand | Shape | Surface | Conceptual risk |
|---|---|---|---|
| PG-03 TrustAssertion | focused depth | 1 SignedPrimitive struct + canonical sign/verify + reg steps 5–7 + `assertion_tier_of` rewire + synthetic issuer | **high** (crypto, keystone) |
| PG-08 Thread | mechanical breadth | 3 events + enum + state + appliers + state-key + validation + builder + tests | **low** (rides M8 path) |

---

## §4 — Recommendation to the design beat

**Scope:** Arc E = PG-03 + PG-08. Role model OUT (later arc).
**Sequence:** PG-03 (block) → PG-08 (block) → close (D-074).

**Open design calls — to become `AE-D#` locks (arc-local, D-069):**

- **AE-D1 (TA wire authority + drift).** Lock ch3 §3.8.4 as the wire-authoritative schema; reconcile AppC (`valid_until` not `expires_at`; `jurisdiction` in/out per AE-A7; `claims`/`signature` restored). Field order per §3.8.5.
- **AE-D2 (TA canonical sign/verify).** Reuse `canonical.rs` (§3.2.4 rules); Ed25519 verify against `issuer`. Confirm the SignedPrimitive seam matches Event/Node.
- **AE-D3 (registration steps 5–7 activation extent).** How much of §3.8.5's 7 checks goes live now (sig + expiry + `tier_verified` claim are pure-local and should; step 7 "required claims by Node policy" + step 1 "issuer registered on this Node" need the trusted-list — decide minimal trusted-list vs config stub). Honest Tier-1/Local-Node posture per AE-A9.
- **AE-D4 (TA scope line).** Struct + validation + synthetic issuer IN; live Auth Module service OUT (AE-A8). State it.
- **AE-D5 (Pass-2 framing).** Arc-E-local vs formal XGID Pass 2 (AE-A6) — recommend arc-E-local, cross-referenced.
- **AE-D6 (Thread event set).** Lock the 3 events + `ThreadStatus` (open/resolved/archived) + which roles may resolve/archive (permission gate; reuse Arc-D `check_permission`).
- **AE-D7 (Thread state + convergence).** `ThreadState` in `SpaceState`; `state_key_for_event` arms; appliers convergence-clean against M8 `derive_resolved`.
- **AE-D8 (Thread flavour stance).** Stay conceptual `xgen_uri`, no `ThreadXgid` (AE-A15).
- **AE-D9 (Thread auth_tier_min gate).** Reuse the PG-13 join-gate path; narrow-not-widen vs parent Room (AE-A13). Honest no-op until PG-03 + a real module.
- **AE-D10 (protocol/client line).** Per-Room-type Thread behaviour + notifications → client/UI milestone, not Arc E (AE-A16).

**No DECISIONS.md change at open** (AE-D# arc-local, D-069; promotion eval at close). **No code** until the design locks and Joe approves the runbook.

---

**Audit complete (v1.0).** Two GAP-CONFIRMED strands grounded; scope (PG-03 + PG-08, Role out) and sequence (PG-03 first) recommended; ten open design calls handed to the design beat. Per Rule 0 / D-071 / D-074.
