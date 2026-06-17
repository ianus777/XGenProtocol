# M10.4 — Production Identity→Home-Node Discovery (MP-F13) — Design
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Status & lineage

Fourth M10 sub-arc (MP-F13). Decisions **M10.4-D1..D5 Joe-LOCKED (J-370)** off the Phase-0 audit
(`tasks/M10_4_HOME_NODE_DISCOVERY_AUDIT.md` v1.0 ACTIVE, `c5677f3`, grounded vs `main`; 5 findings).
The brief's two scope calls (C1 narrow-first-with-escape, C2 aim-clean-green, J-369) hold — the audit
**confirmed** them (M10.4-A3: clean green reachable on Layer-1 alone, the escape does not fire).
Production arc (`xgen-common` wire + `xgen-client`; **no node-side consumer/gate change** — they
already want the pubkey). **Next-active = Clair: author the runbook.** No code until the runbook lands.

## 1. Scope

**In:** reconcile the one namespace violation — the Space-created `content["home_node"]` carries a WS
URL where every consumer wants the node's pubkey `NodeXgid`. Fix = the client learns + writes the
node_id (Shape B). Covers `state.space_create` **and** `state.dm_space_create`.

**Out (recorded boundaries):** Layer-2 discovery of an *unknown* identity (the F1B-D5 / DM-stranger
arc — separately routed, D4); the MP-C-16 box-gated end-to-end re-run (→ M10.5, D3); pre-existing
URL-homed Spaces (leave-as-legacy, D5); the full re-home-notify broadcast (J-278 CP-5 stays deferred —
this arc takes only the minimal node_id echo).

## 2. Locked decisions

**M10.4-D1 — Shape B: the client writes the node_id (a small additive `AuthOk` echo).** The canonical
value of `home_node` is the node's pubkey `NodeXgid` (audit §1.1); the only blocker is that the client
cannot learn the node's pubkey on the current wire (Challenge/AuthOk/RegisterOk carry none — audit §4).
Fix:
- Add an **additive optional** `node_id` field to `TransportMessage::AuthOk` (`xgen-common/src/wire/types.rs:64-68`);
  the node populates it with its own pubkey id (`node_id_uri = pubkey_uri(&signing_key)`, `app.rs:632`)
  on every `AuthOk`. Backward-compatible (old clients ignore it; Ch3 §3.0.3 additive-field rule).
- The client captures it in `client_authenticate` (`xgen-client/src/connection.rs:400,422`) and stashes
  it in `SessionState` beside the URL `home_node` (`xgen-client/src/session.rs`).
- `create_space` / `dm_space_create` write the **node_id** (not the URL) into `content["home_node"]`
  (`xgen-client/src/ops.rs:449-456`).

Chosen over **Shape A** (node-side read projection): Shape A avoids the wire change but pushes
pubkey↔URL projection into the pure signature/authority gates (the cutover gate `validate_event`
exchange.rs:717 is node-context-light — threading a resolver into it is the fragile surface). Shape B
resolves all six consumers + both migration sites with **zero projection**. **Carrier = `AuthOk`, not
`RegisterOk`:** the client receives `AuthOk` on *every* session-open, so a returning/already-registered
client that creates a Space still has the node_id (RegisterOk fires only once, at registration).

