# MP-F9 — late-federation identity catch-up — DESIGN (carries MP-F10)

> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Discipline note

Design phase **Joe-LOCKED at close** (terminal-state A). Builds on the Joe-LOCKED Phase-0 verdict
(`tasks/MP_F9_LATE_FEDERATION_BACKFILL_AUDIT.md`, PROTOCOL/bounded). **Terminal-state LOCKED = (A)
fix-in-fix-phase** — MP-F9 GREENs on the R2 rerun (over (B) Joe-routed → R3-as-named-dependency). The
call was made by the grounding, not by judgment: the fix composes via existing machinery
(`handle_identity_replicate_msg` → `drain_pending_by_identity` auto-drain, §3) and the disclosure is
the **registration-precedent**, NOT MP-F1b's D5 (§4); the only gap is the **trigger**. **F9-D1 / F9-D2
/ F9-D3 + F10-D1 are LOCKED** (arc-local, D-069); the within-A micro-decisions (§3) are runbook detail.
Carries **MP-F10** (§7, pure-harness, independent fix, same C3 arc).

**Two holds through implementation (Joe, D-065):**
1. **F9-D3 stays delta-signers, not members-only** — do **not** let it drift under implementation
   pressure. R3-correctness: a reconnecting peer must validate historical events whose signers may
   have **since left** the Space (§6). Members-only would leave those events permanently F-10-held.
2. **Terminal-A is conditional, not a muzzle** — it holds **iff** the confirming traced re-run nails
   the verdict (the `state.space_create` HeldPending discriminator, §2) **and** implementation surfaces
   no real disclosure-scoping fork the registration-precedent doesn't cover. **If the re-run
   contradicts the trace, or a genuine fork appears → surface it and revert to route-to-R3; do NOT
   shrink the fix to preserve the lock** (D-065).

**Scope guard (J-344):** this is the gate's F9+F10 item — no drift to F8/F7. Anything newly surfaced
is faced-and-routed to its own home and does **not** re-open the four-item gate.

---

## 1. The locked verdict (recap)

Late-federation Space-history **event** backfill exists and streams correctly (sender-side A streams
the full Space onto a late peer via `stream_federation_delta`'s a-i rule +
`compute_federation_delta_for_space(None)`). B holds **all** of it → zero events, because every
backfilled event is alice-signed, alice is unknown to B, and step-11 sender-registration HeldPending
fires (`exchange.rs:629`; `StateSpaceCreate` not in the `node_authored` exempt set). Root: identity
replication (`push_identity_to_peers`) fires **only at registration to then-current peers**
(app.rs:2856/3118); the federation session streams Space-DAG events only, never the registry. **The
event half exists; the identity half is missing.** Full chain + file:line in the audit §2.

---

## 2. First move — the confirming traced re-run (the empirical nail)

The verdict is code-trace + unique-symptom. Before locking a fix shape, **one instrumented re-run** of
Smoke 1 (`late_federation_catch_up_converges`) confirms it empirically. **Box-gated — coordinate with
Joe to run** (a harness-control node build + the C3 catchup smoke, `--test-threads=1`,
background-spawned per the RUN discipline; a spawn/connect timeout is a flake → re-run isolated, Rule 2).

**What to instrument / confirm (the pass/fail signature of the verdict):**
1. On **B**, for each backfilled event (`space_create`, `room_create`, `message`): a
   `ValidationOutcome::HeldPending { missing_identity: Some(alice) }` — i.e. step-11
   (`exchange.rs:629`) fires, **including for `state.space_create`** (the discriminator: even the
   F-3-skipping create holds). Surfaceable via the existing G2 trace + a temporary
   `tracing::debug!` at the step-11 hold site if the held path isn't already traced.
2. On **B**, eventually the 4006 `identity_record_timeout` sweep on those held events (no identity
   ever arrives) — confirming the hold is terminal, not a transient.
3. On **A**, `push_identity_to_peers` taking the **empty-`peer_urls` early-return** (app.rs:3131-3133)
   at alice's registration (`a1`), because B is not yet federated — confirming the trigger gap.
4. The **negative control:** B's `id_registry` never contains alice (no Replicate ever sent to B for
   her) — confirming the federation session does not replicate the registry.

**Decision rule:** if (1)-(4) hold, the verdict is empirically nailed → proceed to the §3 fix shape.
If instead the create *applies* on B (non-empty transcript, partial delivery), the localization shifts
and the design re-opens — but the zero-event symptom already rules that branch out; this re-run is the
honest-boundary close, not an expected surprise.

