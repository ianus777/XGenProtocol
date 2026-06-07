# Multiparty Test Matrix — Scenario Catalogue & Results
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this file is

The single **what-we-test + result** record for the strategic multiparty campaign. Companion
to `tasks/M9_MULTIPARTY_HARNESS_AUDIT.md`: the audit defines **how** the harness is built; this
defines **what** it exercises, in actor-narrative form, with a result per scenario.

- **Scenarios are authored now** (they shape what the harness must support) and **results fill
  in during the Multiparty-tests runs** (R1 → R2 → R3, audit §6.2).
- Supersedes the scattered DEPRECATED `MULTIPARTY_S1..S5_*.md` spec files as the live source of
  scenario intent (those remain readable history).
- **Living document** — scenarios are added as the design phase + run rounds surface them; this
  v1.0 is a seed, not the final set.

---

## 2. Conventions

**IDs.** `MP-C-##` cooperative / realistic · `MP-A-##` adversarial / break-the-system.

**Each scenario records:** Narrative (actor steps) · Expected (the invariant that must hold) ·
Oracle (how it is checked — ties to audit M9-F-D: `.events` transcripts + `state` query) ·
Round (R1/R2/R3) · Batch (the saved aicontrol file(s)) · Result (PENDING → PASS/FAIL + run ref).

**aicontrol batch files (Joe, 2026-06-07 — saved artifacts).** The harness drives the binaries
through the **`--aicontrol`** persistent **JSONL** pipe: one `Command` envelope per line —
`{"cmd": "...", "args": {...}, "id": "...", "bind": "..."}`. Verbs/bindings (grounded):
`register`→`identity_id` · `create-space`/`create-dm-space`→`space_id` · `invite`/`join`→
`space_id` · `send`→`event_id` · `create-room`. Within one connection, `bind:"s"` + `$s` /
`$s.field` chain results. **The batches are saved, versioned files** under
`docs/tests/multiparty_scenarios/<ID>/`, one `.jsonl` per actor (one client process = one pipe =
one batch). The harness reads them and feeds them line-by-line — there is no ad-hoc inline
command generation.

**Two open design seams (flagged, NOT locked — audit M9 design phase):**
- **Cross-actor values.** `bind`/`$` is per-connection, so Bob's `join` cannot see Alice's
  `$space_id`. Cross-actor values use a harness-level placeholder (proposed `{{space_id}}`) the
  orchestrator fills from a prior actor's reply. Exact substitution + ordering/sync mechanism =
  design-phase.
- **Wire-malformation attacks** (forged signature bytes, malformed frames) cannot be valid
  `Command` envelopes → they run through the **F-F raw injector**, not batch files. Logic-attacks
  (expired invite, tier-gate bypass, unauthorized join) **are** batch-expressible: the batch
  sends the command and the Result asserts the expected rejection **code/category** (envelope
  `Category`: protocol / lifecycle / argument / connection / timeout / permission).

---

## 3. Cooperative / realistic family (`MP-C-##`)

### MP-C-01 — multi-client local fan-out (S1)
- **Narrative:** Alice + Carol register on Node A · Alice creates Space S · Alice invites Carol ·
  both post messages.
- **Expected:** each client sees the other's messages; S converges byte-identical on A.
- **Oracle:** `.events` transcript per client + `state` membership/room compare.
- **Round:** R1 · **Batch:** `MP-C-01/{alice,carol}.jsonl` · **Result:** PENDING

### MP-C-02 — invite & join across nodes (S2/INV)
- **Narrative:** Alice (Node A) registers + creates Space S · invites Bob (Node B) · Bob joins
  referencing the invite event · Bob posts a message.
- **Expected:** Bob is a member on **both** A and B; S converges byte-identical; Bob's message
  reaches Alice. Exercises the INV bootstrap (M8.5-B).
- **Oracle:** membership equal across A + B; `.events` shows Bob's message on A.
- **Round:** R1 · **Batch:** `MP-C-02/{alice,bob}.jsonl` (worked example, §5) · **Result:** PENDING

### MP-C-03 — concurrent send under conflict (S2)
- **Narrative:** Alice (A) + Bob (B), both members of S · both `send` at the same frontier ·
  nodes federate.
- **Expected:** both messages retained; resolved order byte-identical on A + B (M8 convergence).
- **Oracle:** ordered `.events` compare A vs B.
- **Round:** R1 (logic) → R2 (volume) · **Batch:** `MP-C-03/{alice,bob}.jsonl` · **Result:** PENDING

### MP-C-04 — federation topology, transitive path (S3)
- **Narrative:** 3 Nodes A-B-C · a Space hosted on A · members on A, B, C · A posts.
- **Expected:** delivery consistent with the locked propagation model (F-5/D-089 pairwise; A→C
  via the established relationships, not transitive relay). Convergence holds on all three.
- **Oracle:** `state` + `.events` compare across A/B/C.
- **Round:** R2 · **Batch:** `MP-C-04/{a,b,c}.jsonl` · **Result:** PENDING

