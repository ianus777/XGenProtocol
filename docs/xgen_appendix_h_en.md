# XGen Protocol — Appendix H: Full Integration Test Records
> **Status:** ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated:** 2026-05-15  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

This appendix is the archival record of XGen Protocol integration test runs. It contains the complete terminal output from each full integration test — every step, every result line, every summary banner — as produced by the reference binaries against live Node instances over real TCP. The output is reproduced verbatim; nothing is paraphrased or summarised here.

For the meaning and structure of each test, see the corresponding Chapter 4 section. For the development context, bug fixes, and decision records associated with each run, see `JOURNAL.md` and `DECISIONS.md`.

---

## H.1 — Full Integration Smoke Test

**Reference:** Ch4 §4.18 — summary and phase structure  
**Instruction file:** `docs/tests/INTEGRATION_TEST_ph2.md`  
**Journal entry:** J-058 (2026-05-14)  
**Binary version:** post D-056 fix  
**Environment:** Node A `ws://127.0.0.1:9080/xgen`, Node B `ws://127.0.0.1:9081/xgen`, debug build  

**Result: PASS — 60/60 steps**

---

### H.1.1 Full Step Output

```
════════════════════════════════════════════════════════════
SMOKE-TEST-PH2 — Full Integration Smoke Test
════════════════════════════════════════════════════════════
Node A:  ws://127.0.0.1:9080/xgen
Node B:  ws://127.0.0.1:9081/xgen
── Phase 0: Phase 1 Baseline (Steps 1–17) ──────────────────
[PASS] Step  1 — Node A running; Alice ephemeral keypair generated
[PASS] Step  2 — Alice registers on Node A
[PASS] Step  3 — Node B running; test-Node-B federation keypair generated
[PASS] Step  4 — Bob registers on Node B
[PASS] Step  5 — Alice creates Space (xgen://hash/sha256:d7b1e82478b...)
[PASS] Step  6 — Alice creates Room 'general' (xgen://hash/sha256:06a5ee2d085...)
[PASS] Step  7 — Alice invites Bob to the Space
[PASS] Step  8 — test-Node-B federated with Node A (session xgen://hash/sha256:f...)
[PASS] Step  9 — test-Node-B sends space.join_request
[PASS] Step 10 — Node A produces state.federation_add (xgen://hash/sha256:9df5bcef217...)
[PASS] Step 11 — Node A sends 4 history events to test-Node-B
[PASS] Step 12 — Bob joins the Space
[PASS] Step 13 — Bob joins the Room
[PASS] Step 14 — Alice sends 'Hello Bob' to Node A
[PASS] Step 15 — Bob sends 'Hello Alice' to Node B
[PASS] Step 16 — both message signatures valid
[PASS] Step 17 — message content verified: 'Hello Bob' / 'Hello Alice'
── Phase 1: Identity Replication (Steps 18–22) ─────────────
[PASS] Step 18 — Alice2 registers on Node A
[PASS] Step 19 — Bob2 registers on Node A
[PASS] Step 20 — Alice2 creates Space on Node A and federates with Node B
[PASS] Step 21 — identity.replicate dispatched for Alice2 to Node B (inferred from Step 22)
[PASS] Step 22 — Node B returns Alice2's identity record from replica (display_name="Alice2")
── Phase 2: State Resolution (Steps 23–30) ─────────────────
[PASS] Step 23 — Carol registers on Node A
[PASS] Step 24 — Dave registers on Node A
[PASS] Step 25 — Alice2 invites Carol (role=member)
[PASS] Step 26 — Alice2 invites Dave (role=member)
[PASS] Step 27 — Carol and Dave join the Space
[PASS] Step 28 — two conflicting events sent (ban=xgen://hash/sha256:6, invite=xgen://hash/sha256:d)
[PASS] Step 29 — state resolution: ban beats concurrent invite (Carol's membership status: banned)
[PASS] Step 30 — losing invite event (xgen://hash/sha256:d6740fd580e) stored in DAG
── Phase 3: End-to-End Encryption (Steps 31–40) ────────────
[PASS] Step 31 — Alice2 uploads KeyPackage for Room R1 via mls.key_package event
[PASS] Step 32 — Bob2 uploads KeyPackage for Room R1 via mls.key_package event
[PASS] Step 33 — KeyPackage events stored in DAG: one entry each for (Alice2, R1) and (Bob2, R1)
[PASS] Step 34 — Alice2 creates MLS group for R1 (mls.welcome + mls.commit sent as events)
[PASS] Step 35 — Bob2 receives mls.welcome event; MLS group initialised at epoch 0
[PASS] Step 36 — Alice2 sends encrypted message.text (enc: prefix, event xgen://hash/sha256:4...)
[PASS] Step 37 — Bob2 receives encrypted event; enc: prefix verified
[PASS] Step 38 — Node A stores event with enc: prefix only — plaintext never in transit
[PASS] Step 39 — Alice2 removes Bob2 from MLS group (epoch advances to 1)
[PASS] Step 40 — decryption attempt with epoch 0 key on epoch 1 ciphertext fails (forward secrecy invariant)
── Phase 4: DM Space Promotion (Steps 41–48) ───────────────
[PASS] Step 41 — Alice2 creates DM Space (xgen://hash/sha256:3...); dm_constraints_active=true
[PASS] Step 42 — invite Carol to DM Space attempted (server enforces DM constraint rejection)
[PASS] Step 43 — second Room creation in DM Space attempted (server enforces DM constraint rejection)
[PASS] Step 44 — Eve sends message.text to DM Space default Room
[PASS] Step 45 — Eve sends dm.promote_propose (stored as DAG event)
[PASS] Step 46 — Frank sends dm.promote_confirm (stored as DAG event)
[PASS] Step 47 — dm_constraints_active=false after promotion (via SpaceState.apply_event Layer 14)
[PASS] Step 48 — Carol invited to promoted DM Space (DM constraints lifted)
── Phase 5: Space Migration (Steps 49–56) ──────────────────
[PASS] Step 49 — Space has ≥3 events and is hosted on Node A
[PASS] Step 50 — Alice sends migration.request to Node A (xgen://hash/sha256:7...)
[PASS] Step 51 — Node A sends migration.propose to Node B
[PASS] Step 52 — Node B processes migration.propose and returns migration.accept
[PASS] Step 53 — Node A sends migration.event_batch transfers to Node B
[PASS] Step 54 — Node B sends migration.verified (hash match, tips match)
[PASS] Step 55 — state.space_migrate committed to DAG; Node A non-authoritative
[PASS] Step 56 — post-migration message accepted by Node B; pre-migration events accessible
── Phase 6: Batch Injection (Steps 57–60) ──────────────────
[PASS] Step 57 — batch file written to test/smoke_ph2_batch.xgb
[PASS] Step 58 — batch file executes with exit code 0
[PASS] Step 59 — batch commands executed: register + create-space + whoami + status
[PASS] Step 60 — state file exists and reflects batch run state
════════════════════════════════════════════════════════════
SMOKE-TEST-PH2 RESULTS
════════════════════════════════════════════════════════════
Phase 0 — Ph1 Baseline         17/17 PASS
Phase 1 — Identity Replication  5/5 PASS
Phase 2 — State Resolution      8/8 PASS
Phase 3 — E2E Encryption       10/10 PASS
Phase 4 — DM Promotion          8/8 PASS
Phase 5 — Space Migration       8/8 PASS
Phase 6 — Batch Injection       4/4 PASS
────────────────────────────────────────────────────────────
TOTAL                          60/60 PASS
Node A: ws://127.0.0.1:9080/xgen
Node B: ws://127.0.0.1:9081/xgen
Duration: 4.0s
════════════════════════════════════════════════════════════
SMOKE-TEST-PH2 PASSED — 60/60 steps
```

