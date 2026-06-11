# MP-F9 — late-federation catch-up does NOT backfill existing Space history — D-071 PHASE-0 AUDIT

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Discipline note

Phase-0 grounding only — **no code, no design lock, no fix**. This is the first item of the MP-R2
fix-phase BOUNDED gate (J-344). The job, per the routing brief: **pin the kind — protocol or
harness** — for the C3 late-federation catch-up RED, by grounding the federation-initiate →
sync/backfill path against live `main`. Surface-and-route honesty (D-065/D-084): if grounding shows
a production-crate change is needed, **that is the verdict**, not a thing to quietly build or work
around. Standing arc shape: Phase-0 → design → Joe-lock → runbook → implement → close. Output here is
this audit for Joe-lock. **Boundary held:** this is MP-F9/F10 only — no drift into F8/F7; anything
newly surfaced routes to its own home and does **not** re-open the four-item gate.

Carried in scope per the brief: **MP-F10** (the director phase-ordering deadlock, pure harness) — §4
records exactly how its fix pairs with whatever MP-F9 needs.

---

## 1. The finding (the symptom, as surfaced at the RUN)

The C3 late-federation / catch-up machinery (built box-free at J-342, MP-R2-D5) ran for the first
time at the box-gated RUN (J-344) and came back **RED, confirmed deterministic** (isolated re-run
×2, Rule 2). Two C3 smokes, two distinct symptoms:

- **Smoke 1 — `late_federation_catch_up_converges`** (`xgen-mptest/tests/mp_r2_catchup.rs:46-83`):
  B federates **late** (the A→B link's `after = "history_ready"` gate is alice's post export, *not* a
  clock key). The `add-peer` + `initiate` fire — **no deadlock** — but B's transcript comes back
  **empty (zero events)** for the Space. This is **MP-F9** (the root finding).
- **Smoke 2 — `mp_a_01_ii_aged_invite_replay`** (`mp_r2_catchup.rs:170-211`): a 45 s timeout
  (`"waiting for cross-actor key {clock_advanced}"`) — the link is gated on a **clock-phase key**.
  This is **MP-F10** (a pure-harness director deadlock; §4).

This is the **first end-to-end exercise of federate-AFTER-history**: the normal G-6 bootstrap
federates **early** (before any Space exists), so its events propagate live; nothing before this RUN
ever streamed a *pre-existing* Space onto a freshly-federated peer. That is exactly the "content +
reachability validated only at first RUN" risk the box-gated RUN exists to catch (J-344 box-free
learning).

**Decisive datum for the verdict:** the symptom is **zero events**, *not* "only `state.space_create`."
That single fact is the discriminator §2.4 turns on.

---

## 2. Grounding — the federation-initiate → backfill path (live `main`, file:line)

### 2.1 The harness drives it correctly (NOT the F9 fault locus)

The late-fed director (`run_director`, `xgen-mptest/src/runner.rs:437-518`) establishes the late link
faithfully — the `after`-gated branch (`:450-464`): wait for the gate key, then `node_add_peer`
**both directions naming the now-built Space**, then `node_initiate` from `from`:

```
node_add_peer(from=A.ctl, to_peer=(B_id,B_url), spaces=[space])   // runner.rs:455
node_add_peer(from=B.ctl, to_peer=(A_id,A_url), spaces=[space])   // runner.rs:458
node_initiate(from=A.ctl, peer=B_id)                              // runner.rs:461
```

This is byte-identical in shape to the early-bootstrap tail (the `None` branch, `:468-475`) — the only
difference is *timing* (it fires after `history_ready` instead of before any Space). `node_add_peer` /
`node_initiate` (`runner.rs:567-585`) are thin aicontrol drives (`federation add-peer` / `federation
initiate`). The director runs **concurrently** with the actor drive, then the harness `settle()`s up to
15 s for event-count quiescence (`runner.rs:329-365`, `:523-525`). **The harness establishment for
Smoke 1 is correct** — the gap is downstream of the verb-ack.

### 2.2 The sender (initiator A) backfill path EXISTS and is structurally complete