### MP-C-05 — sustained n×n chat (S4)
- **Narrative:** N nodes × M clients, sustained interleaved posting for a fixed window.
- **Expected:** loss-free at resolution; all projections converge; no deadlock/hang under load.
- **Oracle:** final-state convergence + per-process liveness; capture RSS/thread curves.
- **Round:** R2 → R3 · **Batch:** generated per ramp profile (round dial) · **Result:** PENDING

### MP-C-06 — identity re-home (S5)
- **Narrative:** Bob registered on B, member of S · Bob re-registers / re-homes to C, same
  identity · Bob posts from C.
- **Expected:** Bob's identity + membership continuous across the re-home; post from C reaches S
  (S5 `re_registration` + `identity.home_changed`, M8.5-C).
- **Oracle:** identity continuity + membership preserved; `.events` shows Bob@C.
- **Round:** R1 · **Batch:** `MP-C-06/bob.jsonl` (+ orchestrator re-home step) · **Result:** PENDING

---

## 4. Adversarial / break-the-system family (`MP-A-##`)

Logic-attacks are crowd-size-independent → R1 (cheap, deterministic). Volume-attacks → R2/R3.
Wire-malformation → F-F raw injector (not batch-expressible).

### MP-A-01 — expired-invite federation replay (logic) [INV-EXP, J-298]
- **Narrative:** Alice invites Bob (14d TTL) · clock advances past `valid_until` · a peer catches
  up the aged Space and replays invite + Bob's join.
- **Expected:** Bob's membership **preserved** on the catching-up peer; the gate does **not**
  re-reject on federation replay (admission-only).
- **Oracle:** membership equal across all nodes. **Round:** R1 · **Batch:** `MP-A-01/*` +
  MockClock advance · **Result:** PENDING

### MP-A-02 — over-ceiling / expired invite at submission (logic) [3044/3045]
- **Narrative:** local actor submits a join against an expired invite / an invite over the tier
  ceiling.
- **Expected:** rejected with the expected code; `category=lifecycle`/`argument` as applicable.
- **Round:** R1 · **Batch:** `MP-A-02/*` · **Result:** PENDING

### MP-A-03 — tier-gate join refusal (logic) [PG-13]
- **Narrative:** an identity below the Space's required tier attempts to join.
- **Expected:** join refused; refusal multiparty-visible + converged on every node;
  `category=permission`.
- **Round:** R1 · **Batch:** `MP-A-03/*` · **Result:** PENDING

### MP-A-04 — unauthorized post / non-member send (logic)
- **Narrative:** a non-member attempts `send` into S.
- **Expected:** rejected; no event admitted to S on any node.
- **Round:** R1 · **Batch:** `MP-A-04/*` · **Result:** PENDING

### MP-A-05 — signature / identity forgery (wire) [F-F]
- **Narrative:** the raw injector emits an event signed with a key not matching the claimed
  identity.
- **Expected:** rejected at validation on every receiving node; never applied.
- **Oracle:** event absent from all node state. **Round:** R1 · **Mechanism:** F-F injector ·
  **Result:** PENDING

### MP-A-06 — equivocation / fork attempt (wire) [F-F]
- **Narrative:** a hostile peer presents two conflicting events at one frontier to different
  nodes.
- **Expected:** all honest nodes converge on the single resolved winner; no permanent fork.
- **Round:** R2 · **Mechanism:** F-F injector · **Result:** PENDING

### MP-A-07 — flooding / DoS (volume) [M8.6 back-pressure]
- **Narrative:** an actor floods a Node with events beyond channel capacity.
- **Expected:** no hang; local liveness preserved; honest traffic still applies (cap-back-pressure
  behaviour, M8.6 C8).
- **Round:** R2 → R3 · **Mechanism:** hostile driver, high rate · **Result:** PENDING

### MP-A-08 — partition + reconnect storm (volume) [M8.6]
- **Narrative:** sever + restore federation links across many nodes; observe reconnect ladder +
  buffered-event drain.
- **Expected:** convergence after heal; no lost admitted events; no reconnect deadlock.
- **Round:** R3 · **Mechanism:** orchestrator link control · **Result:** PENDING

---

## 5. Worked example — MP-C-02 batch shape

Two saved files, fed to each actor's `--aicontrol` pipe. Cross-actor values use the proposed
`{{...}}` harness placeholder (the orchestrator fills `{{space_id}}` / `{{invite_event_id}}`
from Alice's replies — the design seam in §2).

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

(Illustrative — exact arg keys + the cross-actor substitution syntax are confirmed in the M9
design phase against the grounded command surface.)

---

## 6. Status roll-up

| Family | Seeded | PASS | FAIL | PENDING |
|--------|-------:|-----:|-----:|--------:|
| Cooperative (MP-C) | 6 | 0 | 0 | 6 |
| Adversarial (MP-A) | 8 | 0 | 0 | 8 |

All PENDING — harness not yet built (M9 in Phase-0). Results begin at the first Multiparty-tests
R1 run on a finalized binary.

Per D-065 + D-069 + D-074.
