# Multiparty Test Matrix — Scenario Catalogue & Results
> **Status**: ACTIVE  
> Version: 1.1  
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

---

## 2. Conventions

**IDs.** `MP-C-##` cooperative / realistic · `MP-A-##` adversarial / break-the-system.

**Each scenario records:** Narrative · Expected · Oracle (M9-D4: `.events` transcripts + `state`
query) · Round · Batch (saved aicontrol file(s)) · Result (PENDING → PASS/FAIL + run ref).

**aicontrol batch files (saved artifacts, M9-D8).** The harness drives `--aicontrol` (persistent
JSONL): one `Command` envelope per line — `{"cmd": "...", "args": {...}, "id": "...",
"bind": "..."}`. Verbs/bindings: `register`→`identity_id` · `create-space`/`create-dm-space`→
`space_id` · `invite`/`join`→`space_id` · `send`→`event_id` · `create-room`. In one connection,
`bind:"s"` + `$s` / `$s.field` chain. Batches are saved, versioned files under
`docs/tests/multiparty_scenarios/<ID>/`, one `.jsonl` per actor + a `manifest.toml` (actor → node
assignment, batch, ordering/barriers, exported reply keys, imported `{{key}}` placeholders). The
harness reads them and feeds line-by-line — no ad-hoc inline generation.

**Cross-actor values (M9-D8).** `bind`/`$` is per-connection → cross-actor values use a `{{key}}`
placeholder the orchestrator fills from a prior actor's **exported** reply field.

**Wire-malformation vs logic-attacks.** Forged-signature / malformed-frame / equivocation cannot
be valid envelopes → they run through the **M9-D6 raw-wire injector**, not batch files.
Logic-attacks (expired invite, tier-gate, unauthorized join) **are** batch-expressible: the batch
sends, the Result asserts the expected rejection **code/category** (`Category`: protocol /
lifecycle / argument / connection / timeout / permission).

---

## 3. Cooperative / realistic family (`MP-C-##`)

### MP-C-01 — multi-client local fan-out (S1)
- **Narrative:** Alice + Carol register on Node A · Alice creates Space S · invites Carol · both post.
- **Expected:** each sees the other's messages; S converges on A.
- **Oracle:** per-client `.events` + `state` compare. **Round:** R1 · **Batch:** `MP-C-01/*` · **Result:** PENDING

### MP-C-02 — invite & join across nodes (S2/INV) [cooperative Round-0 smoke]
- **Narrative:** Alice (A) creates S · invites Bob (B) · Bob joins referencing the invite · Bob posts.
- **Expected:** Bob a member on A **and** B; S converges; Bob's message reaches Alice. (INV bootstrap, M8.5-B.)
- **Oracle:** membership equal A+B; `.events` shows Bob@A. **Round:** R1 · **Batch:** `MP-C-02/{alice,bob}.jsonl` (worked example, §5) · **Result:** PENDING

### MP-C-03 — concurrent send under conflict (S2)
- **Narrative:** Alice (A) + Bob (B), members of S, both `send` at one frontier · nodes federate.
- **Expected:** both retained; resolved order byte-identical A+B (M8).
- **Oracle:** ordered `.events` compare. **Round:** R1 → R2 · **Batch:** `MP-C-03/*` · **Result:** PENDING

### MP-C-04 — federation topology, transitive path (S3)
- **Narrative:** 3 Nodes A-B-C · Space on A · members on A,B,C · A posts.
- **Expected:** delivery per the locked F-5/D-089 pairwise model; convergence on all three.
- **Oracle:** `state`+`.events` across A/B/C. **Round:** R2 · **Batch:** `MP-C-04/*` · **Result:** PENDING

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
- **Oracle:** `state`+`.events` across all nodes. **Round:** R2 → R3 · **Batch:** generated per topology · **Result:** PENDING

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
raw injector (not batch-expressible).

### MP-A-01 — expired-invite federation replay (logic) [INV-EXP, J-298]
- **Narrative:** Alice invites Bob (14d TTL) · clock advances past `valid_until` · a peer catches up the aged Space + replays invite + join.
- **Expected:** Bob's membership **preserved** on the catching-up peer; gate does not re-reject on federation replay (admission-only).
- **Oracle:** membership equal across nodes. **Round:** R1 · **Batch:** `MP-A-01/*` + MockClock advance · **Result:** PENDING

### MP-A-02 — over-ceiling / expired invite at submission (logic) [3044/3045]
- **Expected:** rejected; `category=lifecycle`/`argument`. **Round:** R1 · **Batch:** `MP-A-02/*` · **Result:** PENDING

### MP-A-03 — tier-gate join refusal (logic) [PG-13]
- **Expected:** join refused; refusal multiparty-visible + converged; `category=permission`. **Round:** R1 · **Batch:** `MP-A-03/*` · **Result:** PENDING

### MP-A-04 — unauthorized / non-member send (logic)
- **Expected:** rejected; no event admitted to S anywhere. **Round:** R1 · **Batch:** `MP-A-04/*` · **Result:** PENDING

### MP-A-05 — signature / identity forgery (wire) [F-F] [adversarial Round-0 smoke]
- **Narrative:** the injector emits an event signed with a key not matching the claimed identity.
- **Expected:** rejected at `ingest_event` on every node; never applied.
- **Oracle:** event absent from all node state. **Round:** R1 · **Mechanism:** M9-D6 injector · **Result:** PENDING