`federation initiate` → `federation_initiate` (`xgen-node/src/admin_ops.rs:1728`) reads the
registry relationship's `shared_spaces` (the late `add-peer` put the Space there, `:1758`) and spawns
`reconnect::attempt_reconnect` with `shared_spaces` (`:1807`). In `attempt_reconnect`
(`xgen-node/src/reconnect.rs:404-510`): build per-Space tips from `shared_spaces`
(`rt.dag_tips`, `:404-416`) → `run_initiating` (bilateral hello/tips exchange,
`xgen-core/src/federation/handshake.rs:133-213`, returns the peer's tips) → then
`run_federation_session_post_handshake(..., shared_spaces, session.peer_tips, ...)` (`:489-510` —
**A's own `shared_spaces`, which includes the late Space**, is the iteration domain).

`run_federation_session_post_handshake` (`xgen-node/src/app.rs:2061`): the **initiator** drains the
receiver's (empty) catch-up to `SyncComplete` (`:2105-2151`), then **streams its own delta**
(`:2157-2178`) via `stream_federation_delta`.

`stream_federation_delta` (`xgen-node/src/federation_session.rs:84-216`) — **the backfill is here**:
- For the late Space, `peer_tips` (B's) has no entry → `peer_absent = true` (`:108-109`).
- A's local tips are non-empty → `we_have_events = true` (`:114-118`).
- `compute_federation_delta_for_space(runtime, space, None)` → **the full topologically-sorted Space
  history** (`xgen-node/src/fanout.rs:605-639`; the `tip_str.is_empty()` branch returns `sorted`
  whole, `:628-631`).
- The **a-i symmetry rule** (`:137`): `peer_absent && we_have_events` → A builds + ingests + persists
  a `state.federation_add` and **pushes it onto the delta** (`:141-187`).
- A then streams the delta in order: `[space_create, room_create, message, federation_add]`
  (`:195-201`) + `SyncComplete`.

**Verdict on capability:** the protocol **HAS** a late-federation history-backfill path, and it
streams correctly. This is **NOT** "no late-federation history-backfill exists at all." The empty
transcript is a *delivery* failure of an existing mechanism, not a missing capability — which is why
§2.4's discriminator matters.

### 2.3 The receiver (B) chain — where the stream lands, gate by gate

B receives the four streamed events via its F-2 receive loop (`process_inbound`, origin
`ReceivedViaFederation`). Two receive-side gates apply per event:

1. **F-3 federation-relationship gate** (`xgen-core/src/node/runtime.rs:999-1099`). `skip_f3`
   (`:1027-1032`) covers `StateFederationAdd | StateSpaceCreate | StateDmSpaceCreate` — so a
   federation-streamed `state.space_create` **passes F-3** (it brings the Space into existence;
   federation_nodes can't exist yet, `:1016-1026`). `state.room_create` and `message.text` are **not**
   in the skip set → on a Space whose `federation_nodes` doesn't yet contain A they go **HeldPending**
   on the `(peer, space)` trigger (`:1041-1096`), to be drained when the `state.federation_add`
   arrives (the drain hook, `:1591`). So F-3 alone would let the chain complete: create applies →
   room/message held → federation_add applies + populates `federation_nodes` + drains them.

2. **Step 11 sender-registration gate** — **the decisive one** (`xgen-core/src/message/exchange.rs:601-634`).
   *Before* F-3 can be satisfied, every event runs `validate_event`, and step 11 holds any event whose
   signer is not a registered Identity on B:
   ```rust
   let node_authored = matches!(event.event_type,
       MembershipNodeEject | MembershipNodeUnban | StateSpaceMigrate);   // :622-627
   if !fed_add_via_federation && !node_authored && !id_registry.contains(sender) {  // :629
       return ValidationOutcome::HeldPending { missing_identity: Some(sender.clone()) };  // :630-633
   }
   ```
   **`StateSpaceCreate` is NOT in `node_authored`** and is not `fed_add_via_federation`. So a
   federation-streamed `state.space_create` signed by alice, with alice unknown to B, returns
   **`HeldPending(missing_identity=alice)`** — held *before* the F-3 skip ever helps. Same for
   `room_create` and `message` (all alice-signed). (`state.federation_add` is node-authored / B3-skip,
   so it could apply — but it never gets the chance to do anything useful, because nothing it would
   drain is held on F-3; everything is held on F-10 sender-registration instead.)

### 2.4 The decisive localization — identity replication never reaches a late peer

For step 11 to hold *everything* (including the F-3-skipping `space_create`), alice's Identity must be
unknown to B. Grounding the identity-replication path confirms it:

- `push_identity_to_peers` (`xgen-node/src/app.rs:3118-3192`) is the **only** identity-replication
  driver, and it is called from **exactly one site**: the registration handler, fire-once at register
  time (`:2853-2857`).
- It snapshots `rt.peer_urls` **at registration** (`:3126-3129`) and **returns early if empty**
  (`:3131-3133`). It pushes only to peers that are federated **at the moment of registration**.
- The federation handshake/session (`run_federation_session_post_handshake` /
  `stream_federation_delta`) streams **Space-DAG events only** — it does **not** replicate the
  registry's Identities. There is **no "replicate existing identities to a newly-federated peer"
  path** anywhere.

In the late-fed scenario alice registers on A (`a1`) **before** B federates, so A's `peer_urls` does
not yet contain B → alice's Identity is pushed to nobody (or only pre-existing peers), and is **never
re-pushed** when B federates later. When A backfills the Space's history, every event is alice-signed,
alice is unknown to B → **all four events HeldPending on step-11 sender-registration** → none stored,
none fanned out, none transcript-visible → **B's transcript = zero events.** (The held events time out
on the F-10 30 s / the 4006 `identity_record_timeout` path, never recovering — no identity arrives.)

This chain **uniquely** explains "zero events." A pure-harness timing/lifecycle cause (settle too
short, collector miss) would not selectively hold the F-3-skipping `space_create`; an F-3-only
receive-path cause would still let `space_create` through (non-empty transcript). Only "the signer
Identity never replicated" holds the create too — matching the observed zero.

---

## 3. KIND VERDICT — PROTOCOL (grounded), bounded

**MP-F9 is a PROTOCOL gap, in the receive/catch-up path — not a harness fault, and not "no backfill
exists."** Precisely:

> Late-federation Space-history **event** backfill exists and streams correctly (§2.2), but the
> federation handshake/session does **not** replicate the **Identities** that signed that historical
> events, and identity replication is a fire-once-at-registration push to then-current peers only
> (§2.4). A peer that federates after history+identities exist receives the backfilled events but
> holds them all on step-11 unknown-signer (F-10) HeldPending → a zero-event catch-up.

The missing capability is **"a federating/catching-up peer receives the Identity records of the
signers of the Space history it is catching up"** (equivalently: the federation session streams the
relevant identity records alongside the Space-DAG delta, or `push_identity_to_peers` is re-driven on
federation-establish for the shared Spaces' members). Production crate (`xgen-node`, and the
receive-path is `xgen-core`). The **event half exists; the identity half is missing.**

**Honest boundary (D-065):** this verdict is grounded by code-trace + the unique zero-event
explanation; I have **not** run a live instrumented re-run (that is RUN/box-gated, a design-phase
step). A design-phase confirm re-run should look for, on B: `event = "f3_reject"` /
`HeldPending(missing_identity=…)` traces + 4006 `identity_record_timeout` on each backfilled event,
and `push_identity_to_peers` taking the empty-`peer_urls` early-return at alice's registration. If
the re-run instead shows the create *applied* on B (non-empty transcript) the localization would shift
— but the zero-event symptom already rules that path out.

**Why this is not "rescope the test to federate early" (D-065).** Forcing early federation would
defeat the entire purpose of the C3 late-federation machinery (MP-R2-D5) — a node joining a network
that *already has* history+identities is the realistic, intended case. The scenario is faithful; the
gap it exposes is real protocol. Surface, don't paper over.

---

## 4. MP-F10 relation — how F9 and F10 pair on the C3 machinery

**MP-F10 is a separate, pure-harness deadlock** (grounded `xgen-mptest/src/runner.rs:437-496`):
`run_director` runs phases **sequentially** federation → clock → migration. A late link with
`after = Some(clock_advanced)` blocks the **federation phase** on `wait_for(clock_advanced)` (`:452`),
but `clock_advanced` is published only in the **later clock phase** (`:494`) → the federation phase
waits for a key the clock phase (which runs after it) never reaches to publish → deadlock (Smoke 2's
45 s timeout). Fix = a harness reorder/interleave in `xgen-mptest` (e.g. interleave the
federation/clock phases, or schedule a federation link *after* a clock step). **Test-crate only.**

**How they relate:**
- **Smoke 1** (`late_federation_catch_up_converges`) is gated on `history_ready` (a *post* export,
  published by the concurrent actor drive — not a clock key). It does **not** hit the F10 deadlock.
  Its RED is **MP-F9 alone** (zero events — the protocol identity-backfill gap).
- **Smoke 2** (`mp_a_01_ii_aged_invite_replay`) is gated on `clock_advanced` (a clock-phase key). It
  hits the **MP-F10** deadlock *first* (never even reaches federation). Even after F10 is fixed, it
  **still needs MP-F9**: bob registers on B and the late peer is C, so bob's Identity + his historical
  invited-join must backfill onto C — exactly the §3 identity-catch-up the protocol lacks.

So: **MP-F9 (protocol) is load-bearing for both C3 rows; MP-F10 (harness) additionally gates the
clock-aged row.** They are independent fixes — F9 in `xgen-node`/`xgen-core`, F10 in `xgen-mptest` —
but both land in the same C3 machinery, so the design/runbook should carry them together. F10's
reorder does not depend on F9; F9's identity-backfill does not depend on F10. Sequence-wise the F10
harness reorder is the cheaper, self-contained half and naturally rides with the F9 design phase.

---

## 5. Recommendation (for design-lock — NOT locked here)

1. **MP-F9 is PROTOCOL — route it as a late-federation identity-catch-up arc** (its own design phase).
   It is **not** "build backfill from scratch" (the event-backfill exists) and **not** harness.

2. **Sizing is the design-phase's first question (bounded vs deep), and it decides the gate terminal
   state:**
   - **Plausibly bounded.** The machinery to move an Identity record between nodes already exists —
     `IdentityReplicateMessage::Replicate` + the receiver hook `handle_identity_replicate_msg`
     (`xgen-node/src/app.rs:2913-3021`) + the `drain_pending_by_identity` arrival drain that already
     releases F-10-held events when an Identity lands. The candidate fix reuses all of it: on
     federation-session-establish (or as part of the delta stream), the sender also replicates the
     Identity records of the members/signers of the shared Spaces to the new peer — so the held
     backfill events drain. If the design phase confirms this composes cleanly, MP-F9 can **GREEN on
     rerun** inside the fix-phase, before the R2 rerun.
   - **If it proves deep** (e.g. it re-opens "which identities does a peer get / privacy scoping /
     who-knows-whom on federation" — the identity-discovery territory MP-F1b's D5 already routed),
     then **Joe-routed → R3-as-named-dependency** is the *allowed terminal state* (J-344): MP-F9 is
     **load-bearing for R3 regardless** — the R3 partition+reconnect storm (MP-A-08) leans on a peer
     catching up after a gap, which has the **identical shape** (a reconnecting peer must catch up
     both the Space events *and* any identities registered during the gap it missed). So carrying it
     into R3 as a named dependency is coherent, not a punt.

3. **MP-F10 — route the harness reorder** (`xgen-mptest`), paired with the MP-F9 design phase. Cheap,
   self-contained, test-crate-only. Pins to **GREEN on rerun** for Smoke 2 *once MP-F9 also lands*
   (Smoke 2 needs both).

4. **Design-phase opener:** a single instrumented late-fed re-run (Smoke 1, tracing on) to **confirm**
   the §2.4 localization (F-10 HeldPending + 4006 on each backfilled event; empty-`peer_urls`
   early-return at registration) before locking the fix shape. This is the honest-boundary close on
   §3 — code-trace says protocol-identity-gap; the re-run nails it.

---

## 6. Scope / route / what this audit does NOT do

- **Does NOT** lock a design, write code, or touch the gate's scope (frozen at four — J-344).
- **Does NOT** re-open F8/F7 or fold anything new into the gate. Nothing new surfaced that needs a
  separate route — the §2.4 chain is entirely explained by the named MP-F9 gap.
- **Route:** MP-F9 → its own design phase (`tasks/MP_F9_*_DESIGN.md`, next, after Joe-lock of this
  verdict) → runbook → implement → close, OR Joe-routed→R3-as-named-dependency if the design sizes it
  deep. MP-F10 → harness reorder in `xgen-mptest`, paired.
- **Canonical-record updates** (`MP_findings.md` MP-F9 kind-pin, JOURNAL, ROADMAP, matrix) are the
  Chat seat's doc-bridge, separate from this arc-doc, assembled at Joe-lock. This audit is the impl
  seat's deliverable.

**Code anchors (single list, for the design phase):**
- Harness late-fed establishment: `xgen-mptest/src/runner.rs:437-518` (`:450-464` late branch);
  drives `:567-585`; concurrency+settle `:329-365`; Smoke 1 `xgen-mptest/tests/mp_r2_catchup.rs:46-83`.
- Sender backfill (exists, correct): `xgen-node/src/admin_ops.rs:1728-1817`;
  `xgen-node/src/reconnect.rs:404-510`; `xgen-core/src/federation/handshake.rs:133-213`;
  `xgen-node/src/app.rs:2061-2178`; `xgen-node/src/federation_session.rs:84-216` (a-i rule `:137`);
  `xgen-node/src/fanout.rs:605-639` (full history on `None`, `:628-631`).
- Receiver gates: F-3 `xgen-core/src/node/runtime.rs:999-1099` (skip set `:1027-1032`, drain `:1591`);
  **step-11 sender-registration HeldPending `xgen-core/src/message/exchange.rs:601-634`** (`node_authored`
  set `:622-627` — `StateSpaceCreate` NOT exempt; hold `:629-633`).
- **The gap — identity replication:** `xgen-node/src/app.rs:3118-3192` (`push_identity_to_peers`,
  registration-only, empty-`peer_urls` early-return `:3131-3133`); sole caller `:2853-2857`; the
  receive hook the fix would reuse `:2913-3021` (`handle_identity_replicate_msg`).
- MP-F10 (harness deadlock): `xgen-mptest/src/runner.rs:437-496` (fed-phase wait `:452`, clock publish
  `:494`); Smoke 2 `xgen-mptest/tests/mp_r2_catchup.rs:170-211`.

---

*Per D-065 (surface, don't work around) + D-069 (arc-local) + D-071 (audit precedes design) + D-084
(route, don't patch in-tranche) + MP-R1-D8 (honest boundary) + the J-344 BOUNDED-gate criterion.*