> **Note:** the re-run is the *opener* of the design's empirical leg; it does **not** gate authoring
> this doc. The fix-vs-route grounding (§3-§5) stands on the code-trace and is presented now for
> Joe's read; the re-run confirms before any code.

---

## 3. The candidate fix — composes cleanly (grounded)

The fix is a **trigger, not new machinery.** Every receiving-side piece already exists and **auto-drains**:

- **Receiver hook (exists):** `handle_identity_replicate_msg` (app.rs:2913) upserts the
  IdentityRecord, then on success fires **`drain_pending_by_identity(identity_id)`** (app.rs:2975) —
  which releases every F-10-held event whose `missing_identity == identity_id`
  (`runtime.rs:1717` → `buf.resolve_identity` → `try_release`), re-dispatches + persists them, **inside
  the same lock** as the upsert (no missed-drain race). So: *land the signer's record on B → the held
  backfill events drain automatically.* No new receive handler, no new drain logic, no new wire message.
- **Sender send-path (exists):** `push_identity_to_peers` (app.rs:3118) already does
  connect → authenticate → `IdentityReplicateMessage::Replicate` → await ack → `add_replica`, per
  record, to every peer in `peer_urls`. It serializes the **whole** `IdentityRecord` (app.rs:3135).
- **The peer URL is already recorded on establish (exists):** at federation-establish, both sides
  `record_peer_url(peer)` — with the live comment **"so we can push identity replicas to it later"**
  (receiver app.rs:1976-1983; initiator the symmetric site in `reconnect::attempt_reconnect`). The
  data the send-path needs is already populated by the time the session is ACTIVE.

**The only gap:** nothing pushes the **backlog** of already-registered identities to a peer that
federates *after* they registered. `push_identity_to_peers` is called for **one new identity at
registration time** (app.rs:2856), never for the existing set on a new establish.

### Framed F9-D# (NOT locked — for Joe-lock at design-close)

- **F9-D1 — fix = a backlog-push trigger on federation-establish.** Reuse the existing
  Replicate send-path + the auto-draining receiver hook. No new machinery; no new wire shape.
- **F9-D2 — trigger point: on establish, both sides, symmetric.** Initiator: `attempt_reconnect`
  post-ACTIVE (after `record_peer_url`, around the post-handshake spawn). Receiver:
  `handle_federation_incoming` post-seal (app.rs:1980-1983, after `record_peer_url`, before the
  post-handshake driver). Each side replicates **its own** known signer records to the peer (each side
  knows its own members' records); symmetric so both directions catch up.
- **F9-D3 — signer set = the distinct senders of the shared Spaces' backfilled history**, NOT just
  current `SpaceState.members`. *Why the larger set:* step-11 checks the **signer** of each historical
  event; an event from a since-departed member still needs that member's record, so current-members-only
  leaves those events permanently F-10-held (correct for Smoke 1 where alice never leaves, **wrong in
  general** — and wrong for R3, §6). The set is derivable from the same delta
  `compute_federation_delta_for_space` already computes (its distinct `ev.sender`), so it's tight,
  bounded, and event-coupled.
- **F9-D4 — delivery path: reuse `push_identity_to_peers`' per-record send to the now-recorded peer**
  (a variant that pushes *many* records to *one* new peer, vs the existing one-record-to-all-peers).
  **Alternative (noted, not led):** stream the records over the *session* connection alongside the
  delta — fewer connections, but it needs a new `Inbound::IdentityReplicate` arm in the initiator
  drain loop (app.rs:2146 `Ok(_) => {}`) **and** the F-2 loop, i.e. more receive-path surface. Lead
  with reuse (F9-D4) for correctness/minimalism; the over-session form is a **perf refinement**, not a
  correctness requirement (the drain-on-arrival makes ordering forgiving — records can land any time
  and the held events drain).
- **F9-D5 — disclosure is already-settled (the terminal-state-A grounding).** Replicating these
  records to the federated peer is **not a new disclosure decision**: `push_identity_to_peers` already
  sends the **whole** `IdentityRecord` (incl. `home_node`, the only field beyond what the events
  themselves carry — `identity_id` is the pubkey URI, already in every signed event) to `peer_urls` at
  registration. So a peer that federated *early* already receives the full records of subsequently-
  registered members. The late-fed fix replicates the **same records, same category, different
  trigger** to a peer that federated *late*. It is **not** MP-F1b's D5 ("production identity→home-node
  **discovery**" — resolving a **stranger** you've never interacted with): here the peer is catching up
  the records of signers whose **events it is already receiving** (it federates the Space). Already-
  entitled, bounded set — not stranger-discovery.
- **F9-D6 — R3 shape-match baked into the abstraction (§6).** The trigger is "on
  establish/catch-up, replicate the catch-up delta's signers"; MP-A-08 reconnect reuses it verbatim.

### Within-A micro-decisions (flagged for the runbook, none a D5 re-open)

1. **Departed signers** — F9-D3 includes them (their events are in the backfilled DAG anyway, so
   their `identity_id` is already disclosed; replicating their record is consistent). A "do we replicate
   departed members" micro-call, settled by F9-D3's correctness argument — within A.
2. **Volume / connection count** — F9-D4(a) opens one connection per record per establish. For a
   many-signer catch-up that's N connections; a batching refinement is possible (or F9-D4's
   over-session form). Perf shape, not correctness — within A.
