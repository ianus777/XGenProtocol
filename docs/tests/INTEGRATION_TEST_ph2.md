# XGen Protocol — Phase 2 Integration Smoke Test
> **Status:** PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated:** 2026-05-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## ⚠️ PROJECT ROOT — Read before opening any file

All work in this task file is performed in the project root:

**`E:\Projects\XGenProtocol`**

Before starting any work, confirm your working directory:

```
echo %CD%
```

Expected output: `E:\Projects\XGenProtocol`

**If your shell shows `.claire\worktrees\`, `.claude\worktrees\`, or any other path — stop.
Navigate to `E:\Projects\XGenProtocol` before touching a single file.**

All file paths in this document are relative to `E:\Projects\XGenProtocol`. Do not create,
edit, or compile from worktrees, temp directories, or any subdirectory unless explicitly stated.

---

## Purpose

This task verifies that all Phase 1 and Phase 2 protocol layers work correctly as a running
system — two live Node binaries, real WebSocket connections, real event signing and DAG storage.
Unit tests verified individual components. This test verifies they compose correctly.

It also serves as the first end-to-end test of the `--batch` command injection pipeline for
the CLI binary, which has not been exercised against the compiled binary before.

The output of this test (step-by-step PASS/FAIL with event IDs and timings) is pasted
directly into `JOURNAL.md` as the Phase 2 integration evidence.

---

## Prerequisites

- All Phase 2 layers (11–19) complete ✅
- `cargo test` passing: 300/300 ✅
- Two Node binaries compiled and present in `bin/`: `xgen-node.exe`, `xgen-client.exe`
- If binaries are stale, run `build.sh` or:
  ```
  set CARGO_TARGET_DIR=C:/cargo-targets/XGenProtocol
  cargo build --release
  copy C:\cargo-targets\XGenProtocol\release\xgen-node.exe bin\
  copy C:\cargo-targets\XGenProtocol\release\xgen-client.exe bin\
  ```

---

## Part A — CLI Extensions (implement before running any test)

The existing `xgen-client` CLI binary requires two additions before the integration test
can run. Implement both in `xgen-client/src/main.rs`.

### A.1 — Add `--batch <file.xgb>` global flag to the CLI binary

The `--batch` flag already exists on the Tauri app (`xgen-client-app`) using named pipe IPC.
For the CLI binary, implement a simpler direct-execution mode:

- Read the `.xgb` file line by line
- Skip blank lines and lines starting with `#`
- Parse each line as a CLI subcommand (same syntax as interactive use)
- Execute each command in sequence against the configured node endpoint
- On first failure, print the failed command and exit with code 1
- On success of all commands, exit with code 0

**Add to `Cli` struct in `main.rs`:**

```rust
/// Execute a batch command file (.xgb) sequentially and exit.
/// Each line is a CLI subcommand. Blank lines and # comments are ignored.
/// Exits 0 if all commands succeed; exits 1 on first failure.
#[arg(long, value_name = "FILE")]
batch: Option<PathBuf>,
```

**Dispatch logic:** after parsing `Cli`, if `batch` is `Some(path)`, read the file,
parse each non-empty non-comment line as a `ClientCommand` using `clap`'s `try_parse_from`,
and dispatch each command exactly as the interactive path does. Do not open a window.
Do not use named pipes. This is a direct sequential executor.

**Error handling:** if the file does not exist or has a `.xgb` extension check fail,
print a clear error and exit with code 2. If a command line fails to parse, print the
failing line and exit code 1.

**Update `docs/xgen_appendix_f_en.md` §F.3** to document the `--batch` flag for the CLI binary,
noting that it is a direct sequential executor (no running instance required), distinct from
the Tauri app's named-pipe batch mode.

### A.2 — Add `smoke-test-ph2` subcommand

Add to the `ClientCommand` enum:

```rust
/// Run the Phase 2 integrated smoke test against two running Node instances.
/// Exercises all Phase 1 and Phase 2 protocol layers end-to-end over real TCP.
/// Produces structured PASS/FAIL output for each step.
SmokePh2(SmokePh2Args),
```

```rust
#[derive(Args)]
struct SmokePh2Args {
    /// Endpoint of Node A. Example: ws://127.0.0.1:8080/xgen
    #[arg(long)]
    node_a: String,
    /// Endpoint of Node B. Example: ws://127.0.0.1:8081/xgen
    #[arg(long)]
    node_b: String,
    /// Do not clean up test identities and spaces after the run.
    #[arg(long)]
    keep: bool,
}
```

Implement in a new function `cmd_smoke_ph2(args: &SmokePh2Args)` following the same
pattern as `cmd_smoke_test`. See Phase B below for the full step sequence.

---

## Part B — smoke-test-ph2 Step Sequence

The command runs 60 steps across 7 phases. Each step prints:

```
[PASS] Step 23 — state resolution: ban beats concurrent invite (winner: xgen://hash/sha256:abc...)
[FAIL] Step 23 — state resolution: ban beats concurrent invite — expected ban to win, got invite
```

