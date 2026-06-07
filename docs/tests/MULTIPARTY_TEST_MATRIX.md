# Multiparty Test Matrix — Scenario Catalogue & Results
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this file is

The single **what-we-test + result** record for the strategic multiparty campaign. Companion
to `tasks/M9_MULTIPARTY_HARNESS_AUDIT.md` + `tasks/M9_MULTIPARTY_HARNESS_DESIGN.md`: the audit/
design define **how** the harness is built; this defines **what** it exercises, in actor-
narrative form, with a result per scenario.

- **Scenarios are authored now** (they shape what the harness must support) and **results fill
  in during the Multiparty-tests runs** (R1 → R2 → R3, audit §6.2).
- Supersedes the DEPRECATED `MULTIPARTY_S1..S5_*.md` spec files as the live scenario source.
- **Living document** — scenarios are added as design + runs surface them.
- **M9 Round-0 status:** the `xgen-mptest` harness is built (M9 C1–C5, closed J-307); MP-C-02 +
  MP-A-05 ran green against the real binaries (single-node). The other 35 stay PENDING for the
  Multiparty-tests milestone. Open findings live in `tasks/M9_findings.md`.

---

## 2. Conventions

**IDs.** `MP-C-##` cooperative / realistic · `MP-A-##` adversarial / break-the-system.

**Each scenario records:** Narrative · Expected · Oracle (M9-D4: `.events` transcripts + `state`
query) · Round · Batch (saved aicontrol file(s)) · Result (PENDING → PASS/FAIL + run ref).

**aicontrol batch files (saved artifacts, M9-D8).** The harness drives `--aicontrol` (persistent
JSONL): one `Command` envelope per line — `{"cmd": "...", "args": {...}, "id": "..."}`.
Verbs/bindings: `register`→`identity_id` · `create-space`/`create-dm-space`→`space_id` ·
`create-room`→`room_id` · `invite`→`event_id` (requires `role`) · `join`→`space_id` (takes
`space` + optional `room`; **no** `invite_event` — chaining is the node's pending-invite
bootstrap + `prev_events`) · `send`→`event_id` (requires `room`). In one connection the binary's
own `bind` + `$name` chains per-connection. Batches are saved, versioned files under
`docs/tests/multiparty_scenarios/<ID>/`, one `.jsonl` per actor + a `manifest.toml` (actor → node
assignment, batch, `[[exports]]`, `[[waits]]` ordering edges). The harness feeds lines verbatim
(after `{{}}` substitution) — no ad-hoc inline generation.

**Cross-actor values (M9-D8).** Per-connection `bind`/`$` cannot cross actors → cross-actor
values use a `{{key}}` placeholder the orchestrator fills from a prior actor's **exported** reply
field. Data-dependency auto-orders; non-data ordering uses a manifest `[[waits]]` edge.

**Wire-malformation vs logic-attacks.** Forged-signature / malformed-frame / equivocation cannot
be valid envelopes → they run through the **M9-D6 raw-wire injector**, not batch files.
Logic-attacks (expired invite, tier-gate, unauthorized join) **are** batch-expressible: the batch
sends, the Result asserts the expected rejection **code/category** (`Category`: protocol /
lifecycle / argument / connection / timeout / permission).

---

## 3. Cooperative / realistic family (`MP-C-##`)

> **Cross-node prerequisite (M9 finding F2).** The scenarios that span nodes (true MP-C-02,
> MP-C-03, MP-C-04, MP-C-14) require a **fresh-peer federation-initiate surface** — two fresh
> node binaries cannot currently be federated through the external control surfaces (initiate is
> known-peers-only, `FED_3006`; no config peer-list). Flagged **Multiparty-tests prerequisite**
> (likely a small initiate verb; Joe-lock when that milestone opens). Round-0 ran MP-C-02
> single-node (real convergence) to prove the machinery without it. See `tasks/M9_findings.md`.

