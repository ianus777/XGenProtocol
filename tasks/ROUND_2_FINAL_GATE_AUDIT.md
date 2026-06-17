# Round-2 Final Pre-UI Gate — Whole-Codebase Coherence Audit (Pass 2)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-17  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

This is the **Round-2 final pre-UI gate** — the single whole-codebase audit that gates
UI in the locked post-M12 chain (`M12 CLOSED → this gate → UI → Streams`, ROADMAP). It is
**Pass 2** of the two-round audit strategy (locked 2026-06-04): a delta, **not** a re-run.

**Pass 1 is the inherited baseline — superseded by reference, not mutated.**
`tasks/ROUND_2_AUDIT.md` (Status COMPLETE, v1.3, 2026-06-05) certified **GO** for the
codebase *as it stood at that time* — an M8 / M9 / multiparty-era tree (suite 1153/0). Its
chain ended "M10 → UI" and it explicitly put **M10 out of scope** (unbuilt then). This Pass
2 covers what that pass structurally could not: the **three arcs that shipped after it** —
**M10** (Auth Module reference set, J-375), **M11** (`self` thread / D-021, J-378), **M12**
(attachments — blob store + delete, federation fetch, erasure, the domain-10 band, Appendix
F v1.9→1.12, D-088/D-093; J-379→J-389). Pass 1 stays a clean COMPLETE record of what it
certified; **this document supersedes its §6 verdict by reference** for the current
(post-M12) tree.

Like Pass 1, this is a **GATE, not an arc**: it builds no fixes and audits **shipped code
only**. Output = this audit doc + the consolidated findings register (§5) + the UI go/no-go
verdict (§6). Any finding that needs work spawns its own arc or feeds the UI build;
per-finding routing is **Joe's lock**, not pre-decided here.

**Grounding posture (D-065).** Every load-bearing claim below was grounded to `file:line`
against the live tree on `main` (HEAD `5f990ad`, tree clean, in-suite **1463/0**) — verified
in code, not taken from the JOURNAL. Where a claim rests on a journal/design assertion that
was not re-read in code, it is marked as such.

**Out of scope / cannot close.** The **D3-fenced** surfaces stay fenced — they are *named*,
not reopened: the DAG-resident erasure residue (event existence + plaintext descriptor key +
text; crypto-shred = D3, the M12.4 WE6 boundary) and production-grade openmls/RFC-9420
maturity (PG-05 interface-locked, real crypto = D3). **Streams** is the post-UI plane, not in
this gate's scope. The identity-orphan half of D-088 is unbuilt (see §3.2 forward-note).

---

## 2. Surfaces swept (post-M12 delta)

Crates: `xgen-common` · `xgen-core` · `xgen-node` · `xgen-client` · `xgen-auth-module`
(new in M10). Plus `docs/` canonical (ch3 / ch6 / Appendix F) + `DECISIONS.md` +
`tasks/MP_findings.md` record coherence. *(Node-side hooks cited below as `app.rs:NNNN` — the federation / redact / erasure surfaces — are `xgen-node/src/app.rs`; the `xgen-client/src/app.rs` cites are the e2e harness paths.)*

Five sweep axes, each grounded against `main`:

1. **New-surface audit** — M10 / M11 / M12 as-built (the auth module ref set; the self
   thread; the full attachment chain: blob crypto, chunked-WS transfer, federation
   β-multiplex fetch, erasure).
2. **Cross-arc coherence redux** — the interactions per-arc Phase-0s could not see now that
   all three arcs exist (attachments × erasure × federation × auth-tier × self-thread; the
   first production `Retention` reader × the existing tier-gate; the D-088/D-093 erasure-fate
   invariants end-to-end).
3. **Wire-code register integrity** — fold the domain-10 band (10001–10004) into the
   register; re-check collisions/orphans across all bands; re-confirm the Pass-1 60xx
   disposition; check whether M10 discharged the Pass-1-checkpoint RC-F-01 item.