If any step fails, print the failure, then print `SMOKE-TEST-PH2 FAILED at step N` and exit 1.
If all steps pass, print `SMOKE-TEST-PH2 PASSED — 60/60 steps` and exit 0.

---

### Phase 0 — Phase 1 Baseline (Steps 1–17)

Re-run the existing 17-step Phase 1 smoke test logic (reuse `run_smoke_test_inner` or equivalent).
All 17 steps must pass before Phase 1 proceeds. If any Ph1 step fails, abort immediately —
the baseline is broken and Ph2 results are meaningless.

**Step 1–17:** same as existing `smoke-test` command.

---

### Phase 1 — Identity Replication (Steps 18–22)

**Setup:** fresh identities Alice2 and Bob2 on Node A. Federate with Node B.

**Step 18:** Register Alice2 on Node A. Expect `identity.register_ok`.

**Step 19:** Register Bob2 on Node A. Expect `identity.register_ok`.

**Step 20:** Alice2 creates a Space on Node A. Initiate federation with Node B
(`federation.hello` → `federation.accept` → `ACTIVE` session).

**Step 21:** Send `identity.replicate` for Alice2 from Node A to Node B (simulated by
querying the federation session and verifying the replication event was dispatched —
check that `identity/replication.rs` pushed the record). This step passes if
`identity.replicate` is present in Node A's outbound event log for the Node B session.

**Step 22:** Query Node B for Alice2's identity record via a direct WebSocket request.
Expect the record to be returned from Node B's local replica store (not proxied to Node A).
Step passes if `identity_id` matches Alice2's key URI and `display_name` is `"Alice2"`.

---

### Phase 2 — State Resolution (Steps 23–30)

**Setup:** Carol and Dave registered on Node A. Both invited to a test Space by Alice2.
Both join. Now trigger concurrent conflicting events.

**Step 23:** Register Carol on Node A. Expect `identity.register_ok`.

**Step 24:** Register Dave on Node A. Expect `identity.register_ok`.

**Step 25:** Alice2 invites Carol (`membership.invite`, role=member). Expect accept.

**Step 26:** Alice2 invites Dave (`membership.invite`, role=member). Expect accept.

**Step 27:** Carol and Dave both join. Expect two `membership.join` accepts.

**Step 28:** Send two conflicting events simultaneously with the same `prev_events` tip:
- Event X: `membership.ban` targeting Carol (sent by Alice2, role=owner)
- Event Y: `membership.invite` targeting Carol again (sent by Dave, role=member, same prev_event)
Both share the same `prev_events` entry — this is a real DAG conflict on state key
`(membership, carol_identity_id)`.

Verify both events are accepted into the DAG (neither rejected outright).

**Step 29:** Query Node A space state. Expect Carol's membership status is `banned`
(Layer 1 of resolution: `membership.ban` beats `membership.invite`).

**Step 30:** Verify Event Y (the losing invite) is still present in the DAG with its
original event_id — the loser is stored, not discarded.

---

### Phase 3 — End-to-End Encryption (Steps 31–40)

**Setup:** Alice2 and Bob2 in a shared Space and Room (created in Phase 1 setup).

**Step 31:** Alice2 uploads a KeyPackage for Room R1 via `mls.key_package`.
Expect `mls.key_package_ack`.

**Step 32:** Bob2 uploads a KeyPackage for Room R1 via `mls.key_package`.
Expect `mls.key_package_ack`.

**Step 33:** Verify KeyPackage store: Node A holds one entry for each of
(Alice2, R1) and (Bob2, R1).

**Step 34:** Alice2 creates MLS group for R1. Alice2 fetches Bob2's KeyPackage,
constructs `mls.welcome` + `mls.commit`. Node A routes `mls.welcome` to Bob2.

**Step 35:** Bob2 receives `mls.welcome`. Verify Bob2's group state initialised
at epoch 0. Verify Alice2's KeyPackage was deleted from Node A after distribution
(one-time-use).

**Step 36:** Alice2 sends an encrypted `message.text` to R1. The `content` field
is an `enc:` prefixed blob. Node A stores and propagates without decryption.

**Step 37:** Bob2 receives the encrypted event. Bob2 decrypts using MLS epoch key.
Step passes if decrypted plaintext matches the original message.

**Step 38:** Verify Node A never had the plaintext — event_trace log for this event
must have an empty `content` field.

**Step 39:** Alice2 removes Bob2 from the group (`mls.commit` with remove operation).
Group epoch advances to 1.

**Step 40:** Alice2 sends a second encrypted message at epoch 1. Bob2 attempts
decryption with epoch 0 key. Step passes if decryption fails
(`wrong_epoch_key_fails_decryption` path — Bob is out of the group).

---

### Phase 4 — DM Space Promotion (Steps 41–48)

**Setup:** fresh identities Eve and Frank on Node A.

**Step 41:** Alice2 creates a DM Space (`state.dm_space_create`) between Eve and Frank.
Expect both are members, `dm_constraints_active = true`.

**Step 42:** Attempt to invite a third identity (Carol) into the DM Space.
Expect rejection with error code in the 9000 range.

