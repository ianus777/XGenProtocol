# XGen Protocol — Protocol Gap Audit
> **Status**: COMPLETED  
> Version: 1.8  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Method, sources, vocabulary

### 0.1 Purpose

A single audit producing **one ranked gap register** at a milestone-selection point (post-J-232, Storage-Engine CLOSED, suite 1024/0/2). Two walks feed the register:

- **Part A — spec-vs-as-built** (§1): walk the **normative surface** of the spec docs and test each promise against live code. A gap = the written protocol promises something the code does not deliver, **or** the code does something the spec never sanctioned.
- **Part B — multiparty-readiness** (§2): what is structurally missing before M8/M9 — the three banked tensions + concrete M8/M9 prerequisites.

The §3 register is the **next-milestone candidate menu**, severity-ordered. This is a standalone Track-1 task doc, **not** a milestone open — it earns a JOURNAL line only if it closes into an acted-on recommendation (per Rule 0 / D-074).

### 0.2 Walk depth (Joe-locked)

**Part A = exhaustive: every normative MUST / SHOULD / MUST NOT / SHALL line**, not chapter-level claims. This forces A1 into a **chunked multi-checkpoint walk**; chunk boundaries below are provisional pending first-read normative density.

### 0.3 Sources

**Spec docs (`docs/`, `_en` canonical):**
ch0_content · ch1_philosophy · ch2_architecture · ch3_specification · ch4_implementation · ch5_protocol · ch6_client_design · appendix_a … appendix_l.

**Code (workspace crates):**
`xgen-common` (GPL-2.0) · `xgen-core` (GPL-2.0) · `xgen-node` (BSL 1.1) · `xgen-client` (BSL 1.1) · `xgen-store-sqlite` (GPL-2.0-or-later).

**Method per normative line:** extract the claim → locate the responsible code via `Select-String` across `xgen-*/src/**/*.rs` → read the site → assign a verdict + evidence pointer (`file:line` or doc §). Not a full code re-read; targeted against each claim.

### 0.4 Verdict vocabulary

- **NO-GAP** — spec promise is honoured in code; evidence cited.
- **GAP-CONFIRMED** — spec promises X, code does not deliver X (or delivers ¬X). The core finding.
- **SPEC-DRIFT** — doc and code diverge but neither is clearly wrong (doc ahead of / behind as-built); a doc-sweep item, not a code defect.
- **NEEDS-DESIGN** — the gap is real but closing it requires a design beat, not a patch.
- **N/A** — normative line is aspirational / out-of-current-scope (recorded, not chased).

### 0.5 Gap-ID scheme

Every non-`NO-GAP` finding gets a stable ID **`PG-NN`** (append-only, never renumbered across checkpoints). Register row columns: **ID · source (doc § or file:line) · normative claim (paraphrase) · verdict · severity · closing cost (one line)**.

Severity: **S1** breaks a stated protocol guarantee / safety or trust property · **S2** functional gap, no guarantee broken · **S3** doc-drift / cosmetic · **S4** aspirational / future.

### 0.6 Progress tracker (A1 chunks — provisional)

| Chunk | Sources | Status |
|---|---|---|
| A1-1a | ch0–ch1 (TOC + philosophy) | ✅ done (4 gaps) |
| A1-1b | ch2 (architecture) | ✅ done (4 gaps) |
| A1-2 | ch3 specification + ch4 implementation | ✅ done (4 gaps) |
| A1-3 | ch5 protocol + ch6 client design | ✅ done (0 new gaps) |
| A1-4 | appendices A–L | ✅ done (0 new; 4 reinforcements) |
| A2   | Part B readiness | ✅ done |
| A3   | register + recommendations | ✅ done |

(Chunk boundaries re-confirmed on first read; high normative density may split a chunk.)

---

## §1 — Part A: Spec-vs-as-built

> Filled chunk-by-chunk per §0.6. Each subsection: normative lines extracted → verdict → `PG-NN` on any non-`NO-GAP`.

### §1.1 — ch0–ch2 (A1-1)

#### §1.1.1 — ch0 (TOC) + ch1 (philosophy) — A1-1a ✅

**Normative surface.** ch0 is a TOC (no norms). ch1 carries **0 uppercase RFC-2119 keywords** — it states norms in prose (lowercase `must`/`cannot`/`never`/`by design`); 75 prose-normative lines. The large majority are **philosophical design-intent** (enshittification resistance, governance independence, the Matrix/Discord/Kyberia comparisons) → **N/A (aspirational)**, recorded in bulk, not chased line-by-line. **~8 lines translate to code-testable claims**; verdicts below.

| ch1 line | Claim (paraphrase) | Verdict | Note |
|---|---|---|---|
| L107 | No proxy trust — admin cannot grant a tier; user must self-verify | NO-GAP (tentative) | `AuthTier` ordered enum exists (`tiers.rs`); cumulative/self-verify semantics confirmed at ch3 §3.11 (A1-2) |
| L195 | Tiers hierarchical + cumulative (T3 ⊇ T1,T2) | NO-GAP (tentative) | same — `AuthTier 1..4` ordered; `>=`-gate usage confirmed at ch3 §3.11 |
| L210/L215 | Protocol owns the **trust assertion**; one vanilla credential always present | **GAP-CONFIRMED → PG-03** | `TrustAssertion` struct **does not yet exist in Rust** (deferred to XGID Pass 2, `xgen-common/.../flavours.rs:36`) |
| L725/L924 | Identity model must handle non-human (AI) verified identity | NO-GAP | `state.ai_operator_delegate` / `state.ai_operator_revoke` EventTypes backed (`wire.rs:142/145`); operator-framing complete |
| L727 | Federation must support **jurisdictional namespacing** | **NEEDS-DESIGN → PG-04** | only a `jurisdiction: String` field on tier config (`tiers.rs:114`); no federation-level namespace by jurisdiction |
| L856/L858 | Spec must prohibit any **central identity-aggregation point** | deferred | structural — resolved in ch2 walk + Part B §2.2 |
| L868 | Bridge modules carry own trust-tier declaration; client must visually distinguish bridged content | N/A (future) | no bridge built; carried to Part B §2.3 |
| L879–L883 | **Uniform deletion Event** mechanism, tier-graded propagate/confirm/audit | **GAP-CONFIRMED → PG-02** | only `MessageRedact` exists (`exchange.rs:703`); no identity/content GDPR-erasure event across federation |

**ch0 doc-drift.** ch0's appendix index lists **A–I**; disk has **A–L** (J/K/L unindexed). Ch5 marked "pending" — matches the 401 B `xgen_ch5_protocol.md` (consistent, not a surprise). → **PG-01**.

#### §1.1.2 — ch2 (architecture) — A1-1b ✅

**Normative surface.** 167 prose-normative lines (0 RFC-2119 uppercase). Code-checked in batches by cluster. Verdicts on the high-yield clusters below; the remaining inventory (pass 2) is listed at the end so nothing is dropped.

