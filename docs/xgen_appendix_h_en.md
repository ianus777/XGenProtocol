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
**Status: PENDING** — to be filled after the stress test run completes.

---

*End of Appendix H*