### MP-C-01 — multi-client local fan-out (S1)
- **Narrative:** Alice + Carol register on Node A · Alice creates Space S · invites Carol · both post.
- **Expected:** each sees the other's messages; S converges on A.
- **Oracle:** per-client `.events` + `state` compare. **Round:** R1 · **Batch:** `MP-C-01/*` · **Result:** PENDING

### MP-C-02 — invite & join (S2/INV) [cooperative Round-0 smoke — ✅ PASS]
- **Narrative:** Alice creates S + a room · invites Bob (`role:member`) · Bob joins (pending-invite bootstrap, no `invite_event` arg) · both post.
- **Expected:** Bob a member; S converges; both views agree `{alice:owner, bob:member}`. (INV bootstrap, M8.5-B.)
- **Oracle:** membership equal across views; per-Space `.events` id-set matches.
- **✅ Round-0 result (M9 C5, run `c5_mp_c_02`):** PASS — **single-node** (Option 1, Joe-lock J-307): alice+bob both on Node A, **real protocol convergence**, full machinery proven (spawn → aicontrol drive → cross-actor `{{}}` + `[[waits]]` ordering → membership oracle). True cross-node A↔B form gated on F2 (above).
- **Round:** R1 · **Batch:** `MP-C-02/{alice,bob}.jsonl` + `manifest.toml` (§5, committed)

### MP-C-03 — concurrent send under conflict (S2)
- **Narrative:** Alice (A) + Bob (B), members of S, both `send` at one frontier · nodes federate.
- **Expected:** both retained; resolved order byte-identical A+B (M8).
- **Oracle:** ordered `.events` compare. **Round:** R1 → R2 · **Batch:** `MP-C-03/*` · **Result:** PENDING (cross-node, gated on F2)

### MP-C-04 — federation topology, transitive path (S3)
- **Narrative:** 3 Nodes A-B-C · Space on A · members on A,B,C · A posts.
- **Expected:** delivery per the locked F-5/D-089 pairwise model; convergence on all three.
- **Oracle:** `state`+`.events` across A/B/C. **Round:** R2 · **Batch:** `MP-C-04/*` · **Result:** PENDING (cross-node, gated on F2)

### MP-C-05 — sustained n×n chat (S4)
- **Narrative:** N nodes × M clients, sustained interleaved posting for a window.
- **Expected:** loss-free at resolution; all projections converge; no hang.
- **Oracle:** final-state convergence + liveness; capture RSS/thread curves. **Round:** R2 → R3 · **Batch:** generated per ramp · **Result:** PENDING

### MP-C-06 — identity re-home (S5)
- **Narrative:** Bob on B, member of S · re-homes to C, same identity · posts from C.
- **Expected:** identity + membership continuous; post from C reaches S (S5 `re_registration` + `identity.home_changed`, M8.5-C).
- **Oracle:** identity continuity + membership preserved; `.events` shows Bob@C. **Round:** R1 · **Batch:** `MP-C-06/*` · **Result:** PENDING

### MP-C-07 — DM private space across nodes
- **Narrative:** Alice (A) `create-dm-space` with Bob (B) · both exchange messages.
- **Expected:** single-homed DM space, both parties converge, no third-party visibility.
- **Oracle:** `.events`+`state` both; absence on a non-party node. **Round:** R1 · **Batch:** `MP-C-07/*` · **Result:** PENDING

### MP-C-08 — multi-room space + per-room overrides (PG-12)
- **Narrative:** Alice creates S + multiple rooms · sets a per-room `Deny` override · members post per room.
- **Expected:** posts honor per-room overrides; each room converges independently; override enforced + converged.
- **Oracle:** per-room `state` + `.events`. **Round:** R1 · **Batch:** `MP-C-08/*` · **Result:** PENDING

### MP-C-09 — ban → converge → post-rejected
- **Narrative:** Member Bob is banned by an admin · Bob attempts a post after the ban.
- **Expected:** ban converges on every node; Bob's post-ban event rejected/excluded everywhere (M8 ban-vs-join Layer 1).
- **Oracle:** membership + `.events` exclude Bob's late post on all nodes. **Round:** R1 · **Batch:** `MP-C-09/*` · **Result:** PENDING