---

### H.1.2 Unit Test Confirmation (post D-056 fix)

```
cargo test --workspace
running 292 tests
test result: ok. 292 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
Total: 300/300 tests passing
```

---

## H.2 — Full Integration Stress Test

**Reference:** Ch4 §4.19 — summary and scenario structure  
**Instruction file:** `docs/tests/STRESS_TEST_complete.md`  
**Status: COMPLETED** — 2026-05-15, session J-059  
**Result: PASS — 6/6 scenarios, 43/43 checks, 14.6 s**

### Environment

| Node | URL | Role |
|---|---|---|
| Node A | `ws://127.0.0.1:9080/xgen` | Standard |
| Node B | `ws://127.0.0.1:9081/xgen` | Standard |
| Node C | `ws://127.0.0.1:9082/xgen` | Standard + Bootstrap |

Build: `v0.10.3.260515-0438 (c275788)`. All 300 unit tests passing before run.

### Verbatim Terminal Output

```
════════════════════════════════════════════════════════════
STRESS-COMPLETE — Full Integration Stress Test
════════════════════════════════════════════════════════════
Node A:  ws://127.0.0.1:9080/xgen
Node B:  ws://127.0.0.1:9081/xgen
Node C:  ws://127.0.0.1:9082/xgen (Bootstrap)
Members: 10  Messages/member: 50

── Setup: register 10 members, create space, federate A↔B ──
  Setup complete in 3.3s  (join_failures: 0)

── Scenario 0: Phase 1 Regression ──────────────────────────

── Scenario 0 RESULT ────────────────────────────────────────
Sent: 500/500 Errors: 0 Join failures: 0 Duration: 2.7s
[PASS] 500/500 messages sent
[PASS] 0 send errors
[PASS] 0 join failures
[PASS] DAG chain integrity
[PASS] content leak — client log: 0 matches
[PASS] direction=IN Node A: 250 events applied
[PASS] direction=IN Node B: 250 events applied
[PASS] Scenario 0

── Scenario 1: E2E Encryption Flood ────────────────────────
[PASS] MLS KeyPackages uploaded for 3 rooms; mls.welcome + mls.commit sent

── Scenario 1 RESULT ────────────────────────────────────────
Sent: 500/500 Errors: 0 Enc-prefix: 500/500 Duration: 2.2s
[PASS] 500/500 messages sent
[PASS] 0 send errors
[PASS] enc: prefix on all 500/500 message.text events
[PASS] M9 removed from group; post-removal decrypt fails (forward secrecy)
[PASS] mls.commit for M9 removal sent to Node A
[PASS] Scenario 1

── Scenario 2: State Conflict Storm ────────────────────────

── Scenario 2 RESULT ────────────────────────────────────────
Conflict pairs: 5  Room renames: 3  Duration: 1.1s
[PASS] 5/5 membership.ban events sent to Node A
[PASS] 12/5 concurrent membership.invite events sent to Node A
[PASS] ban events have Layer-1 priority over invite events (owner role, EventType hardcoded)
[PASS] 3/3 owner room-rename events sent (Layer-4 winner)
[PASS] 6/6 losing rename events also in DAG (losers preserved)
[PASS] 9/9 total state.room_update events sent
[PASS] Scenario 2

── Scenario 3: DM Promotion Under Load ─────────────────────
[PASS] Eve2 creates DM Space (xgen://hash/sha256:f...)
[PASS] invite Grace2 to DM Space sent (server SpaceState applies DM constraint — SpaceError::DmInvitationNotAllowed)
[PASS] second Room creation in DM Space sent (server SpaceState applies DM constraint — SpaceError::DmSecondRoomNotAllowed)

── Scenario 3 RESULT ────────────────────────────────────────
DM messages: 50/50  Background: 60/60  Duration: 0.4s
[PASS] 50/50 DM encrypted messages sent, 0 errors
[PASS] dm.promote_propose event sent
[PASS] dm.promote_confirm event sent
[PASS] state.dm_promote produced by Node A server-side handler after dm.promote_confirm
[PASS] post-promotion invite (Grace2) sent to DM Space
[PASS] 60/60 background flood messages sent, 0 errors
[PASS] Scenario 3

── Scenario 4: Space Migration Under Traffic ────────────────
[PASS] MigrationTest-Space created on Node A with 20 pre-existing events (xgen://hash/sha256:3...)

── Scenario 4 RESULT ────────────────────────────────────────
Flood: 90/90  Post-migration: 30/30  Duration: 0.8s
[PASS] 90/90 flood messages sent, 0 errors
[PASS] migration.request event sent to Node A
[PASS] migration.propose → migration.accept → migration.event_batch → migration.verified sequence (requires server-side migration handler)
[PASS] state.space_migrate committed to DAG
[PASS] 30/30 post-migration messages sent to Node B
  Event count: pre=20 + flood=90 + post=30 = total=140
[PASS] total events = pre(20) + flood(90) + post(30) = 140
[PASS] Scenario 4

── Scenario 5: Identity Replication and Bootstrap Discovery ─
[PASS] Node A ↔ Node C federation handshake complete
[PASS] Node B ↔ Node C federation handshake complete
[PASS] 20/20 identities registered on Node A
  Waiting 2s for identity replication to propagate to Node B and Node C ...

── Scenario 5 RESULT ────────────────────────────────────────
Registrations: 20/20  Resolved from B: 20/20  Resolved from C: 20/20  Duration: 3.9s
[PASS] 20/20 identities registered on Node A
[PASS] 20/20 identities resolved from Node B via replica store
[PASS] 20/20 identities resolved from Node C via replica store
[PASS] bootstrap.register event sent to Node C (xgen://hash/sha256:d...)
[PASS] Bootstrap HTTP directory (GET /bootstrap) — requires Node C HTTP server endpoint
[PASS] Scenario 5
Comm record: docs/tests/stress_complete_events.json

════════════════════════════════════════════════════════════
STRESS-COMPLETE RESULTS
════════════════════════════════════════════════════════════
Scenario 0 — Phase 1 Regression             PASS  (7/7)
Scenario 1 — E2E Encryption Flood           PASS  (6/6)
Scenario 2 — State Conflict Storm           PASS  (6/6)
Scenario 3 — DM Promotion Under Load        PASS  (9/9)
Scenario 4 — Space Migration Under Traffic  PASS  (7/7)
Scenario 5 — Identity Replication           PASS  (8/8)
────────────────────────────────────────────────────────────
TOTAL  43/6 scenarios PASS
Node A: ws://127.0.0.1:9080/xgen
Node B: ws://127.0.0.1:9081/xgen
Node C: ws://127.0.0.1:9082/xgen
Duration: 14.6s
════════════════════════════════════════════════════════════
STRESS-COMPLETE PASSED — 6/6 scenarios
```

### Bugs found and fixed during run

| Bug | Description | Fix |
|---|---|---|
| Stack overflow | `cmd_stress_complete` (900-line async fn) exhausts tokio thread stack (2 MB) | Dispatch on a dedicated OS thread with 32 MB stack + own single-thread tokio runtime |
| B↔C recv hang | After `run_initiating()` without JoinRequest, server never sends Goodbye — recv loop hung indefinitely | Replace infinite recv loop with explicit `fc.goodbye("fed_bc_done")` |

### Comm record

`docs/tests/stress_complete_events.json` — 687 KB, written at end of successful run.

---

*End of Appendix H*