3. **Idempotency / re-establish** — `add_replica` + `handle_incoming_replicate`'s version-guard make
   re-replication idempotent; a re-establish re-pushing the set is harmless. Within A.

---

## 4. The decisive question — does the fix drag in D5? **Grounded: NO.**

The fix-vs-route call turns on whether the candidate fix composes cleanly via existing machinery or
drags in identity-discovery / privacy scoping (MP-F1b's D5 territory). **Grounding says it composes:**

| Dimension | Finding | Drag? |
|---|---|---|
| Receive machinery | `handle_identity_replicate_msg` + `drain_pending_by_identity` exist + auto-drain (app.rs:2975, runtime.rs:1717) | None — reused verbatim |
| Send machinery | `push_identity_to_peers` send-path exists (app.rs:3118) | None — reused (records-to-one-peer variant) |
| Peer-URL availability | `record_peer_url` on establish, "push replicas later" (app.rs:1976-1983) | None — already populated |
| Signer set | distinct senders of the backfilled delta (already computed) | None — derivable, bounded |
| **Disclosure** | full record incl. `home_node` **already** replicated to federated peers at registration (app.rs:3135) | **None — same category, different trigger; NOT stranger-discovery** |

The one dimension that *could* have been D5 — disclosure — is **already settled** by the registration-
time precedent. So the fix is the bounded reuse of existing, fully-working machinery + a trigger. **It
does not re-open "which identities, to which peers, under what disclosure rules."**

---

## 5. Fix-vs-route — **terminal-state A (fix-in-fix-phase) — Joe-LOCKED at design-close**

**MP-F9 GREENs on the R2 rerun (terminal-state A), not Joe-routed→R3.** Grounds: the machinery exists
and auto-drains (§3); the disclosure is already-settled (§4 F9-D5); the only missing piece is a
backlog-push trigger on establish + the signer-set derivation — bounded reuse, no D5 drag. MP-F9 is
implementable inside the fix-phase as its own arc (Phase-0 ✅ → design ✅ → runbook → implement), then
the R2 rerun re-runs the C3 smokes to green-to-criterion.

**Honest boundary (D-065 — the route-to-R3 trigger, stated so it isn't shrunk to fit):** terminal-state
A holds **iff** the §2 confirming re-run nails the verdict **and** design-lock surfaces no disclosure
subtlety the §4 precedent doesn't actually cover. The one place to re-test that precedent at lock:
whether `home_node`-to-a-late-peer is *genuinely* covered by the registration-time push (it is, by
trace — `push_identity_to_peers` sends the whole record to `peer_urls` regardless of *when* the peer
federated relative to the identity) or whether a late peer is materially different in a way the trace
missed. If the re-run or lock surfaces that the fix is bigger than the bounded reuse (e.g. it forces a
disclosure-scoping decision, or the signer-set derivation drags in cross-node history the sender
doesn't hold), **that is the route-to-R3 verdict** — coherent, since R3/MP-A-08 needs identity-catch-up
anyway (§6). Joe locks the terminal state at design-close on this grounding.

---

## 6. R3 shape-match (design input — get the abstraction right)

MP-A-08 (partition + reconnect, R3) is the **identical** problem: a reconnecting peer must catch up
**both** the events **and** the identities registered during the gap it missed. The F9-D1/D2/D3 shape
— "on establish/catch-up, replicate the signers of the catch-up delta to the peer" — is **exactly**
what a reconnecting peer needs. Designing F9 with this in mind argues for **F9-D3 (delta-signers, not
current-members)** and **F9-D2 (the trigger generalizes to any session establish, not just
first-federation)**, even though Smoke 1 would pass with the narrower current-members set. **The
immediate fix is small; the abstraction is R3-load-bearing — get it right once.** The reconnect path
(`attempt_reconnect`) is *already* the same code the late-fed initiate uses (J-085: the reconnect
scheduler is the production caller), so the establish-trigger lands on the path R3 reconnect reuses
with no extra abstraction work.

---

## 7. MP-F10 — the harness reorder (carried, independent fix)

**Pure harness, test-crate only.** `run_director` runs phases **sequentially** federation → clock →
migration (`xgen-mptest/src/runner.rs:437-518`). A federation link with `after = Some(clock_advanced)`
blocks the **federation phase** on `wait_for(clock_advanced)` (`:452`); `clock_advanced` is published
only in the **later clock phase** (`:494`) → the federation phase waits for a key the clock phase
(which runs after it) never reaches to publish → deadlock (Smoke 2's 45 s timeout). The fixed phase
order cannot satisfy a cross-phase publish→wait edge.

### Framed F10-D# (NOT locked)

- **F10-D1 (lead) — a dependency-ordered single-owner director.** Replace the fixed federation→clock→
  migration sequence with an execution order that respects **publish→wait edges among director steps**:
  collect all steps (fed links, clock steps, migrations) with their `after` (wait-key) and `publishes`
  (publish-key); a step whose `after` is published by another director step is ordered **after** that
  step (topological order over the internal publish→wait edges); steps waiting on **external** keys
  (published by the concurrent actor drive, e.g. `history_ready`, `bob_join_ready`) just `wait_for`
  them as today. Sequential, single-owner of `&mut nodes` (no borrow refactor), no deadlock. Smoke 2:
  the clock step (waits external `bob_join_ready`, publishes `clock_advanced`) is ordered **before** the
  fed link (waits `clock_advanced`) → resolves. **Minimal, general, preserves the `&mut nodes`
  single-owner model.**
- **F10-D2 (alternative, heavier) — concurrent director tasks.** Spawn each step as a task that
  `wait_for(after)` → acts → `publish(es)`, mirroring how actors already run concurrently against the
  `Registry`. Removes ordering entirely, but breaks the `&mut [NodeHandle]` single-owner constraint
  (each step mutates a node ctl) → needs per-node ctl ownership (split the borrow). Bigger refactor;
  noted, not led.

**Recommend F10-D1** (dependency-ordered single-owner) — it is the smallest change that removes the
fixed-phase-order deadlock without touching the node-ownership model.

### Row coupling (carry into the runbook)
- **Smoke 1** (`late_federation_catch_up_converges`, gated on a *post* export) → blocked by **MP-F9
  alone** (no clock key → no F10 deadlock).
- **Smoke 2** (`mp_a_01_ii_aged_invite_replay`, gated on a *clock* key) → hits **MP-F10** first
  (deadlock); even after F10-D1, it **still needs MP-F9** (bob's identity + historical invited-join must
  catch up onto the late node C). So **MP-F9 gates both C3 rows; MP-F10 additionally gates the
  clock-aged row.** Two fixes, same C3 arc — travel together. F10-D1 is self-contained and rides the
  F9 implementation.

---

## 8. What design-close delivers + route

- **Terminal-state LOCKED (Joe, design-close) = A: fix-in-fix-phase**, on the §3/§4 grounding + the
  §0 conditional (the §2 confirming re-run is the runbook's exec step 1, gating the fix not the
  authoring).
- **Active path →** runbook (`tasks/MP_F9_LATE_FEDERATION_BACKFILL_IMPL.md`): exec step 1 = the
  confirming re-run; then F9-D1..D6 (the establish-trigger + signer-set + reuse send-path) + F10-D1
  (the dependency-ordered director), with the Smoke 1 / Smoke 2 row coupling as the green-on-rerun
  witnesses; then the R2 rerun re-runs the C3 rows.
- **Revert path (the §0 hold-2 conditional) →** if the re-run contradicts the trace or a genuine
  disclosure-scoping fork appears, MP-F9 reverts to **route-to-R3 as a named dependency** (MP-A-08
  host); MP-F10 still lands now (harness, R2-internal). Allowed terminal state (J-344). Surfaced, not
  shrunk-to-fit.
- **No new gate items.** Nothing in this design surfaced a finding outside MP-F9/F10. The within-A
  micro-decisions (§3) are runbook detail, not gate items.
- **Canonical-record** (`MP_findings.md` MP-F9 design-open + terminal-state ledger, JOURNAL, ROADMAP,
  matrix) is the Chat seat's doc-bridge, separate from this arc-doc, assembled at Joe-lock.

---

*Per D-065 (surface, don't shrink-to-fit) + D-069 (arc-local F9-D#/F10-D#) + D-071 (design follows the
locked Phase-0) + D-084 (route, don't patch in-tranche) + MP-R1-D8 (honest boundary) + the J-344
BOUNDED-gate criterion (terminal-state = GREEN-on-rerun OR Joe-routed-with-reason).*