### MP-C-10 — leave & rejoin
- **Narrative:** Bob `leave`s S, later rejoins via a fresh invite.
- **Expected:** leave converges; rejoin admitted; membership timeline consistent across nodes.
- **Oracle:** `state` membership history compare. **Round:** R1 · **Batch:** `MP-C-10/*` · **Result:** PENDING

### MP-C-11 — membership churn under load
- **Narrative:** Many joins/leaves interleaved with sustained posting over a window.
- **Expected:** convergence holds throughout; no orphaned members; no lost admitted posts.
- **Oracle:** final-state convergence + member-set equality. **Round:** R2 → R3 · **Batch:** generated per ramp · **Result:** PENDING

### MP-C-12 — E2E-encrypted space content-blindness (S6)
- **Narrative:** N-member Space with `e2e_encryption` ON · members exchange encrypted messages.
- **Expected:** zero plaintext in any node-visible surface; KeyPackage consume + replenish; epoch advance on commit (Arc H).
- **Oracle:** node-side `.events`/store carry ciphertext only; client decrypts. **Round:** R2 · **Batch:** `MP-C-12/*` · **Result:** PENDING

### MP-C-13 — thread create / resolve / archive (Arc E)
- **Narrative:** members create a thread, post, resolve, then archive it.
- **Expected:** thread state transitions converge (rides M8 Layer-5c).
- **Oracle:** `ThreadState` projection equal across nodes. **Round:** R1 · **Batch:** `MP-C-13/*` · **Result:** PENDING

### MP-C-14 — 4–5 node star + mesh topology
- **Narrative:** A central node + leaves (star), then add cross-links (mesh) · a Space spanning all.
- **Expected:** delivery + convergence consistent under both topologies (pairwise-trust model).
- **Oracle:** `state`+`.events` across all nodes. **Round:** R2 → R3 · **Batch:** generated per topology · **Result:** PENDING (cross-node, gated on F2)

### MP-C-15 — node restart mid-chat + replay (S4 durability)
- **Narrative:** A node hosting S is killed mid-conversation, restarted (replay-from-disk), catches up.
- **Expected:** replayed `SpaceState` byte-identical; zero orphans; rejoins federation + converges.
- **Oracle:** pre/post-restart `state` equality + cross-node convergence. **Round:** R2 · **Batch:** `MP-C-15/*` + orchestrator kill/restart · **Result:** PENDING

### MP-C-16 — live space migration during chat (Arc F)
- **Narrative:** `migration initiate` moves S's `home_node` A→B while members post.
- **Expected:** `home_node` flips on both nodes; in-flight posts not lost; convergence holds across cutover.
- **Oracle:** `home_node` + `state` equality post-cutover. **Round:** R2 · **Batch:** `MP-C-16/*` + migration verb · **Result:** PENDING

---

## 4. Adversarial / break-the-system family (`MP-A-##`)

Logic-attacks → R1 (cheap, deterministic). Volume-attacks → R2/R3. Wire-malformation → M9-D6
raw injector (not batch-expressible). C4 catalogued all six injector attacks with grounded
rejection points (`tasks/M9_findings.md`); MP-A-05 ran live at Round-0.

### MP-A-01 — expired-invite federation replay (logic) [INV-EXP, J-298]
- **Narrative:** Alice invites Bob (14d TTL) · clock advances past `valid_until` · a peer catches up the aged Space + replays invite + join.
- **Expected:** Bob's membership **preserved** on the catching-up peer; gate does not re-reject on federation replay (admission-only).
- **Oracle:** membership equal across nodes. **Round:** R1 · **Batch:** `MP-A-01/*` + clock advance · **Result:** PENDING (needs the F3 clock-advance surface)

### MP-A-02 — over-ceiling / expired invite at submission (logic) [3044/3045]
- **Expected:** rejected; `category=lifecycle`/`argument`. **Round:** R1 · **Batch:** `MP-A-02/*` · **Result:** PENDING