4. **Carried-forward register re-confirm** — Pass-1 items (R2-F01, R2-F07, R2-F09) + their
   current disposition against the post-M12 tree.
5. **Routed-open + carry-over sweep** — MP-F12 / MP-F13 / MP-F16; the M12.3 federation
   throwaway `pending_fetches`; the M11 `create-dm-space` Appendix-F gap; DECISIONS.md
   numbering/ordering housekeeping debt.

---

## 3. Per-axis findings (grounded)

### 3.1 New-surface audit — M10 / M11 / M12 as-built

**M10 (Auth Module reference set) — as-built, clean.** The `xgen-auth-module` binary exists
(`xgen-auth-module/src/{lib,main}.rs`); it issues Tier-1 reference assertions and (M10.3) a
parameterized T2–T4 **mock** (`issue --tier <N>`; N∈{2,3,4} self-labels `module_kind = mock`
+ grounded TTL + tier-appropriate `erasability`, **T4 → `Retained`**, `main.rs:43`). The node
gate **live-reads** the registry: `AssertionPolicy { trusted_issuers, accepted_tiers_by_issuer,
… }` is composed from `AuthModuleRegistry::{trusted_issuers, accepted_tiers_by_issuer}` at the
gate (`app.rs:841` "`trusted_issuers` is sourced LIVE from the registry"), so a `revoke` bites
without restart (`xgen-auth-module/tests/end_to_end.rs:119-127`). `accepted_tiers` is
enforcement-bearing → **`3032 assertion_tier_unauthorized`** (restrictive-only, distinct from
the node-floor `3030 tier_mismatch`). **M10 adds no Space-DAG event type** and no new
state-mutation arm.

**M11 (`self` thread, D-021) — as-built, clean.** The entire applier delta is the
**guard-at-construction** at both DM constructors: skip seeding the self-invite when
`invitee == creator` (`state.rs:425` `from_dm_space_create`, `state.rs:541`
`from_dm_space_create_node`). Constructor-only; `apply_join` already short-circuits
`AlreadyMember`; the still-built self-targeted auto-invite is swallowed under DM constraints
(documented inert residue). The `self` verb is wired across **all four** D-092 dispatch arms
(`main.rs:267` CLI · `app.rs:1016` run-path · `batch.rs:435` · `aicontrol.rs:447`) →
`ops::self_open` (create-if-absent, auto-resolves the session identity). The never-federates
wall (`DmFederationNotAllowed`) is inherited unchanged. **M11 adds no new event type and no
new state arm.**

**M12 (attachments) — as-built, clean.** The full feature is present and self-documented:
- Content-addressed, content-blind `BlobStore` (`blob_store.rs`) — `put`/`get`/`contains` +
  the M12.4 **`delete`** (idempotent: `Ok(true)` removed / `Ok(false)` absent-no-op / `Err`
  on malformed-ref or I/O); `get` verifies the content-address on read (W3).
- Per-blob crypto (`encryption/blob.rs`) — fresh per-blob key + nonce per call.
- `Descriptor` + `build_message_file_event` / `build_message_redact_event`
  (`exchange.rs:918-983`); both ride the existing signed-envelope (`ExchangeError`) path and
  **emit no new `ExchangeError` wire code**.
- Erasure spine (M12.4): `resolve_redact_erasure` (`runtime.rs:574`, pure decision fn) +
  `author_is_retained` (`runtime.rs:623`) + the `apply_redact_erasure` hook (`app.rs:3004`,
  fired in the Accepted arm at `app.rs:3324`).

**No new state-mutation conflict domain (verified in code, D-065).** `message.file` and
`message.redact` have **no `apply_event` arm** — both fall through to `_ => Ok(())`
(`state.rs:655`). The node's redact work is a **side-effect in `process_inbound`**, not an
applier; the redact event itself is a valid signed event with no state mutation. The Pass-1
§3.1 finding ("conflict domains are non-overlapping") therefore **holds unchanged** — M10 /
M11 / M12 added zero new conflict domain.