**Checked — verdicts:**

| ch2 line | Claim | Verdict | Evidence |
|---|---|---|---|
| L2148/L2357/L2547 | E2E **encryption boundary** — federated Nodes get encrypted Events, never decrypted server-side ("protocol guarantee") | **GAP-CONFIRMED → PG-05** | MLS wire-types (`wire.rs:159–174`) + `chacha20poly1305` dep + client *interface* exist, but RFC 9420 crypto **explicitly deferred to Phase 3** (`client_mls.rs:9`). Self-declared deferral, not hidden drift |
| L1304 | Tier 1 deletion = a `message.delete` Event propagated | **SPEC-DRIFT → PG-06** | code implements `message.redact` (`wire.rs:32`), not `message.delete`; also no tier-graded propagate/confirm (feeds PG-02) |
| L798/L936 | `auth_tier_min` — min tier to join Room/Space, **enforced at protocol level** | **SPEC-DRIFT → PG-07** | tier-gate primitive exists (`tiers.rs` error 3030 "assertion tier below required tier" + ordered `AuthTier` + module `accepted_tiers`); spec field name `auth_tier_min` ≠ code "required tier"; join-path wiring confirmed at ch3 §3.11 / ch4 |
| L27 | Every primitive carries a `meta_atts` field | NO-GAP (Event) | `wire.rs:376 pub meta_atts: Option<Value>` + canonical sort (`canonical.rs:35`); full-primitive coverage spot-checked at appendix I (A1-4) |
| L819/L1007/L1795 | Room / Space / Identity are **never deleted** | NO-GAP | no `room.delete`/`space.delete`/`identity.delete` event types exist — the absence *is* the guarantee |
| L1593 | Key rotation cryptographically chained | NO-GAP (event) | `system.key_rotation` EventType (`wire.rs:70`); chain-validation logic confirmed at ch3 |
| L2012/L2352 | Received Events / announcements signature-verified | NO-GAP (tentative) | verify infra in `signing.rs` + `directory.rs`; exact per-Event ingest step confirmed at ch3/ch4 |
| L1372/L1390/L1391 | Trust Assertion uniform structure / `expires_at` / `issuer` | (covered by PG-03) | struct absent in Rust; fields are ch3 detail |

**Pass 2 — remaining inventory resolved:**