**M10.4-D2 — both stall sites reconcile to the pubkey namespace (hard proof obligation).** The fix must
clear **Site 1** (`migration initiate` homed-here precondition `MIG_6010`, `admin_ops.rs:2096`:
`st.home_node == rt.node_id`) **and Site 2** (the cutover authority gate `6009`, `exchange.rs:717`:
`event.sender == s.home_node`, + the defensive applier re-check `state.rs:1158`). Once `home_node`
stores the source's pubkey, Site 1 = `pubkey == rt.node_id` ✓ and Site 2 = `source-pubkey ==
home_node(source-pubkey)` ✓, then the applier flips `home_node` to the destination pubkey
(`state.rs:1161`). **No node-side gate change** — they already want the pubkey; the value becoming
correct is the whole fix.

**M10.4-D3 — MP-C-16 clean-green witness lands at M10.5.** The design names the proof obligation: the
box-gated `mp_r2_fixed::mp_c_16_live_migration_space_rehomes` must `require_ok` the `migration initiate`
(Site 1 passes on the real-binary client path) + the cutover passes (Site 2) + the Space replicates to
B + `home_node` flips to B on **both** nodes (the per-Space home query the witness doc-comment flags,
mp_r2_fixed.rs:319-322). Per the locked sequence the box-gated re-run is **M10.5**; M10.4 ships the fix
+ fast witnesses (§4).

**M10.4-D4 — Layer-2 route recorded, NOT a dependency.** Production identity→home-node discovery of an
*unknown* identity (gossip / directory / DHT / XGID-encoded home) = the **F1B-D5 / DM-stranger sibling**
(audit §3), genuinely heavy, **separately routed** on the ROADMAP horizon. Confirmed NOT required for
MP-C-16 (every migration identity is supplied or self-known). The F1b DM-stranger convergence boundary
(F1B-D4) is **not** incidentally cleared by this fix and stays as recorded.

**M10.4-D5 — persistence = leave-as-legacy, named, out of scope.** Pre-existing Spaces carry a URL
`home_node` (signed content — immutable, cannot be rewritten in place). Shape B makes **new** Spaces
correct; pre-existing Spaces stay URL-homed (legacy). MP-C-16 creates a fresh Space each run (moot). A
real migration of legacy Spaces is its own concern — **named here + on the ROADMAP horizon, not built
in M10.4.**

## 3. Impl surface (audit-grounded; Clair confirms file:line at runbook)

- **Wire (D1):** `node_id: Option<String>` on `TransportMessage::AuthOk` (`xgen-common/src/wire/types.rs:64-68`);
  node populates at AuthOk send (the `AuthOk` construction site in `xgen-node`, from `node_id_uri`
  `app.rs:632`).
- **Client (D1):** `SessionState` gains a `node_id` field (`session.rs:73`-adjacent, beside the URL
  `home_node`); `client_authenticate` captures the echo (`connection.rs:400,422`);
  `build_space_create_event` / the DM create path write `content["home_node"] = session.node_id`
  (`ops.rs:449-456`) instead of `ctx.session.home_node` (the URL). `--node-override` semantics: the
  override is a *transport* URL, so when overriding, the node_id must still come from the AuthOk of the
  connection actually used (Clair confirms the override interaction at runbook).
- **Node (D2):** **no change** — the six `SpaceState.home_node` consumers + both migration gates
  already compare against the pubkey; they pass once the stored value is the pubkey.
- **Persistence (D5):** none built; named only.

## 4. Proof obligations (RED-on-revert; Clair builds in the runbook)

1. **Namespace value-correctness** — after a `create_space` on the real-binary client path, the
   resulting `SpaceState.home_node` equals the node's pubkey node_id (not the WS URL). RED on revert
   (revert → URL). The fast witness that fixes the in-process-test blind spot (audit §1.2: the unit
   fixtures wrote a pubkey, hiding the bug).
2. **Site 1 clears** — `migration initiate` on the genuine source node passes the homed-here
   precondition (no `MIG_6010`) when the Space was created via the client path. RED on revert.
3. **Site 2 clears** — the `state.space_migrate` cutover passes `validate_event` (no `6009`) + applies.
   RED on revert.
4. **AuthOk echo** — the client receives + stores the node_id from `AuthOk`; an old/echo-absent path
   degrades safely (Clair names the fallback — likely error-on-create rather than silently writing a
   URL; confirm at runbook).
5. **(M10.5, box-gated)** MP-C-16 end-to-end re-home green — named here, lands at M10.5 (D3).

## 5. Design-close details (Clair confirms at runbook; Joe-call only if non-obvious)

- The exact `AuthOk` field shape + whether any `protocol_version` note is warranted (additive optional
  → no bump expected; confirm).
- The client fallback when `node_id` is absent from `AuthOk` (older node): create_space should **not**
  silently fall back to writing the URL (that reintroduces the bug) — prefer an explicit error or a
  guarded path. Clair picks the safe shape; Joe-call only if it forces a UX decision.
- The `--node-override` ↔ node_id interaction (the override is a URL; the node_id must match the
  connection used).

## 6. Close deliverables (at M10.4 close)
- Appendix F: if the `AuthOk` echo is operator-visible (likely not — it's an internal handshake field;
  confirm). The `create_space` behaviour note if warranted.
- ch3: the `AuthOk` wire field (additive) recorded in the transport-message spec section.
- Findings flips: **M10.4-A1 RESOLVED** (namespace reconciled), **M10.4-A2 RESOLVED** (both sites),
  **M10.4-A4 RESOLVED-as-Shape-B**; **M10.4-A3** confirmed (escape didn't fire); **M10.4-A5** recorded
  (leave-as-legacy). **MP-F13** → RESOLVED at the M10.5 MP-C-16 re-run (not at the M10.4 code commit —
  no unobserved-result claim; J-352 precedent).
- DECISIONS: candidates only, arc-local (D-069). The "Space `home_node` is the pubkey node_id, written
  by the client from the AuthOk echo" invariant is a DECISIONS candidate (not promoted yet).

## 7. Next-active

Clair: author `tasks/M10_4_HOME_NODE_DISCOVERY_IMPL.md` (the `AuthOk` node_id echo + client capture +
the create_space/dm_create write + the §4 witnesses), confirming the §3 groundings + §5 details to
file:line → implement → Chat doc-bridge → close → M10.5 (MP-C-16 re-run + MP-F6 fold + MP-C-06).
No code until the runbook lands.

## 8. Close (J-371)

**SHIPPED + CLOSED.** Clair shipped 4 commits (`cfa9775` runbook → C1 `77c906d` / C2 `59e9193` /
C3 `de53cf0`); D1–D5 honored. Verified `cargo test --workspace` **1397/0** (+7 witnesses over the 1390
baseline), clippy clean (default + all-features). Chat spot-checked the load-bearing groundings (the
`AuthOk.node_id` additive field; the `create_space` pubkey write + connect-before-build + absent-echo
refusal) — all confirmed.

**Grounding corrections surfaced at impl (D-065; none forks a locked decision — touch-points relocated,
the shape held):**
1. The wire types + auth fns live in **`xgen-core`**, not `xgen-common`/`xgen-client` as §3 cited
   (`xgen-core/src/wire/types.rs` AuthOk@64-73; `xgen-core/src/transport/connection.rs`
   `server_authenticate`@~375 / `client_authenticate`@~401). Line numbers held; only the crate was wrong.
2. D1 landed as **two additive `xgen-core` API touches** — `server_authenticate(node_id)` (the value was
   already in scope as `handle_connection`'s `home_node_id`, app.rs:1450, from `node_id_uri` @632 — no
   `run_node` plumbing) and `client_authenticate → AuthOutcome` (return widened to carry the node_id; of
   ~80 callers only the 3 binding sites + ai_service/injector/wireactor changed; the `server_authenticate`
   arg rippled to ~17 test callers — surfaced by `cargo test --workspace`, which `cargo build` skips).
3. `create_space`/`create_dm_space` needed a **connect-before-build reorder** (the node_id is captured at
   auth; the URL is kept only for the client's own `node_endpoint` dial record).

**Findings flipped (this bridge, §6):** **M10.4-A1 RESOLVED** (namespace reconciled — client writes the
pubkey); **M10.4-A2 RESOLVED** (both stall sites clear with a pubkey, fire on a URL — C3 witnesses);
**M10.4-A4 RESOLVED-as-Shape-B**; **M10.4-A3** confirmed (escape did not fire); **M10.4-A5** recorded
(leave-as-legacy). **MP-F13 → RESOLVED at the M10.5 MP-C-16 re-run, NOT at this code commit** (no
unobserved-result claim; J-352 precedent). **One honest deviation (D-065):** C3 **Witness 3** proves the
cutover authority gate at the **applier level** (`apply_space_migrate`, the documented defensive twin of
`validate_event`'s 6009 — same `sender == home_node` equality), not by driving `validate_event` (whose
fixture was disproportionate for a fast witness); the `validate_event`-level + end-to-end re-home is the
M10.5 box-gated MP-C-16 (D3) regardless. **No DECISIONS change** (arc-local, D-069; the "Space
`home_node` = pubkey node_id from the AuthOk echo" invariant remains a candidate). **Next: M10.5.**