### 3.2 Cross-arc coherence redux

**F2b (M12's first production `Retention` reader) × M10's module-policy descriptor —
COHERENT end-to-end.** M10.1 landed the descriptor chain in `xgen-common/src/trust_assertion.rs`:
`TrustClaims::module_policy() → ModulePolicy.erasability → Erasability.retention →
Retention::{Erasable, Retained}` (`trust_assertion.rs:162-229`). M12.4's `author_is_retained`
reads **exactly that chain** (`runtime.rs:623-641`):
`claims.module_policy().and_then(|p| p.erasability).and_then(|e| e.retention)` →
`matches!(…, Some(Retention::Retained))`. This is precisely the consumer the AI-D8 descriptor
was authored to feed — M10.1's own doc-comment says "the *enforcement* of the gradient (and
the default for an absent descriptor) is the deferred D3-gated consumer." **F2b is that first
consumer.** Both reads are **lenient and stack consistently**: `module_policy()` returns
`None` on malformed input, and `author_is_retained` maps `None`/`Erasable`/absent-record →
`false` → erase, only explicit module-declared `Retained` → `true`. This is exactly D-088's
"T1 / no-module = max-erasable" default. **The reader reads what M10 writes; no drift.**

**F2b × the existing tier-gate (`verify_tier_assertion`) — no interference.** Both read the
same `TrustClaims`, but different fields: the tier-gate (PG-13, Arc D / M10) reads
`tier_verified` / `tier`; `author_is_retained` reads `module_policy().erasability.retention`.
Both are pure reads, no mutation. Independent.

**Erasure × admission × federation — side-effect-not-admission + origin-agnostic, verified.**
The `apply_redact_erasure` hook fires **after** the redact is admitted + persisted + before
fanout (`app.rs:3314-3337`), gating only the **blob-delete** side-effect (M12.4-D3 / D-093 c2)
— the redact event always converges regardless of outcome. The hook is **origin-agnostic**
(runs for both `LocallySubmitted` and `ReceivedViaFederation`), so a federated redact erases a
peer's cached copy (WE4). On `RefusedRetained` (original content author `Retained`), bytes are
kept and a `10004 erasure_refused_retained` side-channel goes to a local redactor via
`reject_signal` (the redact still converged). **D-093 c3 (no shared physical copy across
erasure-fate) is honored with zero storage reshape:** `blob_ref = hash_uri(ciphertext)` is
per-send-unique by construction because `encrypt_blob` mints a fresh key+nonce every call
(`blob_store.rs:181-191` documents the forward-constraint; `encryption/blob.rs`), so
`delete(blob_ref)` erases only the redacted reference's own copies. *(See R2G-F02 for the one
forward-looking edge this rests on.)*

**Attachments × Arc-G jurisdiction containment — clean by construction, verified in code.**
The M12.3 federation blob-fetch (`federation_fetch_blob`, `app.rs:2855`) resolves the Space's
federated holders (`home_node ∪ federation_nodes`) **intersected with B's *live* federation
sessions** (`app.rs:2846`) and **injects `OutboundMsg::BlobFetchRequest` into the existing
peer session** (`app.rs:2915`). It does **not** call `connect_url` — it never re-dials. The
only `connect_url` callers (federation establish `app.rs:3857`, reconnect `reconnect.rs:379`)
pass through `policy_permits` + `jurisdiction_permits` (`app.rs:3163` / `:3194`). Therefore a
jurisdiction- or policy-blocked node has **no live session → cannot be a blob holder**:
Arc-G jurisdiction containment and the federation policy gate are preserved **transitively**
through the attachment-fetch path. **Positive cross-arc result.**

**Attachments × self-thread (M12 × M11) — clean.** The `self` thread is M12's intended front
door (intra-home multi-device, never federation). A redact on a self-thread attachment runs
the same `resolve_redact_erasure` path against the (single-home) self-DM store; WE4
(federated erasure) is structurally moot for a thread that never federates. No interaction
bug.

