# MP-F14 — regular-Space pre-join-message backfill — runbook (exec-step-1 shape)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

The implementation runbook for **MP-F14**, the sole MP-R3 fix-phase gate item (J-351). It executes the
Joe-locked design (`tasks/MP_F14_PREJOIN_BACKFILL_DESIGN.md`, MP-F14-D1..D7) in the **MP-F9 exec-step-1
shape**: the **first step is a box-gated re-trace** that locks the fork (MP-F14-D2) against observation,
*then* the fix + spine + enrichment + witness. No code beyond this runbook until Joe-locks it; the fix
commit waits on the exec-step-1 fork lock.

**Locked at design (consume, do not re-litigate):**
- **D1** — the certain gap: federation push is send-time-only; no outbound re-stream when
  `federation_nodes[S]` grows over a live session. The invariant the fix establishes: outbound catch-up
  to a **legitimate** peer on growth, **F-3 intact**.
- **D3** — exec-step-1 = the box-gated re-trace (the 4-point diagnostic); spine in-process RED-on-revert;
  witness box-gated.
- **D4** — the J-333 hole-safety spine (mandatory, RED-on-revert hole-closed, mirrors C5 `mp_f11_*`).
- **D5** — the MP-C-14 leaf-content coverage enrichment.
- **D6** — convergence-safety / D-076 discharged-by-inheritance; DM untouched; scope fence.
- **D7** — the trace is authoritative; a falsification (delivery-bug) re-shapes the mechanism, gate
  disposition unchanged.

**Conditional (the runbook resolves it):**
- **D2** — the fork. **Recommended = Fork A** (outbound re-stream at the relationship/`federation_nodes`-
  growth point). **Locks at the §2 exec-step-1 checkpoint** against the captured trace, read against the
  design §4.3 trace→fork table. May flip to **Fork B** (member-join-driven) or the **§8/D7 delivery-bug
  branch**.

**Method:** surface-and-route (D-065/D-084); pin-by-observation BEFORE the fork lock; spine-first before
the box witness (the C5 split). Clair's commits FIRST (Joe pushes); Chat's doc-bridges separate.

---

## 1. Commit plan (overview)

| # | Beat | Crate(s) | Box? | Gate |
|---|---|---|---|---|
| **§2** | **Exec-step-1 — box-gated re-trace + fork lock** | (throwaway diagnostic; reverted) | **box** | **Joe-lock checkpoint #1 (the fork)** |
| **§3 / C1** | The fix + the J-333 spine (RED-on-revert, in-process) | `xgen-core` + `xgen-node` | box-free | per-commit DoD |
| **§4 / C2** | The D5 coverage enrichment (gate leaf sends; 3-class oracle) | `xgen-mptest` | box-free | per-commit DoD |
| **§5** | Box-gated witness — MP-C-14 green → R3 rerun to all-green-except-MP-C-16 | (run) | **box** | rerun-to-criterion |
| **§6** | MP-R3 close + the consolidated R1+R2+R3 ledger (Chat bridge) | (docs) | — | milestone close |

**Sequencing rule (MP-F9):** **no §3 production code until §2's fork is Joe-locked.** The fix *site*
depends on the locked fork; locking it before the trace is the exact failure MP-F9 caught.

---

## 2. Exec-step-1 — the box-gated re-trace (D3) → the fork lock

**Purpose:** pin which growth-point is live in MP-C-14 (audit §3 / design §4.3) and lock MP-F14-D2.
This is a **throwaway diagnostic** (keep-artifacts; reverted after — the MP-F11/MP-F13 pin precedent),
on production-crate instrumentation, run against the real `mp_r3_topology` scenario. **No production fix
lands here.**

### 2.1 The 4-point diagnostic (exact instrumentation sites, D-078)

Add bounded `tracing::warn!` (so it surfaces under `--nocapture`), keyed by `(space_id, event_id,
node)` for cross-binary correlation:

1. **Push snapshot** — inside `apply_federation_push`
   ([`federation_session.rs:331-337`](../xgen-node/src/federation_session.rs#L331), the
   `federation_nodes` read) + the `try_send` loop
   ([`:380`](../xgen-node/src/federation_session.rs#L380)): log `local_node_id`, `event_id`, the **full
   `federation_nodes[S]` set at the push instant**, and the peers actually sent to. → *Was the missing
   leaf's node in the set when `p0` was pushed?*
2. **Establish firing** — inside `establish_federation_relationship`
   ([`runtime.rs:1983`](../xgen-core/src/node/runtime.rs#L1983)) + its hook
   ([`federation_session.rs:116-120`](../xgen-node/src/federation_session.rs#L116)): log `(space, peer,
   federation_nodes before, federation_nodes after, drained_count)`. → *Did the missing leaf's establish
   fire (candidate i vs ii)? when, relative to `p0`? did the surrounding `stream_federation_delta`
   outbound stream run?*
3. **`federation_add` growth** — `apply_federation_add`
   ([`state.rs:699`](../xgen-core/src/space/state.rs#L699), the `federation_nodes.push`) + the Step-7
   drain pair ([`runtime.rs:1570`](../xgen-core/src/node/runtime.rs#L1570) /
   [`:1619-1621`](../xgen-core/src/node/runtime.rs#L1619)): log the peer added + a relative timestamp. →
   *Is the growth event-driven (`federation_add`) and does it post-date `p0`?*
4. **The miss** — at settle, surface the convergence verdict's divergence
   ([`oracle.rs:259`](../xgen-mptest/src/oracle.rs#L259) membership / `oracle.rs:274` transcript) **plus
   a per-node transcript diff naming the missing `event_id`** (the verdict names the nodes; add the
   specific missing id). → *Which node ends missing `p0`?*

### 2.2 Run + capture

```text
cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
# re-run until a FAILING run is captured (~60% fail at default settle; the finding pinned ×8):
cargo test -p xgen-mptest --test mp_r3_topology -- --ignored --nocapture
```

Capture a **failing** run's trace (the ~60% case; run several times). Correlate points 1–4 for the
victim node missing `p0`.

### 2.3 Read against the design §4.3 table → Joe-lock the fork (checkpoint #1)

| Captured trace | Reading | Lock |
|---|---|---|
| establish (S, missing-leaf) **never fires**; `federation_nodes[S]`←leaf comes (if at all) from a `federation_add` post-`p0` | candidate (i): re-initiate reused the pre-seeded session | **Fork A**, trigger = the `federation_add` growth point (+ outbound re-stream) |
| establish (S, leaf) **fires after `p0`** but `p0` not carried outbound / leaf still misses it | candidate (ii-b): establish populated but no outbound catch-up | **Fork A**, trigger = the establish hook gains the outbound re-stream |
| establish (S, leaf) **fires before `p0`**, leaf IS in `federation_nodes[S]` at `p0`-push, yet leaf misses `p0` | **delivery** bug, not backfill | **§8 / D7 falsification** — re-shape to the delivery path; re-lock |
| growth purely membership-driven; no establish/`federation_add` adds the leaf's node | gap-2 at the membership layer | **Fork B** |

**Output of §2:** MP-F14-D2 locked (Fork A expected, by-recomm, against the trace) + the **exact trigger
site** named for §3. Revert the diagnostic (clean tree). **Then** §3 proceeds.

**Falsification handling (D7):** if the trace shows a delivery bug (row 3), re-pin the finding (delivery,
not backfill), re-shape §3's fix to the delivery path, and re-author §3's spine accordingly. The gate
disposition is **unchanged** (MP-F14 stays a fix-in-round gate item, terminal = GREEN-on-rerun). Any
**orthogonal** bug the trace surfaces faces-and-routes to its own home (sibling MP-F12) and does **not**
extend the {MP-F14} gate.

---

## 3. Commit 1 — the fix + the J-333 spine (box-free, in-process RED-on-revert)

The production-crate change + its spine, proven RED-on-revert **before** the box witness (the C5 split).
**Spine-first:** the two spine tests are authored with the fix and must go RED when the fix is neutered.

### 3.1 The fix (the §2-locked fork; the outbound re-stream trigger)

**Fork A (expected):** at the §2-locked growth point (the establish hook, or the `federation_add`
growth/drain-pair on the live path), add an **outbound catch-up re-stream** of the Space's existing
content to the newly-added legitimate peer — **in addition to** the inbound drain
`establish_federation_relationship` already does.

- **Reused machinery (no re-invention):** full history via
  `compute_federation_delta_for_space(rt, &space_id, None)`
  ([`fanout.rs:605`](../xgen-node/src/fanout.rs#L605), `None` ⇒ full topo-sorted history,
  canonical-sorted [`:615-627`](../xgen-node/src/fanout.rs#L615)); send via the peer's
  `FederationPeerSenders` channel (the same registry + `try_send(OutboundMsg::Event)`
  `apply_federation_push` uses, [`federation_session.rs:378-384`](../xgen-node/src/federation_session.rs#L378));
  the relationship record + F-3 authority + inbound drain stay `establish_federation_relationship`
  ([`runtime.rs:1983`](../xgen-core/src/node/runtime.rs#L1983)); rebuild survival via
  `repopulate_regular_federation_nodes` ([`runtime.rs:2110`](../xgen-core/src/node/runtime.rs#L2110),
  unchanged).
- **Crate split:** the legitimacy/F-3 authority + the growth signal stay `xgen-core`; the **outbound
  re-stream is `xgen-node`** (it needs `FederationPeerSenders` + `compute_federation_delta_for_space`,
  node-side). If the growth originates `xgen-core` (a `federation_add` apply / `establish_*`), the
  "peer P needs catch-up for S" signal flows to `xgen-node` (mirroring the existing inbound
  `drain → persist` split at [`federation_session.rs:116-135`](../xgen-node/src/federation_session.rs#L116)).
- **ES-D4:** the re-streamed events are already persisted on the sender (it is the sender's own
  history); the re-stream is delivery-only (no new persist on the sender). The receiver persists on
  apply (its normal path), as today.
- **F-5 / F-3 untouched:** the outbound re-stream is a locally-originated catch-up to a legitimate peer
  (not a re-forward of a `ReceivedViaFederation` event — the F-5 guard
  [`federation_session.rs:301`](../xgen-node/src/federation_session.rs#L301) is not weakened); the
  receiver's F-3 gate ([`runtime.rs:1057-1071`](../xgen-core/src/node/runtime.rs#L1057)) accepts the
  re-streamed content because the peer is a legitimate `federation_nodes[S]` member by construction.

*(Fork B, if §2 locks it: the trigger is the cross-node member-join apply for a regular Space — a
regular-Space sibling of `repopulate_dm_federation_nodes`; same outbound re-stream machinery, different
trigger site. The runbook's §3.2/§3.3 test shapes carry over; the home shifts to the membership-apply
path.)*

### 3.2 The two J-333 spine tests (D4, mandatory, RED-on-revert)

**Names finalize with the §2 fork** (the trigger site determines the exact in-process repro), but the
shapes are fixed:

- **Spine #1 — `mp_f14_backfill_on_federation_nodes_growth_re_streams_existing_content`** (`xgen-node`,
  the `federation_push_integration.rs` mock-receiver pattern — `FederationPeerSenders` +
  `mpsc::channel::<OutboundMsg>(1024)`, template
  [`alice_post_propagates_to_bob_via_federation_push`](../xgen-node/src/tests/federation_push_integration.rs#L194)):
  A holds content `c` in a regular Space posted while peer `B` is **NOT** in `federation_nodes[S]`;
  then `B` legitimately grows into `federation_nodes[S]` (the §2 trigger); assert **`B`'s `OutboundMsg`
  channel receives `c`** (the re-stream fired). **RED-on-revert:** neuter the outbound re-stream → `B`'s
  channel stays empty → test fails.
- **Spine #2 — `mp_f14_third_party_not_re_streamed_on_growth`** (`xgen-node`) **+** the F-3 receiver
  intact: growing for a legitimate peer `A` does **NOT** send `c` to a third party `B` (not in
  `federation_nodes[S]` / not a member); `B`'s channel stays empty and `B` never enters
  `federation_nodes[S]`. **RED-on-revert:** an over-broad re-stream (to all peers / skipping the
  legitimacy check) → `B`'s channel gets `c` → test fails. **Plus a no-regression assertion:** C5's
  existing [`mp_f11_third_party_regular_space_content_blocked_by_f3`](../xgen-core/src/node/runtime.rs#L3048)
  (the receiver-side F-3 block) **stays green** (the fix must not weaken F-3 on ingest). Sibling to C5's
  hole-closed spine.

### 3.3 Commit 1 DoD (D-078)

- `cargo build --workspace --all-targets` 0-error.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean (default + harness-control).
- `xgen-core` + `xgen-node` fast suites 0-failed; the two spine tests GREEN; **RED-on-revert demonstrated
  and recorded** for both (neuter → RED → restore → GREEN).
- C5's `mp_f11_*` spine + the MP-F1b DM spine + J-298 INV-EXP **byte-unaffected** (DM untouched — D6).
- D-076 discharge note: the re-stream reuses the canonical full-history delta
  ([`fanout.rs:615-627`](../xgen-node/src/fanout.rs#L615)); delivery-only, no ordering/resolution change.
- No "commit pushed" line (Joe pushes).

---

## 4. Commit 2 — the D5 coverage enrichment (box-free, test-crate)

`xgen-mptest` only. Makes a green MP-C-14 actually prove leaf-authored cross-node convergence (audit §5).

### 4.1 Gate leaf sends on join

In the MP-C-14 template ([`mp_r3_topology.rs:39-71`](../xgen-mptest/tests/mp_r3_topology.rs#L39)): make
each leaf's `send` depend on its `join` having **landed** (sequence the send after the join's confirmed
reply / a settle gate) so the leaf post is genuine cross-node content — not a post racing its own join.
The `StarPlusMesh` generator path stays unchanged
([`sweep.rs:456-471`](../xgen-mptest/src/sweep.rs#L456)).

**Load-bearing constraint (D5, D-065):** **keep `p0` authored BEFORE the leaves join.** The enrichment
ADDS coverage; it must NOT relax the timing into a no-race scenario (that would hide MP-F14, not fix it).
`a0`'s `p0` stays at its early `after_ms` ([`:50`](../xgen-mptest/tests/mp_r3_topology.rs#L50)); the
leaves still join after it.

### 4.2 The enriched 3-class oracle

The enriched scenario's convergence assertion must cover all three post-classes (so a GREEN proves
*every member sees every post regardless of join order*):
- **(a) pre-join creator post** — `p0` (the MP-F14 headline gap; stays);
- **(b) post-join leaf post** — each leaf posts after its join lands (the previously-untested half);
- **(c) post-join creator post** — `a0` posts again after all joins (the steady-state control).

`convergence_verdict` ([`oracle.rs:244`](../xgen-mptest/src/oracle.rs#L244)) already asserts all-node
transcript-set + membership equality — the enrichment ensures the transcript under test **contains** (a),
(b), (c) on every node.

### 4.3 Commit 2 DoD

- `cargo build -p xgen-mptest --all-targets` 0-error; clippy `--all-features` clean.
- The enriched `mp_r3_topology` smoke compiles + stays `#[ignore]` (box-gated); the fast `xgen-mptest`
  suite 0-failed (the enrichment is in the `#[ignore]` smoke + any unit-level template assertions).
- The generator unit test `star_plus_mesh_emits_leaf_cross_links`
  ([`sweep.rs:898`](../xgen-mptest/src/sweep.rs#L898)) stays GREEN (topology unchanged).
- No "commit pushed" line.

---

## 5. Box-gated witness → R3 rerun

The witness is the **box-gated rerun**, not a code commit (the C5 split: spine in-process,
witness at the RUN).

1. Re-bench is **not** re-required for a fix-phase rerun (the J-351 ceiling ~1384 stands; this is a
   correctness rerun, not a climb).
2. Run the **enriched** `mp_r3_topology` (`--features harness-control`):
   ```text
   cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
   cargo test -p xgen-mptest --test mp_r3_topology -- --ignored --nocapture
   ```
   **MP-C-14 flips RED → GREEN** — the 4-node star+mesh converges, and the enriched oracle confirms (a)
   `p0` (pre-join creator), (b) leaf posts (post-join), (c) the post-join creator post all land on every
   node. Run several times (the gap was ~60% intermittent — the green must be **stable**, the conservative-
   settle discriminator no longer surfacing a stuck `p0`).
3. **R3 rerun to criterion:** re-run the affected/adjacent box-gated smokes to confirm no regression
   (the fix is a federation-delivery trigger — re-confirm the dep-witness rows that ride the same
   machinery: MP-A-01(ii) `mp_r2_catchup::mp_a_01_ii_*` stays GREEN; MP-A-08 relationship-heal
   `mp_r3_partition` stays GREEN — both ride the establish/`federation_nodes` path the fix touches).
   Result: **all-green-except-MP-C-16** (MP-C-16 stays red-with-reason, MP-F13/J-278, the M10+ carve-out).

**If the witness does NOT flip green** (or a dep-witness regresses): the §2 fork lock or §3 fix site is
the pin-by-observation item — re-trace (the §2 diagnostic) the failing rerun before any further change.
Newly-surfaced bugs face-and-route; the gate stays frozen at {MP-F14}.

---

## 6. MP-R3 close + the consolidated R1+R2+R3 ledger (Chat bridge)

When §5 lands **all-green-except-MP-C-16**, MP-F14 is terminal (GREEN-on-rerun) → the bounded gate
{MP-F14} is terminal → **MP-R3 closes**. R3 is the last round, so **the R3 close is the milestone
close** (HANDOFF §3). Chat's doc-bridge (a separate commit; Clair's code + this runbook → COMPLETED
commit FIRST) produces:
- the **consolidated R1+R2+R3 ledger** (the standing HANDOFF §3 deliverable — every `MP-C-##` / `MP-A-##`
  row across all three rounds with its FINAL status + the full `MP-F#` findings table);
- the **§3.1 breadcrumb sweep:** MP-F2-followon (7 unmapped wire-codes) → re-home to **M10** explicitly;
  the D-091 mis-file tidy verified **done-or-routed**;
- the canonical-record flips: `MP_findings` (MP-F14 RESOLVED/GREEN-on-rerun), matrix §6 (MP-C-14 GREEN;
  MP-R3 CLOSED), ROADMAP (Multiparty milestone ✅), CLAUDE.md PLAY, JOURNAL J-NNN, the arc docs
  (AUDIT/DESIGN/this IMPL) → COMPLETED.

---

## 7. Per-commit discipline (standing)

- **DoD grounded to live file:line (D-078)** — each commit re-verifies its anchors against `main` at
  pickup (lines drift; C5 shifted them once already).
- **Every test named** in the commit (this runbook names them; the §2 fork finalizes the spine names).
- **Build 0-error; clippy `--all-features` clean** (default + harness-control), every commit.
- **Spine-first** — the in-process RED-on-revert spine (§3.2) precedes the box-gated witness (§5).
- **No "commit pushed" line** — Joe pushes. Clair's code + runbook commit FIRST; Chat's doc-bridge is a
  separate commit.
- **No self-close.** The box-gated witness + R3 rerun gate the close; the ledger is Chat's.

---

## 8. Scope guard / not-in-scope

- **Frozen gate = {MP-F14}.** Newly-surfaced bugs at the §2 trace or the §5 rerun **face-and-route** to
  their homes (sibling MP-F12) — they do **NOT** extend the gate (the R2/J-344 discipline).
- **NOT this arc:** MP-C-16/MP-F13 (home_node WS-URL vs pubkey-id; M10+ carve-out). MP-A-08's
  transport-level reconnect-deadlock half (R3-D2, routed). The residents-multiplexer (R3-D1, routed).
  DM federation (untouched — D6).
- **Falsification (D7):** if §2 shows a delivery bug, the mechanism re-shapes but the gate disposition
  (fix-in-round, terminal = GREEN-on-rerun) is unchanged.

---

## 9. Entry point (Rule 0)

CLAUDE.md PLAY (J-351 RUN-#1/fix-phase head) → JOURNAL J-351 → `tasks/HANDOFF_MP_R3.md` →
`tasks/MP_F14_PREJOIN_BACKFILL_AUDIT.md` → `tasks/MP_F14_PREJOIN_BACKFILL_DESIGN.md` (MP-F14-D1..D7) →
this runbook → `tasks/MP_findings.md` (MP-F14) → `docs/tests/MULTIPARTY_TEST_MATRIX.md` §6.

**Next step:** Joe-lock this runbook → **§2 exec-step-1 re-trace** (box-gated) → fork lock (MP-F14-D2) →
**§3 fix + spine** (box-free, RED-on-revert) → **§4 enrichment** → **§5 box-gated witness + R3 rerun** to
all-green-except-MP-C-16 → **§6 MP-R3 close + the consolidated R1+R2+R3 ledger** (= the milestone close).
No code until Joe-locks this runbook.