| ch2 line | Claim | Verdict | Evidence |
|---|---|---|---|
| L620–685 | **Thread primitive** — lifecycle (open/resolved/archived), one Room, `auth_tier_min` | **GAP-CONFIRMED → PG-08** | no `thread.*` EventType, no `ThreadStatus` enum (only "threaded" code-prose). Thread is 1 of the 4 core primitives (Space/Room/Event/**Thread**) — specified, unimplemented |
| L866/872/2097/2106 | State resolution deterministic / convergent / causal | NO-GAP (causal) | `prev_events` DAG + cycle/root validation (`graph.rs`), topo-sort under D-076 v1.1; full convergent state-resolution-v2 completeness → Part B M8/M9 readiness |
| L905/1018/1042 | Space owned by members (Owner role); Node hosts not owns; no-hostage | NO-GAP | `SpaceRole::Owner` (`event_trace.rs:43`, `state.rs:179`); non-ownership structural (federated, hash-derived id); ownership-transfer verb confirmed at ch3 |
| L1827/L1865 | Presence Space-scoped, protocol-enforced | NO-GAP (tentative) | presence referenced (13 sites) — **corrects the earlier "may be unimplemented" flag**; Space-scoping enforcement confirmed at ch3/ch6 |
| L405/411 | DM Space always invite-only / never discoverable | NO-GAP | distinct `state.dm_space_create` primitive (`wire.rs:36`); non-discoverability structural |
| L795/1163 | ids hash-derived, permanent, never reassigned | NO-GAP | XGID sha256-derived ids (core of the XGID system) |
| L2128 | Event buffering / replay on peer reconnect | NO-GAP | federation `pending_queue` buffering (prior arcs) |
| L2142 | Room-scoped federation (member-gated) | NO-GAP | bidirectional-nodes arc (D-075); `apply_fanout` member-gated |
| L1463 | Revoked module voids its Trust Assertions | (covered by PG-03) | `TrustAssertion` struct absent; revocation semantics are ch3 detail |
| L381 | Unknown event types stored/forwarded/ignored (not dropped) | **NEEDS-VERIFY** | parser returns `None` on unknown (`wire.rs:256`); key-level forward-compat (`wire.rs:453`); whether unknown *types* are relayed vs dropped → confirm at ch4 ingest/forward path |
| L948/974/988 | Role cascade Space→Room; Room override narrows; tier/role independent | **NEEDS-VERIFY** | fixed role set (Owner/Admin/Mod/Member) present; per-Room permission *override* not evidenced → confirm at ch3 §permissions |

**Two NEEDS-VERIFY carried to A1-2** (resolved in ch3/ch4, not logged as PG rows until confirmed): unknown-event forward-vs-drop; per-Room role/permission override. **ch2 walk complete — 4 gaps (PG-05–08).**

### §1.2 — ch3–ch4 (A1-2)

ch3 sub-chunks: **A1-2a** Phase 1 §3.0–§3.8 · **A1-2b** Phase 2 §3.9–§3.16 · **A1-2c** ch4. ch3 carries **real RFC-2119**: 168 uppercase MUST/SHALL + 266 prose.

#### §1.2.1 — ch3 Phase 1 §3.0–§3.8 — A1-2a 🟡

**Pass 1 — validation-pipeline cluster** (also resolves the two ch2 carried verifies):

| § / line | Claim (MUST) | Verdict | Evidence |
|---|---|---|---|
| §3.2 L648 | Unknown-`type` Event MUST be **stored + propagated** to peers (not dropped) | **GAP-CONFIRMED → PG-09** | `EventType` is a **closed enum** (no `Unknown(String)`); parser returns `None` on unknown (`wire.rs:256`); structural **step 6 rejects** unknown types (`validation.rs:112 UnknownEventType`). Apply-layer `_ => Ok()` ignore (`state.rs:476`) is unreachable for truly-unknown types. Breaks forward-compat/extensibility (also ch2 L381) |
| §3.2 L741–745 | `prev_events`: root empty, non-root ≥1, ≤max, no cycle, hold-on-unseen | NO-GAP | `graph.rs` (`EmptyPrevEvents`, `MAX_PREV_EVENTS=10`, non-root≥1, `HeldPending` on unseen) — D-076 territory |
| §3.2 L756/776/782 | Ordered validation; sig-verify (step 12) after structural (steps 1–7) | NO-GAP | `validation.rs` 7 structural steps; sig verification placed later (matches spec order) |

**Carried-verify resolution:** ch2 #1 (unknown-event forward) → **resolved as PG-09 (gap)**. ch2 #2 (per-Room role/permission override) resolves at §3.7 (pass 2).

**A1-2a pass 2 — AI / auth / Space-Room / wire-value MUSTs:**

| § / line | Claim (MUST) | Verdict | Evidence |
|---|---|---|---|
| §3.6.10 L2003 | `is_ai` immutable; reject `identity.update` changing it (3041) | NO-GAP | `registration.rs:80` `AiFlagImmutable` → 3041 |
| §3.6.10 L2026/2052/2017 | AI capability flags **hard-enforced at event time**; AI **MUST NOT** be Space owner | **GAP-CONFIRMED → PG-10** | declaration + `is_ai` immutability backed (`registration.rs`), but per-flag runtime enforcement (dm_initiate/spontaneous_post) + AI-not-owner reject **not found** in core/node event path after 2 targeted greps; final confirm at ch4 (§1.2.3) |
| §3.8 L2951 | Reject **expired** Trust Assertion | NO-GAP | `tiers.rs` `AssertionExpired` + `verify` (`:171`) |
| §3.8 L2941 | Tier-1 registration 7 checks all MUST pass | NO-GAP (tentative) | §3.6.4 acceptance pipeline incl. step 8 (AI shape) |
| §3.7 L969 | Per-Room **override narrows** a Space-role's permissions | **NEEDS-VERIFY** | step-13 per-Room permission *enforcement* exists (`exchange.rs:198`); per-Room *override/narrowing* of Space roles not evidenced → resolve at §3.7 prose |
| §3.7 L2634/2648 | Node validates temperature float 0–1 + `warm<hot<fiery` | **NEEDS-VERIFY** | temperature meta_atts documented (`wire.rs:504/508`); Node-side range validation not confirmed |
| §3.1 L486 | Reject standard base64 (`+`,`/`,`=`) | NO-GAP | `encoding.rs:12` / `flavours.rs:103` |
| §3.1 L504–510 | Reject unknown `major`; accept any `minor` | NO-GAP | `handshake.rs:365` major-mismatch → None |
| §3.1 L373/465 | Reject `null` / float / unsafe-int | NO-GAP (structural) | typed-serde rejects null/float for typed fields; `timeout.rs:143` fractional → BAD_ARGUMENT |

**Lower-risk formatting MUSTs spot-confirmed NO-GAP (structural / typed-serde, not individually deep-traced — low gap-yield):** field-name stability (L336/338) · `meta_atts` key rules + silent-ignore-unknown (L351/357) · transport auth nonce-match (L1020) · keypair-no-regen (L1585) · announcement expiry/self-cert (L1636/1661).

**A1-2a complete — 2 gaps (PG-09, PG-10).** 2 NEEDS-VERIFY carried to §3.7-prose / ch4: per-Room role override; temperature range validation.

#### §1.2.2 — ch3 Phase 2 §3.9–§3.16 — A1-2b ✅

| § / line | Claim (MUST) | Verdict | Evidence |
|---|---|---|---|
| §3.9 L3179 | Hold Events with unsatisfied deps in pending buffer | NO-GAP | `PendingBuffer`/`HeldPending` (prior arcs) |
| §3.9 L3208 | Reconstruct state snapshot from Event log on startup | NO-GAP | `rehydrate_space_from_store` (`runtime.rs:291`) + `replay_spaces_from_dir` + engine-replay (J-228/J-232) |
| §3.10 L3328/3330/3416/3481 | E2E: KeyPackage pool ≥3, expire-discard, epoch-advance-on-leave, no-E2E client indicator | (under PG-05) | MLS interface-only, RFC 9420 crypto deferred to Phase 3 — all fold into PG-05; no-E2E indicator is a ch6 client item |
| §3.11 L3580–3880 | Tier 2–4 operator obligations (ISO 27001, KYC/AML, audit trail, data localisation) | N/A (institutional) | not core-team-implementable (ch2 L1320–1334; only Tier 1 ships); data-localisation ties to PG-04 |
| §3.11 L3799/3836 | Node append-only protocol audit log; never auto-deleted | NO-GAP | `protocol_audit` writer + PAL-D1 (`admin_ops.rs:3252`) — M6 A6 |
| §3.12 L4190 | Space **Migration** protocol (source MUST NOT delete DB immediately) | **GAP-CONFIRMED → PG-11** | wire types defined (`state.space_migrate`, `migration.request/propose/accept`, `wire.rs:81–98`) but full migration **subsystem/handlers deferred** (named future arc; `migrate-start` deferred at M7C). Confirm handler-absence at ch4 |
| §3.13 L4324/4340/4387 | Identity replication; reject lower `update_version`; expire 90d | NO-GAP (tentative) | `identity.replicate`/`_ack` (`wire.rs:119`) + `pending_identity_replication` (`state.rs:51`); `update_version` reject-lower confirm light |
| §3.14 L4543/4545/4581 | Bootstrap directory signature-verify; 1h freshness; 7d TTL | NO-GAP | `directory.rs` signature verify (M6 bootstrap-client) |

**A1-2b complete — 1 gap (PG-11).** E2E cluster folded into PG-05; Tier 2–4 obligations N/A.

#### §1.2.3 — ch4 implementation — A1-2c ✅

ch4 is the reference-implementation chapter; its 18 MUSTs describe as-built code and all map to already-verified behaviour: canonical serialiser (L440), frame codec validate + no-recover (L455/461), keypair-existence-check (L486), explicit sig verify Ok/Err (L521), structural steps 1–7 (L553), loopback-bind on `local_node` (L595), state-reconstruct-on-startup (L629), `membership.join` for unknown Space held (L655), audit retention (L1751). → **NO-GAP across ch4** (matches code by construction).

**Carried-verify resolutions:**

| Carried item | Verdict | Evidence |
|---|---|---|
| AI capability hard-enforcement (PG-10) | **GAP confirmed (final)** | broader ch4-pass grep still finds no per-flag (`dm_initiate`/`spontaneous_post`) or AI-not-owner enforcement at event-validation time; `capability.rs` is NodeAnnouncement caps, not AI-identity caps — PG-10 no longer tentative |
| Per-Room role/permission override | **GAP-CONFIRMED → PG-12** | step-13 per-Room EventType permission *enforcement* exists (`exchange.rs:198`), but no per-Room *override/narrowing* of Space-role permissions (no `RoomPermission`/override structure); ch2 L948/969 mechanic ("Moderators can't post in announcements") not evidenced |
| Temperature range validation | NO-GAP (minor drift) | `clamp_temperature` (`types.rs`) + `node_policy.rs:61` [0,1] threshold present; minor SPEC-DRIFT — spec §3.7 L2634 says *reject* out-of-range, code *clamps*; low-impact, noted not logged |

**A1-2 (ch3 + ch4) complete — 4 gaps (PG-09–12).**

### §1.3 — ch5–ch6 (A1-3) ✅

**ch5 (Open Protocol)** — 401 B, Version 0.0, header-only. The multi-party-adoption chapter is **intentionally unwritten** (ch0 marks it "pending"). No normative surface → **N/A (future chapter)**. Minor doc-hygiene nit: header reads `Status: ACTIVE` while empty at v0.0; `PENDING` would fit the status vocab better.

**ch6 (Client Design)** — 100 KB but only **3** uppercase normative lines (mostly UI/design prose). The reference Client/UI is a **future milestone** (roadmap: UI follows multiparty tests), so ch6's client-side normatives are **specified-but-not-yet-built by design**, not spec-vs-code drift:
- L775 configurable text-substitution list (UI) → N/A future-UI
- L1392 non-blocking implementations (compute guidance) → N/A future-UI
- L1412 mention rails OR'd (impl detail) → N/A future-UI
- (ch3 L3481 no-E2E client indicator also lands here — under PG-05 + future-UI)

**A1-3 complete — 0 new gaps.** Structural note for §4: the entire reference **Client/UI surface is specified but unbuilt** (the UI milestone) — a known roadmap item, carried to Part B / §4 as context, not a hidden gap.

### §1.4 — appendices A–L (A1-4) ✅

**Non-normative (N/A):** A (Why-own-protocol), B (Funding), E (Lifecycle states), H (Test records) — 0 MUSTs, explanatory. **As-built references (NO-GAP, spot-checked):** F (CLI, 4 MUSTs) · G (Log convention, 12) · J (XGID, 1) · K (M6 verb reference, 0) · L (EventStore, 2) — describe shipped surfaces. J/K/L are the three **unindexed in ch0** (= PG-01).

**Gap-relevant findings:**

| Appendix | Finding | Effect |
|---|---|---|
| I L75 | "Event `type` MUST be one of the known strings" | **Internal spec contradiction with ch3 §3.2 L648** (unknown types MUST be stored + propagated). Code follows Appendix I's closed-set view. → **PG-09 needs a spec decision first** (which statement wins), not just a code change |
| I L489/490 | `AiCapabilities.extra` map MUST survive round-trip | NO-GAP — `registration.rs:609` preserves unknown keys (key-level forward-compat works; distinct from event-type-level PG-09) |
| I L591 | Temperature `0.0<warm<hot<fiery≤1`, NaN rejected, `is_valid()` | NO-GAP — ordering validation backed; refines the A1-2c minor-drift note (only single-value clamp-vs-reject differs) |
| C L59/67 | Schema diagram defines `TrustAssertion` + `Thread` classes | reinforces **PG-03** + **PG-08** — both documented as first-class primitives, unimplemented in Rust |
| D §3.3 | "Right-to-Erasure Problem in Federated Systems" section; "Node cannot selectively delete from DAG without breaking [chain]" | reinforces **PG-02** — append-only DAG vs GDPR Art.17 is an **acknowledged-but-unsolved** problem (= Part B §2.1) |

**A1-4 complete — 0 new gaps; 4 reinforcements (PG-02/03/08/09).** **Part A (A1) complete — 12 gaps (PG-01–12).**

---

## §2 — Part B: Multiparty-readiness

> What is structurally missing before M8 (multiparty improved pass) / M9 (multiparty redesign).

### §2.1 — Banked tension: GDPR right-to-be-forgotten in federated, no-anonymity system

**Status: acknowledged, unsolved (= PG-02).** The append-only DAG forbids selective Event deletion without breaking the hash chain (Appendix D L101); identity is a public key replicated across replica Nodes (no-anonymity pillar); Appendix D §3.3 names the problem explicitly. Today only `message.redact` (display redaction) exists — no uniform, tier-graded erasure Event that propagates + confirms + audits across federation (ch1 L879–883). **Three sub-problems unsolved:** (a) DAG immutability vs Art.17 erasure; (b) federated propagation of a delete with delivery confirmation (Tier 3+); (c) erasure of a *replicated identity record* across Nodes that each hold a self-certifying cache. **Readiness verdict:** a deletion/erasure design arc is a prerequisite for any Tier-2+ multiparty deployment claiming GDPR posture; not blocking for Tier-1 multiparty tests. **Closing this = PG-02 (S1).**

### §2.2 — Banked tension: no-anonymity vs. government identity demands

**Status: architecturally addressed; residual legal exposure by design.** The protocol's structural defense (ch1 L856/858) holds in code: identity = pubkey, no central identity registry, each Node's registry is *a cache of self-certifying records* (Appendix D L49) — there is no single subpoena target. **But** the no-anonymity pillar means every identity is attributable, and a Tier-4 (government) module verifies *legal* identity; a government can compel a *specific* Node operator (in its jurisdiction) to surrender the member records that Node holds. The protocol neither prevents nor can prevent this — it's the operator's jurisdiction, by design. **Readiness verdict:** not a code gap; resolution is a *spec-stance* statement (the protocol prohibits central aggregation — confirm ch3/spec explicitly prohibits it per ch1 L858) + the **jurisdictional-namespacing** mechanism (PG-04) that lets Spaces declare and contain their legal domain. **Closing the actionable part = PG-04 (S2) + a spec-prohibition clause.**

### §2.3 — Banked tension: Discord-bridge trust-model collision

**Status: unbuilt; future module.** No bridge code exists. The collision is real: Discord permits anonymity; XGen requires verified identity, so bridged Discord content cannot carry an XGen Trust Assertion. ch1 L866/868 + ch2 L369 set the answer — the bridge is *one module among many*, carries its **own trust-tier declaration**, and the client **MUST visually + technically distinguish** bridged content from verified XGen content. **Readiness verdict:** not an M8/M9 blocker; it's a self-contained future module arc that depends on (a) the module-framework (D-085, now landed) and (b) a client "bridged/unverified" indicator (future-UI, sibling to the no-E2E indicator). Lower priority than the multiparty core.

### §2.4 — Concrete M8 / M9 structural prerequisites

M8 = multiparty improved pass; M9 = multiparty redesign. Against Part A:

| Prerequisite | State | Source |
|---|---|---|
| **State-resolution convergence** under concurrent/conflicting State Events at scale (the Matrix state-res-v2 problem) | **DONE (M8, 2026-06-03)** — the seven-layer `resolve()` was built-but-unwired (SR-F1); M8 wired it onto the node apply path (`derive_resolved` + the SR-D1 ingest gate) and proved convergence in-process (C1), at the runtime seam (C2) and two-node (C3). M9 not triggered | was NO-GAP (causal); the convergence layer is now closed via the D-071 Phase-0 audit (`tasks/STATE_RESOLUTION_AUDIT.md`) |
| **Forward-compat / unknown-event relay (PG-09)** | **gap** — closed `EventType` enum rejects unknown types; multiparty across protocol/feature versions would partition federation | PG-09 (S1) — **prerequisite** (version-skew is intrinsic to multiparty) |
| **Per-Room permission override (PG-12)** | **gap** — enforcement point exists, override layer missing; multiparty permission semantics need it | PG-12 (S2) — prerequisite if Rooms vary permissions |
| **Thread primitive (PG-08)** | **gap** — unimplemented; multiparty forum/thread semantics depend on it | PG-08 (S2) — prerequisite *iff* Threads are in the multiparty scope |
| Identity replication | **built** | NO-GAP (§3.13) |
| Federation propagation reliability (pending-buffer, topo-sort, bidirectional nodes) | **built** | NO-GAP (prior arcs, D-075/076) |
| Durable EventStore + engine substitution | **built** | NO-GAP (J-228/J-232) |

**Open scoping question (from project memory) resolved here:** the deferred hardening / client / standalone arcs are **not** multiparty prerequisites — only PG-09 (forward-compat) is a hard prerequisite; PG-08/PG-12 are scope-conditional. The **state-resolution convergence audit is the proper M8/M9 Phase-0 gate** (D-071), and it is the one item this gap-audit cannot pre-empt (it needs its own subsystem audit).

**Scope-fold resolved (M8 close, 2026-06-03).** The Phase-0 audit ran (`tasks/STATE_RESOLUTION_AUDIT.md`); M8's locked scope was **membership-core convergence** (SR-D4). **PG-08, PG-10 and PG-12 were NOT folded into M8** — the audit §5 confirmed none is an M8 prerequisite; all three stay open as independent Wave-3 arcs (D / E). M8 therefore closed **no PG-NN** (convergence was the NO-GAP-causal row above, not a catalogued gap), so the §5 register count is unchanged by design. M9 held as a contingency — not triggered, all three proof levels (C1 in-process · C2 runtime seam · C3 two-node) passed clean.

---

## §3 — Consolidated ranked gap register

> Append-only. Populated across all checkpoints; re-sorted by severity at A3.

| ID | Source | Claim | Verdict | Sev | Closing cost |
|----|--------|-------|---------|-----|--------------|
| PG-01 | ch0 appendix index vs `docs/` | TOC lists appendices A–I; disk has A–L | SPEC-DRIFT | S3 | Add J/K/L rows to ch0 TOC (one edit) |
| PG-02 | ch1 L879–883 | Uniform tier-graded deletion Event (GDPR erasure) across federation | GAP-CONFIRMED | S1 | Design erasure event + federation propagate/confirm/audit (large; = Part B §2.1) |
| PG-03 | ch1 L210/215; `flavours.rs:36` | `TrustAssertion` first-class primitive | GAP-CONFIRMED | S2 | Implement `TrustAssertion` struct (XGID Pass 2 scope) |
| PG-04 | ch1 L727; `tiers.rs:114` | Federation jurisdictional namespacing | NEEDS-DESIGN | S2 | Design jurisdiction namespace in federation addressing |
| PG-05 | ch2 L2148/2357; `client_mls.rs:9` | E2E encryption boundary (MLS) — server-side never decrypts | GAP-CONFIRMED | S1 | Implement RFC 9420 MLS crypto (deferred to Phase 3; large) |
| PG-06 | ch2 L1304 vs `wire.rs:32` | Deletion event named `message.delete` | SPEC-DRIFT | S3 | Reconcile name `message.delete` ↔ `message.redact` in ch2 (or rename event) |
| PG-07 | ch2 L798/936 vs `tiers.rs` | Room/Space `auth_tier_min` field name | SPEC-DRIFT | S3 | Reconcile `auth_tier_min` ↔ "required tier"; confirm join-path wiring (ch3/ch4) |
| PG-08 | ch2 L620–685 | **Thread** primitive (lifecycle, one-Room, tier-min) | GAP-CONFIRMED | S2 | Implement Thread primitive + `thread.*` events + status lifecycle (core primitive, sizable) |
| PG-09 | ch3 §3.2 L648; `wire.rs`/`validation.rs:112` | Unknown-`type` Event stored + propagated (forward-compat) | GAP-CONFIRMED | S1 | Add `EventType::Unknown(String)` (or raw-type passthrough) + relay-not-reject on unknown type; sizable wire change |
| PG-10 | ch3 §3.6.10 L2026/2052 | AI capability flags hard-enforced at event time; AI-not-Space-owner | GAP-CONFIRMED → **NO-GAP** (reclassified Arc D, J-244) | S2 | None — already enforced in `dispatch_event` step 4 (AI-not-owner 3041, dm_initiate 3042; spontaneous_post spec-deferred). Original GAP was a grep-surface error (checked `validate_event`/`capability.rs`); see §5 |
| PG-11 | ch3 §3.12 L4190; `wire.rs:81–98` | Space Migration subsystem (handlers, source-DB-retention) | GAP-CONFIRMED | S2 | Implement migration subsystem behind the existing wire types (named deferred arc) |
| PG-12 | ch2 L948/969; `exchange.rs:198` | Per-Room override narrowing Space-role permissions | NEEDS-DESIGN | S2 | Add per-Room×per-Role permission override (enforcement point exists; override layer missing) |
| PG-13 | ch2 L798; `tiers.rs:142` (no prod caller) | Tier-gate (`verify_tier_assertion`) not wired into Room/Space join path — "enforced at protocol level" unmet | GAP-CONFIRMED | S2 | Wire the gate into membership/join validation (Arc D; no-op under Tier-1-only today, load-bearing at Tier 2–4). Surfaced by PG-07's verify, 2026-06-03 |

**Severity rollup:** **S1 ×3** — PG-02 (erasure), PG-05 (E2E), PG-09 (forward-compat) · **S2 ×7** — PG-03, PG-04, PG-08, PG-10 (reclassified NO-GAP, Arc D), PG-11, PG-12, PG-13 · **S3 ×3** — PG-01, PG-06, PG-07. Register kept in stable PG-ID order (append-only); severity grouping drives §4. *(PG-13 added 2026-06-03 during Arc A execution — surfaced by PG-07's join-path verify; the Part-A narrative count of 12 reflects the original walk, register now 13.)*

---

## §4 — Candidate-milestone recommendations

12 gaps grouped into 9 candidate arcs across 4 **waves** (priority bands — distinct from auth Tiers), ordered for selection.

### Wave 1 — quick + high-leverage (do before M8)

**A. Doc-drift sweep** — PG-01 (ch0 TOC +J/K/L) · PG-06 (`message.delete`↔`message.redact`) · PG-07 (`auth_tier_min`↔"required tier"). All S3, ~1 doc-only commit, zero code risk, clears register noise; confirm the PG-07 join-path wiring while there.

**B. Unknown-Event Forward-Compat** — PG-09 (S1). A hard multiparty prerequisite (version-skew would partition federation) **and** an internal spec contradiction to settle first (Appendix I L75 closed-set vs ch3 §3.2 L648 store-unknown — Joe decides which wins). If "store + propagate" wins: `EventType::Unknown(String)` / raw-type passthrough + relay-not-reject. Self-contained, bounded wire change. **Should precede M8.**

### Wave 2 — the main next milestone

**C. M8 / M9 multiparty — ✅ M8 DONE (2026-06-03).** The planned major arc. Was gated by (i) **B done** (✅) and (ii) its **own state-resolution-convergence Phase-0 audit** (D-071, ✅ `tasks/STATE_RESOLUTION_AUDIT.md`). M8 wired the built-but-unwired seven-layer `resolve()` onto the node apply path + proved convergence (C1 in-process · C2 runtime seam · C3 two-node, J-238–J-240). Locked scope = membership-core (SR-D4); **PG-08 + PG-12 NOT folded** — they stay Wave 3 (arcs D/E). M9 not triggered (held as a contingency if a future scenario surfaces a structural limit).

### Wave 3 — independent S2 design arcs (pick by deployment need)

**D. Permission / enforcement hardening** — ✅ **DONE (Arc D, J-242 open → J-244 close, 2026-06-03).** PG-13 (tier-gate on join) ✅ + PG-12-min (per-Room×per-Role override) ✅; PG-10 reclassified **NO-GAP** (already enforced). The planned **privilege-model arc**. Full first-class Role object model → arc E.

**E. Primitive completion** — ✅ **DONE (Arc E, J-245 open → J-248 close, 2026-06-04).** PG-03 (`TrustAssertion` SignedPrimitive + `validate_assertion`) ✅ + PG-08 (Thread primitive + `thread.*` events + lifecycle, rides M8 convergence) ✅. The full first-class Role object model + per-Room-type Thread behaviour stay spun-out (privilege-model continuation arc / client-UI milestone).

**F. Migration subsystem** — ✅ **DONE (Arc F, J-251 open → close, 2026-06-04).** PG-11. Wire types + core were built; the node driver was wholly absent — Arc F wired it (source + destination halves, cutover applier flipping `home_node` on both Nodes, retention, dormant admission, operator verb, two-node e2e).

**G. Jurisdictional namespacing** — ✅ **DONE (Arc G, J-249→J-250).** PG-04 + the spec clause prohibiting central identity aggregation (ch3 §3.6.7 MUST-NOT). Addressed the actionable part of the no-anonymity-vs-government tension: a Space declares a set-once `jurisdiction`; a Node MAY contain federation by it; the protocol forbids any central identity-aggregation point.

### Wave 4 — large, known-deferred

**H. E2E encryption (MLS)** — 🔷 **INTERFACE-LOCKED (Arc H, J-254 open → C1 J-255 → C2 J-256 → close)** — PG-05 (S1). Operationalised the Phase-2 epoch scheme onto the live `message.*` path + the `enc:` v2 envelope (per-message `CK`, a D-088 amendment) + the content-blindness proof + KeyPackage lifecycle + epoch-advance. **Server-blindness is real + proven on the wire.** Deferred to **D3** (parallel, timing-open per D-066): real RFC 9420/openmls crypto behind the now-operational interface + concurrent-commit resolution; the no-E2E client indicator is a ch6/UI item (Round-2). Cascade: `D-088 content-erasure → PG-05 real crypto → D3`.

**I. GDPR erasure / right-to-be-forgotten** — ✅ **DESIGN-LOCKED (Arc I, J-253; D-088; design-only, impl deferred behind PG-05/Arc H)** — PG-02 (S1). Large design arc (DAG-immutability vs Art.17, federated delete-confirm, replicated-identity erasure). Gates Tier-2+ deployments; not Tier-1-multiparty-blocking.

### Recommended sequence

**A** (cheap cleanup) → **B** (unblocks multiparty) → open **C**'s state-resolution Phase-0 audit. **D–I** are pickable by deployment priority; **H** and **I** are the heavy long-horizon arcs.

---

**Audit complete (v1.0).** Part A — 12 gaps, exhaustive walk ch0–appendix-L · Part B — 3 tensions + M8/M9 prerequisites · ranked recommendations above. This doc is the input to milestone selection; per Rule 0 / D-074 it earns a JOURNAL line only when a recommendation is acted on. Status stays ACTIVE until then.

---

## §5 — Gap closure tracker

> Severity-sorted live tracker (added v1.1). **Status legend:** ⬜ OPEN · 🟢 IN-PROGRESS · ✅ DONE · 🔷 INTERFACE-LOCKED (mechanism + interface shipped + proven; a named-deferred strengthening — e.g. real crypto — remains, tracked elsewhere). Mark a gap DONE only when its fix has landed + pushed; note the closing arc/commit in-place when it does.

| ID | Sev | Verdict | Gap | Source | Closing cost | Status |
|----|-----|---------|-----|--------|--------------|--------|
| PG-02 | S1 | DESIGN-LOCKED / impl-deferred (D-088, PG-05-**real-crypto**/D3-gated) | Uniform tier-graded deletion/erasure Event across federation (GDPR) | ch1 L879–883 | Design erasure event + federated propagate/confirm/audit (large; arc I) | ⬜ OPEN |
| PG-05 | S1 | INTERFACE-LOCKED / impl-deferred (D3-gated) | E2E encryption boundary — server never decrypts | ch2 L2148/2357; `client_mls.rs:9` | Real RFC 9420/openmls crypto behind the now-operational wire interface (= D3, parallel/timing-open per D-066) | 🔷 INTERFACE-LOCKED (Arc H, J-254 open → C1 J-255 `ee06168` → C2 J-256 `ffda2af` → close — the Phase-2 epoch scheme operationalised onto the live `message.*` path: `e2e_encryption` Space flag [set-once/default-OFF] + `state.mls_group_init` + the `enc:` v2 **envelope** [per-message random `CK` wrapped under the epoch key — a D-088 amendment, AH-D1] + Node DS blind-route + the **content-blindness proof** [5 assertions incl. the threat-defended erasability invariant] + KeyPackage pool [≥3/expiry/single-use, 5001/5002] + epoch-advance on `mls.commit` [commit-race fenced to D3]. **Server-blindness is now real + proven on the wire**; only the crypto *strength* upgrade [RFC 9420] is D3. Honest dormant-but-correct: no production MLS client drives it yet [C1 Finding 1].) |
| PG-09 | S1 | GAP-CONFIRMED | Unknown-`type` Event stored + propagated (forward-compat) | ch3 §3.2 L648; `validation.rs:112` | `EventType::Unknown` + relay-not-reject (arc B) | ✅ DONE (Arc B — C1 `9bf57d1` + C2 `e0a1972`; Appendix I §I.2 reconciled) |
| PG-03 | S2 | GAP-CONFIRMED | `TrustAssertion` first-class primitive | ch1 L210/215; `flavours.rs:36` | Implement struct (XGID Pass 2; arc E) | ✅ DONE (Arc E / C1, J-246 — `TrustAssertion` SignedPrimitive: struct + canonical/sign/verify + `from_assertion` + full 7-check `validate_assertion` wired into `accept_registration`; registration steps 5–7 now enforced [3004/3005/3006/3010/3011/3030]; honest dormant-but-correct — empty trusted-list + Local-Node bypass, real teeth at a live T2–4 module) |
| PG-04 | S2 | NEEDS-DESIGN | Federation jurisdictional namespacing | ch1 L727; `tiers.rs:114` | Design jurisdiction namespace in addressing (arc G) | ✅ DONE (Arc G, J-249 open → J-250 close — C1 `SpaceState.jurisdiction: Option<String>` set-once at create [ch3 §3.7.3 schema + AppC]; C2 `FederationPolicy.allowed_jurisdictions` + pure `jurisdiction_permits` AND-composed with `policy_permits` at both enforcement sites [strict undeclared-denied] + `--allowed-jurisdiction` operator surface + ch3 MAY clause; close ch3 §3.6.7 central-aggregation **MUST-NOT**. Honest dormant-but-correct — live hook, no-op until an operator sets a policy; active data residency NOT delivered, AG-D8) |
| PG-08 | S2 | GAP-CONFIRMED | Thread primitive (lifecycle, one-Room, tier-min) | ch2 L620–685 | Implement Thread + `thread.*` events + status (arc E) | ✅ DONE (Arc E / C2, J-247 — `thread.create`/`.resolved`/`.archived` + `ThreadStatus` + `ThreadState` in `SpaceState`; appliers + `state_key` arms ride M8 `derive_resolved` [resolved-vs-archived converges]; narrow-not-widen + AE-D9 participation tier gate; resolve/archive `ChangeInfo`-gated. Per-Room-type behaviour + notifications → client/UI milestone, AE-D10) |
| PG-10 | S2 | **NO-GAP** (reclassified) | AI capability hard-enforcement + AI-not-Space-owner | ch3 §3.6.10 L2026/2052 | None — already enforced in `dispatch_event` step 4 (Arc D grounding) | ✅ NO-GAP (Arc D, J-244) |
| PG-11 | S2 | GAP-CONFIRMED | Space Migration subsystem (handlers, source-DB retention) | ch3 §3.12 L4190; `wire.rs:81–98` | Implement handlers behind existing wire types (arc F) | ✅ DONE (Arc F, J-251 open → close — C1 `transition()` + `apply_space_migrate` cutover applier [home_node flip, idempotent, AF-D2 self-protecting gate]; C2 node driver: 12-msg dispatch + per-session `MigrationState` + transport + EventStore bridge [`ensure_store`/`append`/`rehydrate_space_from_store`] + retention [AF-D5, no auto-teardown] + federation-notify via applier-reuse [AF-D8a] + `migration initiate` operator verb; two-node e2e flips `home_node` on both Nodes. Wire 6009 `migration_authority` added [AF-D2, superseded the C2-guessed 6007]) |
| PG-12 | S2 | NEEDS-DESIGN | Per-Room override narrowing Space-role permissions | ch2 L948/969; `exchange.rs:198` | Add per-Room×per-Role override (arc D) | ✅ DONE (Arc D / C2, J-243 — PG-12-**min**: overrides on the fixed-role enum via `state.room_update`; full first-class Role model → arc E) |
| PG-13 | S2 | GAP-CONFIRMED | Tier-gate not wired into Room/Space join path | ch2 L798; `tiers.rs:142` | Wire `verify_tier_assertion` into join validation (arc D) | ✅ DONE (Arc D / C1, J-243 — wired onto `MembershipJoin`, honest Tier-1 no-op; meaning gated on PG-03 + a real T2–4 module) |
| PG-01 | S3 | SPEC-DRIFT | ch0 TOC lists appendices A–I; disk has A–L | ch0 vs `docs/` | Add J/K/L rows to ch0 TOC (arc A) | ✅ DONE (Arc A, v1.3) |
| PG-06 | S3 | SPEC-DRIFT | Deletion event named `message.delete` vs code `message.redact` | ch2 L1304 vs `wire.rs:32` | Reconcile the name (arc A) | ✅ DONE (Arc A, v1.3 — ch2→`message.redact`) |
| PG-07 | S3 | SPEC-DRIFT | `auth_tier_min` vs code "required tier" | ch2 L798/936 vs `tiers.rs` | Reconcile name; confirm join-path wiring (arc A) | ✅ DONE (Arc A, v1.3 — ch2→`auth_tier`; wiring verify spun off PG-13) |