**Forward cross-arc note (not numbered — the mechanism is unbuilt).** The **identity-orphan**
half of D-088 (orphan the pubkey↔person binding) is **not built** (J-253/J-389: PG-05-independent,
could ride a future Tier-1 auth-module rebuild). When it lands, F2b's `author_is_retained`
reads the target author's stored assertion — and an orphaned author whose assertion was
removed would resolve **lenient → `false` → erase**. For a `Retained` (T4) author this would
wrongly permit erasure of legal-held content. The D-088 invariant **"T4 = no record
destruction"** must guard the orphan path (a T4 identity must not be orphanable, or the
assertion's retention must survive the orphan). Flagged for whoever builds identity-orphan;
**not a current defect** (no orphan mechanism exists today).

### 3.3 Wire-code register integrity

**Domain-10 band (M12) — clean, fully typed, no collisions.** `BlobError::to_wire_code`
(`blob_store.rs:93-101`) maps the entire band: **10001** `blob_hash_mismatch` (M12.1) · **10002**
`blob_too_large` (F6/M12.2) · **10003** `blob_unavailable` (F3/M12.3) · **10004**
`erasure_refused_retained` (F2b/M12.4). Domain 10 = 10000–10999 is **clear of every allocated
band** (the highest live band is 60xx), so no collision is possible with the pre-M12 register.
Each code is RC-F-01-re-grepped at its arc and unit-tested.

**RC-F-01 — DISCHARGED by M10 (the Pass-1-checkpoint item closes).** The J-357 Round-2
checkpoint left RC-F-01 OPEN → routed to M10: ch3 double-defined 3010/3011 (§3.6.5 Arc-E live
vs §3.11.7 higher-tier). M10 reconciled it (M10.1-D1 + M10.3 C2), verified in code +ch3:
- §3.6.5 / `registration.rs:122-125`: **3010** `assertion_identity_mismatch` + **3011**
  `assertion_claims_insufficient` stay Arc-E's (live, emitted, tested).
- The old §3.11.7 `auth_tier_insufficient` **folded into the live 3030 `tier_mismatch`**
  (`registration.rs:124`; same code `auth::tiers` emits); `kyc_verification_pending` **re-homed
  to 3031** (`registration.rs:151`, reserved/no-emitter).
- **3032** `assertion_tier_unauthorized` added (M10.3 C2) — and the close-catch where the
  M10.3 draft's 3012 collided with the reserved §3.11.7 **3012 `watchlist_match`** was resolved
  by renumber (J-367): `watchlist_match` keeps 3012 (ch3 §3.11.7 L3861), the auth-tier-authz
  code moved to 3032. ch3 §3.11.7 L3866-3868 now reads 3030 live / 3031 reserved / 3032 live.

The 30xx band is internally consistent: 3041/3042/3043 (AI/eject, `exchange.rs:132-134`),
3044/3045 (invite, `pending.rs`), **3046** (timestamp, M9.1, `exchange.rs:139`). No new
collision from M11 (reuses `DmFederationNotAllowed`) or M12 (domain-10 only).

**60xx migration band — Pass-1 disposition (R2-F02) holds.** Pass 1 doc-reconciled it (J-259:
ch3 §3.12.11 dormant/target annotation; the code renumber `6010/6011`-vs-`MIG_6010/6011` was
**deferred to when the migration subsystem activates**). M10 / M11 / M12 did **not** touch
migration. The disposition re-confirms unchanged. *(See §4 R2-F02 / R2-F07.)*

### 3.4 Routed-open + carry-over sweep