### MP-A-06 — equivocation / fork attempt (wire) [F-F]
- **Narrative:** a hostile peer presents conflicting events at one frontier to different nodes.
- **Expected:** honest nodes converge on the single resolved winner; no permanent fork. **Round:** R2 · **Mechanism:** injector · **Result:** PENDING

### MP-A-07 — flooding / DoS (volume) [M8.6]
- **Expected:** no hang; local liveness; honest traffic still applies (cap back-pressure, C8). **Round:** R2 → R3 · **Mechanism:** injector high-rate · **Result:** PENDING

### MP-A-08 — partition + reconnect storm (volume) [M8.6]
- **Expected:** convergence after heal; no lost admitted events; no reconnect deadlock. **Round:** R3 · **Mechanism:** orchestrator link control · **Result:** PENDING

### MP-A-09 — duplicate-event_id replay / dedup (wire)
- **Narrative:** the injector re-sends a valid event with the same `event_id`.
- **Expected:** idempotent — applied once, no duplicate in state. **Round:** R1 · **Mechanism:** injector · **Result:** PENDING

### MP-A-10 — causal gap / missing-parent (wire)
- **Narrative:** an event arrives whose `prev_events` are absent.
- **Expected:** buffered (HeldPending) then drained on arrival, or dropped — never applied out of causal order. **Round:** R1 · **Mechanism:** injector · **Result:** PENDING

### MP-A-11 — oversized payload (resource)
- **Expected:** rejected or bounded; node stays live; no OOM. **Round:** R2 · **Mechanism:** injector · **Result:** PENDING

### MP-A-12 — malformed / truncated frame (wire)
- **Expected:** rejected at parse; connection handled gracefully; node stays live. **Round:** R1 · **Mechanism:** injector · **Result:** PENDING

### MP-A-13 — anti-transitivity probe (federation) [F-5/D-089]
- **Narrative:** A→B delivers an event; assert B does **not** re-forward it to C (pairwise, not transitive relay).
- **Expected:** C does not receive via B; the locked full-mesh/pairwise model holds. **Round:** R2 · **Mechanism:** observe `.events` on C · **Result:** PENDING

### MP-A-14 — ban-evasion via new identity (logic)
- **Narrative:** a banned user registers a fresh identity and attempts to rejoin.
- **Expected:** treated as a new identity subject to the same gates (no automatic re-entry); recorded behaviour. **Round:** R1 · **Batch:** `MP-A-14/*` · **Result:** PENDING

### MP-A-15 — clock-skew timestamp (wire)
- **Narrative:** the injector sends an event with a far-future / far-past timestamp.
- **Expected:** resolution unaffected (wire-order determinism, D-076); no state corruption. **Round:** R1 · **Mechanism:** injector · **Result:** PENDING

### MP-A-16 — forged invite ("never issued") (logic/wire)
- **Narrative:** a join references an invite event that was never issued.
- **Expected:** rejected; no membership granted. **Round:** R1 · **Mechanism:** injector / batch · **Result:** PENDING

### MP-A-17 — wrong-space_id confusion (logic)
- **Narrative:** an event references a space the actor is not in / does not exist.
- **Expected:** rejected; no cross-space leakage. **Round:** R1 · **Batch:** `MP-A-17/*` · **Result:** PENDING

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

## 5. Worked example — MP-C-02 batch shape

Two saved files fed to each actor's `--aicontrol` pipe. Cross-actor values use the `{{...}}`
harness placeholder (the orchestrator fills `{{space_id}}` / `{{invite_event_id}}` from Alice's
exported replies — M9-D8).

`docs/tests/multiparty_scenarios/MP-C-02/alice.jsonl`:
```
{"cmd":"register","args":{"name":"alice"},"id":"a1","bind":"me"}
{"cmd":"create-space","args":{"name":"S"},"id":"a2","bind":"s"}
{"cmd":"invite","args":{"space":"$s","identity":"{{bob_identity_id}}"},"id":"a3","bind":"inv"}
{"cmd":"send","args":{"space":"$s","text":"hi bob"},"id":"a4"}
```

`docs/tests/multiparty_scenarios/MP-C-02/bob.jsonl`:
```
{"cmd":"register","args":{"name":"bob"},"id":"b1","bind":"me"}
{"cmd":"join","args":{"space":"{{space_id}}","invite_event":"{{invite_event_id}}"},"id":"b2"}
{"cmd":"send","args":{"space":"{{space_id}}","text":"hi alice"},"id":"b3"}
```

(Illustrative — exact arg keys + the cross-actor substitution syntax confirmed in the runbook
against the grounded command surface.)

---

## 6. Status roll-up

| Family | Seeded | PASS | FAIL | PENDING |
|--------|-------:|-----:|-----:|--------:|
| Cooperative (MP-C) | 16 | 0 | 0 | 16 |
| Adversarial (MP-A) | 21 | 0 | 0 | 21 |

All PENDING — harness in M9 build (Round-0 smokes MP-C-02 + MP-A-05 first). Results begin at the
first Multiparty-tests R1 run on a finalized binary.

Per D-065 + D-069 + D-074.