**Step 43:** Attempt to create a second Room in the DM Space.
Expect rejection with error code in the 9000 range.

**Step 44:** Eve sends a message in the DM Space default Room. Expect `message.text` accepted.

**Step 45:** Eve sends `dm.promote_propose` to Node A.
Verify proposal stored on Node A and `dm.promote_propose` delivered to Frank's session.

**Step 46:** Frank sends `dm.promote_confirm`.
Verify Node A produces `state.dm_promote` event signed by the Node keypair (not by Frank).
Verify `state.dm_promote` committed to Space DAG.

**Step 47:** Verify `dm_constraints_active = false` on Space state after promotion.

**Step 48:** Attempt to invite Carol again. This time expect success — DM constraints lifted.

---

### Phase 5 — Space Migration (Steps 49–56)

**Setup:** use Alice2's Space from Phase 1, which is federated with Node B and has messages.

**Step 49:** Verify the Space currently has at least 3 events and is hosted on Node A.

**Step 50:** Alice2 (Space owner) sends `migration.request` to Node A, requesting
migration to Node B.

**Step 51:** Verify Node A sends `migration.propose` to Node B.

**Step 52:** Node B processes `migration.propose`. Verify `migration.accept` returned.

**Step 53:** Verify Node A begins sending `migration.event_batch` transfers to Node B.
Verify event count in each batch matches transfer spec (3.12.5).

**Step 54:** After all batches sent, verify Node B sends `migration.verified` back to Node A
(hash match, tips match, event count match).

**Step 55:** Verify `state.space_migrate` event in DAG. Verify Node A marks the Space
as migrated (no longer authoritative).

**Step 56:** Verify: Alice2 can send a new message to the Space via Node B after migration.
Expect `message.text` accepted on Node B. Verify all pre-migration events still accessible.

---

### Phase 6 — Batch Injection (Steps 57–60)

This phase tests the `--batch` CLI flag implemented in Part A.

**Step 57:** Write a temp `.xgb` file at `test/smoke_ph2_batch.xgb` containing:
```
# smoke-test-ph2 batch injection test
register --name "BatchTestUser" --node ws://127.0.0.1:8080/xgen
create-space --name "BatchTestSpace" --node ws://127.0.0.1:8080/xgen
whoami
status
```

**Step 58:** Run:
```
xgen-client --batch test/smoke_ph2_batch.xgb
```
Expect exit code 0. Capture stdout.

**Step 59:** Verify stdout contains `BatchTestUser` (from `whoami` output) and
`BatchTestSpace` (from space creation confirmation).

**Step 60:** Verify the state file written by the batch run reflects the registered
identity and the created Space. Step passes if both are present in the state file.

---

## Output Format

After all 60 steps, print a summary block:

```
════════════════════════════════════════════════════════════
SMOKE-TEST-PH2 RESULTS
════════════════════════════════════════════════════════════
Phase 0 — Ph1 Baseline         17/17 PASS
Phase 1 — Identity Replication  5/5  PASS
Phase 2 — State Resolution      8/8  PASS
Phase 3 — E2E Encryption       10/10 PASS
Phase 4 — DM Promotion          8/8  PASS
Phase 5 — Space Migration       8/8  PASS
Phase 6 — Batch Injection       4/4  PASS
────────────────────────────────────────────────────────────
TOTAL                          60/60 PASS
Node A: ws://127.0.0.1:8080/xgen
Node B: ws://127.0.0.1:8081/xgen
Duration: 12.4s
════════════════════════════════════════════════════════════
```

**This full output block must be pasted verbatim into `JOURNAL.md`.** Do not paraphrase.
Do not summarise. Paste the actual output.

---

## Running the Test

Prerequisites: Node A and Node B must be running before invoking the command.

```powershell
# Terminal 1 — Node A
cd E:\XGen\NodeA
xgen-node

# Terminal 2 — Node B
cd E:\XGen\NodeB
xgen-node

# Terminal 3 — Run the test
cd E:\Projects\XGenProtocol
bin\xgen-client smoke-test-ph2 --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen
```

---

## Verification

After `smoke-test-ph2` exits 0:

1. `cargo test` — must still show 300/300 (no regressions from Part A changes)
2. `cargo build --release` — must compile cleanly, no warnings
3. Paste the full 60-step output into `JOURNAL.md`
4. Confirm `.xgb` batch injection exit code 0 output is in the journal entry

---

## Definition of Done

- [ ] `--batch` flag implemented on CLI binary and documented in Appendix F §F.3
- [ ] `smoke-test-ph2` subcommand implemented
- [ ] `cargo test` — 300/300 passing, 0 warnings
- [ ] `cargo build --release` — clean
- [ ] `smoke-test-ph2` exits 0, all 60 steps PASS
- [ ] Full output block pasted verbatim into `JOURNAL.md`
- [ ] `CLAUDE.md` updated: integration test marked COMPLETE with journal reference
- [ ] `DECISIONS.md` updated with any decisions made during Part A implementation
- [ ] Committed and pushed