### MP-A-03 — tier-gate join refusal (logic) [PG-13]
- **Expected:** join refused; refusal multiparty-visible + converged; `category=permission`. **Round:** R1 · **Batch:** `MP-A-03/*` · **Result:** PENDING

### MP-A-04 — unauthorized / non-member send (logic)
- **Expected:** rejected; no event admitted to S anywhere. **Round:** R1 · **Batch:** `MP-A-04/*` · **Result:** PENDING

### MP-A-05 — signature / identity forgery (wire) [F-F] [adversarial Round-0 smoke — ✅ PASS]
- **Narrative:** the injector emits an event signed with a key not matching the claimed identity.
- **Expected:** rejected at `validate_event` (F-4 13-step, **step 12** signature check, exchange.rs — *not* `ingest_event`, which is the no-validation direct-insert) on every node; never applied.
- **Oracle:** event absent from all node `.events`.
- **✅ Round-0 result (M9 C5, run `c5_mp_a_05`):** PASS — node returned `Error(4000, "step 12: signature verification failed")`; forged event absent; the legitimate control message applied. Step-12 isolation against Alice's real member-context Space.
- **Round:** R1 · **Mechanism:** M9-D6 raw-wire injector

### MP-A-06 — equivocation / fork attempt (wire) [F-F]
- **Narrative:** a hostile peer presents conflicting events at one frontier to different nodes.
- **Expected:** **not a rejection** — both valid events apply; M8 resolution converges on a single winner; no permanent fork. (Outcome = convergence-on-winner, not absence.) **Round:** R2 · **Mechanism:** injector · **Result:** PENDING

### MP-A-07 — flooding / DoS (volume) [M8.6]
- **Expected:** no hang; local liveness; honest traffic still applies (cap back-pressure, C8). **Round:** R2 → R3 · **Mechanism:** injector high-rate · **Result:** PENDING

### MP-A-08 — partition + reconnect storm (volume) [M8.6]
- **Expected:** convergence after heal; no lost admitted events; no reconnect deadlock. **Round:** R3 · **Mechanism:** orchestrator link control · **Result:** PENDING

### MP-A-09 — duplicate-event_id replay / dedup (wire)
- **Narrative:** the injector re-sends a valid event with the same `event_id`.
- **Expected:** idempotent — DAG dedup (`graph.add_event`, after validation); applied once. **Round:** R1 · **Mechanism:** injector (member-context) · **Result:** PENDING

### MP-A-10 — causal gap / missing-parent (wire)
- **Narrative:** an event arrives whose `prev_events` are absent.
- **Expected:** buffered (HeldPending) then drained on arrival, or dropped — never applied out of causal order. **Round:** R1 · **Mechanism:** injector · **Result:** PENDING

### MP-A-11 — oversized payload (resource)
- **Expected:** rejected or bounded; node stays live; no OOM. **Round:** R2 · **Mechanism:** injector · **Result:** PENDING

### MP-A-12 — malformed / truncated frame (wire)
- **Expected:** rejected at transport frame-parse (never reaches `validate_event`); node stays live. **Round:** R1 · **Mechanism:** injector (needs the F4 raw-send seam) · **Result:** PENDING

### MP-A-13 — anti-transitivity probe (federation) [F-5/D-089]
- **Narrative:** A→B delivers an event; assert B does **not** re-forward it to C (pairwise, not transitive relay).
- **Expected:** C does not receive via B; the locked full-mesh/pairwise model holds. **Round:** R2 · **Mechanism:** observe `.events` on C · **Result:** PENDING

### MP-A-14 — ban-evasion via new identity (logic)
- **Narrative:** a banned user registers a fresh identity and attempts to rejoin.
- **Expected:** treated as a new identity subject to the same gates (no automatic re-entry); recorded behaviour. **Round:** R1 · **Batch:** `MP-A-14/*` · **Result:** PENDING