- **MP-F2-followon — PARTIALLY discharged (a precision the Pass-1 checkpoint conflated).**
  The J-357 checkpoint framed "RC-F-01 = the already-named MP-F2-followon → M10." That is true
  for the **auth-band / tier-code half** (the 3010-3032 reconcile above — discharged). But
  MP-F2-followon also has a **separate, non-auth half**: the **7 unmapped event-validation
  variants** that all emit `error_code = 4000` (generic) with the reason in the message string
  (`MP_findings.md:170`; client map at `xgen-client` `aicontrol.rs:88`). That half is
  **state-resolution/validation-band, not auth-band — M10 (auth module) did not touch it, so it
  remains OPEN.** → **R2G-F01.** (Note: MP-F5/J-335 already made the reject *batch-observable*
  as a field — the reason carries the meaning — so this is wire-code **precision**, not
  loss-of-information.)
- **MP-F12** (departed-signer post not re-dispatched, `MP_findings.md:369`) — ROUTED to its own
  home (J-346), LOW breadcrumb; untouched by M10/M11/M12. **Re-confirm open.**
- **MP-F13** (production identity → home-node discovery; the `home_node` = WS-URL vs pubkey
  namespace violation, J-278/J-347) — routed to an identity-replication / federation-endpoint
  arc; the two deferred multiparty rows MP-C-06 / MP-C-16 await it. **M10 closed but did NOT
  discharge MP-F13** (it is not auth-module-scoped). **Re-confirm open.** Not UI-blocking (it
  gates node-home migration, not a UI surface).
- **MP-F16** (`federation_initiate` advertises `config.node.listen` raw, not the
  `--port`-corrected `effective_endpoint`; `admin_ops.rs:1784`, `MP_findings.md:427`) — ROUTED
  to the same federation-endpoint arc cluster, LOW (production); harness-cleared; production
  inconsistency remains; untouched. **Re-confirm open.**
- **M12.3 throwaway `pending_fetches`** (`admin_ops.rs:1825-1830`) — CONFIRMED present + flagged
  in-code as a named M12.3 boundary; a client-miss fetch routed over an admin-`federation
  initiate` session serves a graceful `10003` until the scheduler re-establishes with the shared
  registry (self-heals). **Re-confirm open, S4.**