**Open: 1 / 13 · Done: 10 · Interface-locked: 1 · NO-GAP-reclassified: 1** (done: PG-01/06/07 Arc A, PG-09 Arc B, PG-12/13 Arc D, PG-03/08 Arc E, PG-04 Arc G, PG-11 Arc F; PG-10 reclassified NO-GAP at Arc D close; **PG-05 interface-locked at Arc H**). **Open: PG-02 only** (GDPR content-erasure implementation — gated behind PG-05 *real crypto* = D3). Register 13 (PG-13 spun off from PG-07's verify). Arc letters map to §4 candidate-milestone groupings.

**Arc H (E2E encryption / MLS) closed 2026-06-04 (J-254 open → C1 J-255 → C2 J-256 → close).** **PG-05 closes INTERFACE-LOCKED, not ✅ DONE** (D-065 — the honest shape, distinct from PG-02's design-only DESIGN-LOCKED: Arc H shipped working code + a *proven* server-blindness guarantee, only the crypto-strength upgrade is deferred). The Phase-2 epoch scheme (built-but-unwired, AH-A1) was operationalised onto the live `message.*` path; the **`enc:` v2 envelope (per-message random `CK` wrapped under the epoch key) is a D-088 amendment** (AH-D1, promoted into D-088) that gives crypto-shred its per-message erasure granularity. **The cascade is now named and tracked: `D-088 content-erasure → PG-05 real crypto → D3/openmls`** — PG-02's content-erasure build stays gated behind PG-05's *real* crypto (D3), not merely behind Arc H; the **identity-orphan half of D-088 remains PG-05-independent** (could ride the Tier-1 auth-module rebuild). With PG-05 interface-locked, the **last Round-1 D-071 arc is closed**; PG-02 (impl) is the sole remaining open gap, gated on D3.

**Arc C / M8 (state-resolution convergence) closed 2026-06-03** — the Wave-2 main arc. It closed **no PG-NN**: convergence was the §2.4 NO-GAP-causal prerequisite, not a catalogued gap, so the 9/13 count is unchanged. Its Phase-0 audit confirmed **PG-08, PG-10 and PG-12 are NOT folded into M8** — all three remain ⬜ OPEN as Wave-3 arcs (D / E). Next candidate per §4: arc D (enforcement-hardening — PG-10/12/13) or arc E (primitives — PG-08/03).

**Arc D (enforcement-hardening / privilege-model) closed 2026-06-03 (J-242 open → J-244 close).** Wired the tier-gate onto join (PG-13, honest Tier-1 no-op) + per-Room×per-Role overrides on the fixed-role enum (PG-12-min, via `state.room_update`). **PG-10 reclassified NO-GAP** — the original GAP-CONFIRMED was a grep-surface error (it checked `validate_event`, which excludes AI checks by design §7.7, + `capability.rs`); AI-not-owner (3041) + dm_initiate (3042) enforcement lives in `dispatch_event` step 4, and spontaneous_post is spec-deferred — all conform. Now **6/13 done, PG-10 NO-GAP, 6 open**. The full first-class Role object model (custom roles, `permissions[]`, `position`, `Guest`) + `TrustAssertion` (PG-03, which gives PG-13's gate real teeth) cluster into **arc E**. Next candidate per §4: arc E (primitives — PG-08/03) or a heavier arc (F migration, G jurisdictional, H E2E, I GDPR erasure).

**Arc E (primitive completion) closed 2026-06-04 (J-245 open → J-248 close).** Shipped the two documented-but-unimplemented core primitives: **PG-03 `TrustAssertion`** (C1, J-246) — the third SignedPrimitive (Event/Node/TA); struct + canonical Ed25519 sign/verify + `TrustAssertionXgid::from_assertion` + the full §3.8.5 7-check `validate_assertion` wired into `accept_registration` (registration steps 5–7, dead since Phase 1, now enforced; codes 3004/3005/3006/3030 reused + **3010/3011** added — the design's guessed 3006/3007/3008 collided with existing variants); honest dormant-but-correct (empty trusted-list + Local-Node bypass). **PG-08 `Thread`** (C2, J-247) — `thread.create`/`.resolved`/`.archived` + `ThreadStatus` + `ThreadState` in `SpaceState`; appliers + `state_key` arms ride M8 `derive_resolved` (resolved-vs-archived converges via the shared `thread.status` key); narrow-not-widen + the AE-D9 participation tier gate (real teeth post-PG-03); resolve/archive `ChangeInfo`-gated. The full first-class Role object model stays spun-out to a later privilege-model arc; per-Room-type Thread behaviour + notifications → a client/UI milestone (AE-D10). Now **8/13 done, PG-10 NO-GAP, 4 open** (PG-02 erasure · PG-04 jurisdictional · PG-05 E2E · PG-11 migration). All AE-D# stay arc-local (D-069 — none a cross-arc invariant). Next candidate per §4: a heavier arc (F migration · G jurisdictional · H E2E · I GDPR erasure).

**Arc G (jurisdictional namespacing) closed 2026-06-04 (J-249 open → J-250 close).** Closed **PG-04** + the §4-G spec clause. The keystone (AG-A2): the federation containment chokepoint already existed (`policy_permits`, two-site-enforced, default-permit) — Arc G added a restrictive *dimension*, not a new gate. **C1** (protocol half): `SpaceState.jurisdiction: Option<String>`, set-once at create (no applier / no `state_key` arm — rides M8 via `PartialEq` for free), read by all three constructors (DM ⇒ `None`, AG-D4); `build_space_create_event` gains a `jurisdiction` param; ch3 §3.7.3 schema field + AppC Space class. **C2** (implementation half): `FederationPolicy.allowed_jurisdictions: Option<Vec<String>>` (additive serde, restrictive-only, default permit-all) + pure `jurisdiction_permits` **AND-composed** with `policy_permits` at both enforcement sites (outbound `federation_session.rs`, inbound `app.rs`); **strict undeclared-denied** (an undeclared Space fails a restrictive set, mirroring `allowed_spaces`); `--allowed-jurisdiction` operator surface on `federation set-policy`; ch3 **MAY** clause. **Close** (doc-only): ch3 §3.6.7 central-identity-aggregation **MUST-NOT** (promotes ch1 L858's implication to normative). Honest dormant-but-correct (D-065): the field is real + convergence-safe from C1; the hook is live but a no-op until an operator declares `allowed_jurisdictions`; active data residency (geo-pinned storage, legal attestation) is **not** delivered — operator/Tier-2+ infra, fenced out (AG-D8). All AG-D# stay arc-local (D-069). Now **9/13 done, PG-10 NO-GAP, 3 open** (PG-02 erasure · PG-05 E2E · PG-11 migration). Next candidate per §4: a heavier arc (F migration · H E2E · I GDPR erasure).

**Arc I (GDPR erasure / right-to-be-forgotten, design-only) closed 2026-06-04 (J-253).** Resolved **PG-02** architecturally (D-088): crypto-shred content over PG-05 / orphan identity binding / monotonic tier-graded permission — protocol binds endpoints (T1 max-erasable, T4 no-record-destruction), Auth Module declares the T2/T3 interior on the Trust Assertion (forward-extensible descriptor). Implementation design-locked + deferred behind PG-05 (Arc H); identity-orphan half is PG-05-independent. **PG-02 stays OPEN as implementation** (not flipped to DONE — D-065). Appendix D §3.3 superseded (tombstone → crypto-shred); ch1 compliance philosophy was already consistent.

**Arc F (Space Migration subsystem) closed 2026-06-04 (J-251 open → close).** Closed **PG-11** + §4-F. The keystone (AF-A1): the wire layer (12 msg types + `state.space_migrate`) AND the core (`xgen-core/src/migration/` — state machine + transfer + verification + a pure end-to-end test) were **built but the node driver was wholly absent** (zero dispatch in xgen-node) — so Arc F was *wiring*, not design. **C1** (core completion): pure `transition()` sequence guard (`MigrationMsgKind` local enum, CP-1) + `apply_space_migrate` cutover applier (AF-D1/D2 — flips the authority anchor `home_node` source→dest; idempotent; self-protecting `sender == home_node` gate; causally-terminal singleton, no `state_key` arm). **C2** (node driver): `migration_driver.rs` source (`run_source_migration`, spawned by the operator verb) + destination (`handle_migration_incoming`, off the `handle_connection` first-message branch) halves; 12-msg dispatch through `transition()`; transport via `send_migration`; EventStore bridge (CP-4 — `ensure_store` fresh per-Space store + `append` + `rehydrate_space_from_store`, engine-agnostic, no separate SQLite cache-rebuild); cutover commits `state.space_migrate` (flips source) + ships it to dest (flips dest) + pushes to peers (CP-2/AF-D8a — applier reuse, `migration.federation_notify` courtesy); retention (AF-D5 — source keeps its store, teardown operator-gated only); dormant admission (AF-D6 — accept-unless-hosting, 6003/4/5 dormant); `migration initiate` operator verb (CP-3, audited). Required the `state.space_migrate` wiring into `validate_event`'s Node-authored path (the C1 grounding finding) — new wire **6009 `migration_authority`** (AF-D2; **superseded the C2-guessed 6007**, which collided with the spec's §3.12.11 `migration_verification_failed` — the Arc-E renumber pattern). Two-node e2e flips `home_node` on **both** Nodes + proves retention + AF-D2 stale-source rejection + already-hosted (6002) rejection. Honest dormant-but-correct (D-065): mechanism is real + e2e-tested; destination admission 6003/4/5 dormant, source teardown operator-gated. All AF-D# stay arc-local (D-069). Now **10/13 done, PG-10 NO-GAP, 2 open** (PG-02 erasure · PG-05 E2E). Next candidate per §4: H E2E (PG-05) or I GDPR erasure (PG-02).