### MP-A-15 — clock-skew timestamp (wire)
- **Narrative:** the injector sends an event with a far-future / far-past timestamp.
- **Expected:** *(intended)* resolution unaffected (wire-order determinism, D-076); no state corruption. **🔴 M9 finding F1 (gap G6):** `validate_event` has **no timestamp-bound check** → a skewed-but-otherwise-valid event is silently accepted. Routed to a fix-arc. **Round:** R1 · **Mechanism:** injector · **Result:** PENDING (finding open)

### MP-A-16 — forged invite ("never issued") (logic/wire)
- **Narrative:** a join references an invite event that was never issued.
- **Expected:** rejected (missing-predecessor / membership) or HeldPending→timeout; no membership granted. **Round:** R1 · **Mechanism:** injector / batch · **Result:** PENDING

### MP-A-17 — wrong-space_id confusion (logic)
- **Narrative:** an event references a space the actor is not in / does not exist.
- **Expected:** rejected (`Error 4000` space-not-found observed live at C4); no cross-space leakage. **Round:** R1 · **Batch:** `MP-A-17/*` · **Result:** PENDING

### MP-A-18 — connect / disconnect storm (volume) [C4 leak gauge]
- **Expected:** no task/handle leak; node stays live (the M8.6 C4 attempt-gauge property at the binary). **Round:** R2 → R3 · **Mechanism:** orchestrator churn · **Result:** PENDING

### MP-A-19 — slow-loris / held connections (resource)
- **Expected:** held/partial connections do not exhaust the node; honest traffic unaffected. **Round:** R2 · **Mechanism:** injector partial-write · **Result:** PENDING

### MP-A-20 — privilege escalation (logic)
- **Narrative:** a non-admin actor attempts an admin verb (`space set-node-policy`, ban).
- **Expected:** refused; `category=permission`; no state change. **Round:** R1 · **Batch:** `MP-A-20/*` · **Result:** PENDING

### MP-A-21 — stale / rollback MLS commit (wire) [M8.7]
- **Narrative:** the injector replays a stale `mls.commit` against an advanced epoch.
- **Expected:** no epoch regression; concurrent-commit resolution holds (`mls_commit_tip`, M8.7). **Round:** R2 · **Mechanism:** injector · **Result:** PENDING

---

## 5. Runnable batches (authoritative on disk)

The runnable MP-C-02 batches are committed at `docs/tests/multiparty_scenarios/MP-C-02/` —
`alice.jsonl` + `bob.jsonl` + `manifest.toml` (M9 C5, commit `6d08859`) — and are the
authoritative shapes. They match the **real** client arg surface, which differs from the early
illustrative seed (corrected here so the catalogue does not teach a false mechanism):

- `invite` requires a `role` arg (e.g. `member`) — no default.
- `join` takes **no** `invite_event` arg — invite-chaining is the node's pending-invite bootstrap
  + `prev_events`, not a join argument (`JoinArgs = {space, room?}`).
- `send` requires a `room` arg — the room id comes from `create-room` (exported as `room_id`),
  not from the `create-space` reply (`{space_id, event_id}` only).
- Cross-actor ordering that data-dependency cannot express (Bob's `join` must follow Alice's
  `invite`) uses a manifest `[[waits]]` edge (`bob.b2` waits for the exported `invite_ready`).

---

## 6. Status roll-up

| Family | Seeded | PASS | FAIL | PENDING |
|--------|-------:|-----:|-----:|--------:|
| Cooperative (MP-C) | 16 | 1 | 0 | 15 |
| Adversarial (MP-A) | 21 | 1 | 0 | 20 |

**Round-0 (M9) complete (J-307):** MP-C-02 (cooperative) + MP-A-05 (adversarial) ✅ PASS against
the real binaries via the `xgen-mptest` harness (single-node — the harness is the machinery, the
proof). The remaining 35 scenarios stay PENDING for the **Multiparty-tests** milestone
(R1 → R2 → R3) on a finalized binary, gated on the open findings in `tasks/M9_findings.md`
(notably the F2 fresh-peer federation-initiate surface for the cross-node cooperative set and the
F3 clock-advance surface for the deterministic round).

Per D-065 + D-069 + D-074.