- **`create-dm-space` Appendix-F gap** — CONFIRMED **already self-recorded** (Appendix F
  Session 5 note, L1307: "the underlying `create-dm-space` verb remains undocumented in this
  appendix — a pre-existing gap, out of M11 scope"). `self` / `fetch` / `redact` **are**
  documented (Appendix F v1.12, F.0.4 L104-105). → **R2G-F04** (carry the already-named gap).
- **DECISIONS.md numbering/ordering debt** — SUBSTANTIATED: **D-030 appears twice** (L1833
  "system service" + L2241 "runtime file placement") and **D-031 appears twice** (L1806 "MLS" +
  L2285 "config reference"); ordering is non-monotonic (D-089 trailing at L3739; the
  D-021..D-061 block scrambled). Substantive content intact; purely numbering/ordering hygiene.
  → **R2G-F03.**

---

## 4. Carried-forward register (Pass-1 items — current disposition)

| Pass-1 ID | Pass-1 disposition | Post-M12 disposition (this gate) |
|----|----|----|
| **R2-F01** (client/node resolution divergence) | CLOSED J-264 (A-pure client re-derive); residual **A+thin-fetch escalation flagged-UNBUILT** (reachable only in a federated multi-home Space at the 3/5a/5b conflict class; node authoritative; client = local projection). | **Holds — disposition unchanged.** None of M10/M11/M12 newly reaches the divergent class: the `self`-thread is single-home (never federates → the cross-home 3/5a/5b class is structurally impossible on it); attachment-fetch is a content-addressed blob pull, not a state-resolution site; the auth-tier gate is node-side. The residual A+thin-fetch is the **one client-correctness item the UI build inherits** — a re-pointer, not a new finding (UI rendering of federated multi-home member/authorship under concurrency reads the same client projection; node stays authoritative). |
| **R2-F07** (Arc-F/Arc-G "Round-2-homed" carry-ins: migration sibling-drift + deferred Arc-G federation-block) | S4 — absorb; expand against close notes when a fix-arc opens. | **Holds — open-as-absorbed.** Untouched by M10/M11/M12. Ties to R2-F02 (the 60xx code-renumber is deferred to migration activation). The M12.3 federation-fetch added a federation surface but **rides the already-policy/jurisdiction-gated session** (§3.2 positive result) — it does **not** widen the Arc-G federation-block surface. |
| **R2-F09** (multi-device seam — AH-D4 epoch-advance, no own `state_key`; PULLED at Pass-1 as D3-gated, UI-prototype-motivated) | ⤴ PULLED (2026-06-05): not a UI blocker; relocated to a future multi-device arc, to be motivated by the UI prototype exercising device add/remove. | **Holds — strengthened as anticipated.** The D3-gated epoch-advance seam is unchanged. But **M11 (`self`-thread = explicitly multi-device, "reachable from any client authenticated as the user") and M12 (cross-device attachment fetch — the M12.1 witness is literally a second-same-identity-client fetch) now provide the concrete multi-device surfaces the PULL named as the motivator.** The UI prototype now has real multi-device surfaces to motivate the arc. Still **not a UI blocker** (the epoch seam stays D3-gated; the self-thread + same-identity-fetch surfaces work today, node-resident). |
| R2-F02 / F03 / F04 / F05 (doc-housekeeping, closed J-259) | ✅ DONE | **No regression.** 60xx renumber-on-activation still deferred (R2-F02); M12 did not touch the band. |
| R2-F06 (operator terminology) | ✅ DONE (zero-rename, J-266) | **No regression** — M10/M11/M12 introduced no new "operator" sense; D-082's classifier still holds. |

---

## 5. Consolidated findings register

Severity: S1 (critical) · S2 (significant) · S3 (moderate) · S4 (minor).
Status: 🟪 OPEN. Per-finding routing is **Joe's lock** — recommendations below are not
dispositions.

| ID | Sev | Status | Finding (grounded) | Recommended routing |
|----|-----|--------|---------|-------------|
| **R2G-F01** | S3 | 🟪 OPEN | **MP-F2-followon partially open.** M10's RC-F-01 reconcile discharged the auth-band/tier-code half (3010/3011 kept Arc-E, `auth_tier_insufficient`→3030, kyc→3031, +3032; verified §3.3). But MP-F2-followon's **other half — the 7 unmapped event-validation variants all emitting generic `4000`** (`MP_findings.md:170`; client map `aicontrol.rs:88`) — is state-resolution/validation-band, **not** auth-band, and M10 did not touch it. The J-357 checkpoint's "RC-F-01 → M10 discharges MP-F2-followon" covered only the auth half. **Doc/observability, not loss-of-information** (MP-F5 made the reject batch-observable; the reason string carries meaning — the gap is wire-code precision). NOT a UI blocker. | Route the event-validation-code-precision half to a future wire-code-precision arc, or fold into the UI/Streams error-surfacing pass. |
| **R2G-F02** | S4 | 🟪 OPEN | **D-093 c3 forward-constraint is enforced only by code-comment + the absence of a feature.** "No descriptor `blob_ref` reuse across erasure-fate" (D-093 c3, the no-shared-physical-copy invariant) holds today *because* `blob_ref` is per-send-unique by construction **and no attachment-forward / re-share / quote-with-attachment feature exists** (`blob_store.rs:188-191` documents the forward-constraint). **The UI milestone is the first place such a feature could land.** A UI attachment-forward that copied a descriptor's `blob_ref` instead of re-encrypting would put two events on one physical copy across erasure-fate — breaking D-093 c3 at the byte layer. **Not a current defect.** | A **named UI-adjacent build constraint** for the UI milestone: attachment-forward/re-share MUST re-encrypt (fresh `blob_ref`), never reuse a descriptor. Hand to the UI build, not a fix-arc. |
| **R2G-F03** | S4 | 🟪 OPEN | **DECISIONS.md numbering/ordering hygiene debt.** Duplicate D-numbers: **D-030 ×2** (L1833 "system service" + L2241 "runtime file placement"), **D-031 ×2** (L1806 "MLS" + L2285 "config reference"). Non-monotonic ordering (D-089 trailing at L3739; the D-021..D-061 block scrambled). Substantive content intact; the recent cross-arc decisions (D-088/D-090/D-092/D-093) are clean. Pure numbering/ordering hygiene. NOT UI-blocking. | A doc-hygiene pass (re-number the two duplicates with suffixed ids or a reconciliation note; optional monotonic sort). Any time. |
| **R2G-F04** | S4 | 🟪 OPEN | **`create-dm-space` undocumented in Appendix F** — the M11 `--invitee <own-id>` "documented floor" is not in the command reference, while `self` / `fetch` / `redact` are (v1.12). **Already self-recorded** as a pre-existing gap (Appendix F Session 5 note, L1307). NOT UI-blocking. | Add a `create-dm-space` row in a doc pass (fold with R2G-F03 housekeeping, or with the first Appendix-F touch). |

**Routed-open re-confirms (not re-numbered — tracked in `tasks/MP_findings.md`):** MP-F12
(departed-signer, own home) · MP-F13 (identity→home-node discovery, identity-replication arc)
· MP-F16 (federation endpoint advertisement, federation-endpoint arc) · the M12.3 throwaway
`pending_fetches` (self-heals). All LOW, none UI-blocking; all untouched by M10/M11/M12.

**Positive results (recorded so the §6 GO is grounded, not asserted):**
- **P1** — domain-10 wire band (10001–10004) clean + fully typed + collision-free (§3.3).
- **P2** — RC-F-01 (the Pass-1-checkpoint item) **DISCHARGED by M10** (auth-band reconcile real + ch3-aligned + tested) (§3.3).
- **P3** — no new state-mutation conflict domain: `message.file`/`message.redact` are `_ => Ok(())` (§3.1).
- **P4** — F2b `Retention` reader × M10 `module_policy` descriptor: **coherent end-to-end** — F2b reads exactly the chain M10 writes, lenient-on-lenient = D-088 T1-max-erasable default (§3.2).
- **P5** — erasure is side-effect-not-admission + origin-agnostic (WE4) + D-093 c3 honored with zero storage reshape (per-send-unique `blob_ref`) (§3.2).
- **P6** — M12.3 federation blob-fetch **rides the already-policy/jurisdiction-gated session, never re-dials** → Arc-G containment + federation policy preserved transitively (verified in code) (§3.2).
- **P7** — M11 `self`-DM guard is constructor-only at both ctors; the never-federates wall is intact (§3.1).

---

## 6. UI go/no-go verdict — **GO**

The protocol surface the UI will couple to — **Spaces / Rooms / membership · attachments
(send / fetch / size-gate / cross-home / erase) · the `self` thread · auth tiers · federation**
— is **coherent, complete, and free of UI-blocking debt.**

- **State-mutation coherence holds** — M10 / M11 / M12 added **zero** new conflict domain
  (`message.file`/`message.redact` are inert appliers; M11 is constructor-only; M10 adds no
  Space-DAG event). The Pass-1 §3.1 non-overlap result carries forward unchanged.
- **Cross-arc interactions check clean** — the first production `Retention` reader (F2b) reads
  exactly what M10's module-policy descriptor writes; erasure gates the side-effect not
  admission (convergence-safe); D-093 c3 is honored with zero storage reshape; the federation
  blob-fetch preserves jurisdiction containment by riding the gated session.
- **The wire-code register is clean** — domain-10 collision-free + typed; the Pass-1-checkpoint
  RC-F-01 double-definition is **discharged by M10**.

**Nothing in the consolidated register gates UI.** R2G-F01 is wire-code precision
(doc/observability, information already present); R2G-F02 is a forward-looking UI **build
constraint** (not a defect); R2G-F03 / R2G-F04 are doc-hygiene. The routed-open items are LOW
and UI-orthogonal.

**Two items the UI build inherits explicitly (named, not blockers):**
1. **R2-F01 residual (the A+thin-fetch escalation, flagged-UNBUILT).** UI rendering of a
   federated multi-home Space's membership/authorship under concurrency reads the same
   client-local projection that can diverge from the node-resolved view at the 3/5a/5b conflict
   class. Node stays authoritative; client = local projection. This is the **one
   client-correctness item** the UI couples to — already dispositioned (escalation flagged, not
   auto-built); the UI prototype is the natural place to decide whether to build it.
2. **R2G-F02 (attachment-forward must re-encrypt).** If/when the UI adds attachment
   forward/re-share/quote-with-attachment, it MUST re-encrypt (fresh `blob_ref`), never reuse a
   descriptor — or D-093 c3 breaks at the byte layer.

**Verdict: GO.** UI may proceed on the clean table the post-M12 chain intends. The four new
findings (R2G-F01..F04, all S3/S4) and the routed-open carry-overs spawn their own
arcs/doc-passes or feed the UI build at Joe's routing; none is a pre-UI blocker. The D3-fenced
surfaces (DAG-residue crypto-shred, production openmls) remain fenced as named — out of UI
scope, to land with their own arcs.

---

## 7. Status & next-active

**Pass 2 (this gate) — audit complete; register §5 tracked.** Supersedes the Pass-1
`tasks/ROUND_2_AUDIT.md` §6 verdict by reference for the post-M12 tree (Pass 1 stays a clean
COMPLETE record). Audit doc committed; **Joe pushes**. Then **Chat opens the triage/design
discussion** on R2G-F01..F04 + the carried/routed items (recommends dispositions) → **Joe
locks** per-finding routing → fix-arcs (if any) or **UI proceeds**.

**Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-389 → `docs/ROADMAP.md` (post-M12 chain) → this
doc → `tasks/ROUND_2_AUDIT.md` (Pass-1 baseline) → `DECISIONS.md` D-088/D-093.**

**Round-2 final pre-UI gate CLOSED COMPLETE — J-390 (Joe, 2026-06-17).** Verdict **GO**, Joe-locked by-recomms. Chat cross-verified the load-bearing hinges in code on `main` (D-065): P3 (`state.rs:655` `_ => Ok(())`, no new conflict domain), P4 (`author_is_retained` reads the `module_policy().erasability.retention` chain M10 writes), P6 (`federation_fetch_blob` `xgen-node/src/app.rs:2855` injects into live sessions + never re-dials; establish/reconnect gated at `app.rs:3163/3194` before `connect_url` `:3857` — Arc-G containment transitive), R2G-F03 (D-030/D-031 duplicates), and the MP-F2-followon split. **Locked routings (by-recomms):** R2G-F01 -> carried into the UI error-surfacing pass (not a pre-UI arc); R2G-F02 -> named UI build-constraint (attachment-forward must re-encrypt); R2G-F03 + R2G-F04 -> one doc-hygiene pass, any time (D-number duplicates **suffixed, not renumbered** — preserve cross-refs); routed-open (MP-F12 / MP-F13 / MP-F16 / M12.3 `pending_fetches`) left on their named future homes. **UI build inherits explicitly** (named, not blockers): the R2-F01 residual A+thin-fetch (flagged-UNBUILT, node authoritative) + R2G-F02. **No DECISIONS change** (all S3/S4; no new principle). **No Appendix F change** (the gate touched no CLI verb). **Next-active: UI** (clean-table build) -> Streams (post-UI). Canonical (D-074, J-390): this doc ACTIVE v1.0 -> COMPLETED v1.1; `CLAUDE.md` PLAY head (gate CLOSED, next-active = UI); `docs/ROADMAP.md` v3.78 -> v3.79 (gate done at tree / chain / detail); JOURNAL J-390.

Per Rule 0 + D-065 + D-069 + D-074 + the two-round audit principle (2026-06-04).
