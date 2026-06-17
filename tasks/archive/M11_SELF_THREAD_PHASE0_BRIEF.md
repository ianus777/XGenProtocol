# M11 — `self` Thread: Phase-0 Framing Brief (D-021)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Framing brief that OPENS M11 — the `self` thread (D-021). Authored by Chat at the M11-open
doc-bridge (J-376) after the concept and the Phase-0 scope were Joe-LOCKED in discussion.
This brief carries the locked concept + the Phase-0 grounding scope + the forks; it is **not**
a design or a runbook. Supersedes `tasks/HANDOFF_M11.md`.

D-071 arc discipline: Phase-0 audit (Clair, this brief is its agenda) → design → Joe-lock →
runbook → Clair implements → Chat doc-bridge → close. **No code before Joe locks the design.**

## Provenance (grounded J-376)

`self` has **zero detailed grounding in the protocol spec** — a fresh grep of ch0–ch6 and all
appendices A–L returns nothing (every "self" hit is unrelated: self-certifying, self-declared,
Rust `&self`). The sole original source is **D-021** (DECISIONS.md, 2026-04-28), a terse
deferral note with "Spec reference: —". ROADMAP carries name-only placeholders. So M11 builds
something the protocol never described — which is **why the surgical relaxation of D-021's
pre-machinery wording is legitimate, not drift**: there is no specified mechanism to override.

## The LOCKED concept (Joe, J-376)

**M11 `self` = a Node-side, never-federated, never-broadcast personal thread that reuses the
user's existing keypair** (single identity — `self` is *you*, not a second account). Streams
omitted. It is realized on the existing Space/Room/Event/DAG + DM-non-federation machinery;
**chronological history and "reachable from any client on the Node" are free properties of the
Space apparatus**, not new mechanisms. M12 attachments are inherited later (text-first at M11).

Reconciliation with D-021:
- **Relaxes** "never registered on any Node" → registered via the existing (already-registered)
  identity. This is the single pre-machinery clause that collides with the as-built system.
- **Keeps** D-021's spirit: own keypair (the user's), never federated, never broadcast, private,
  Node-mediated reach.

Why **B (reuse existing keypair)** over the alternatives (decided in discussion):
- **vs A (own synthetic key):** a second key would quietly turn "my own space" into "a separate
  little account" — the server-shaped reading the concept moved away from — and a second key that
  signs stored events would itself need local registration (step-11 gate). B reuses the identity
  already in the registry: no new key, no new registration.
- **vs C (single-member regular Space):** C is the named **fallback** if the self-DM admissibility
  edge (below) resists — it has no invitee and no admission edge, but lacks the DM path's *hard*
  `DmFederationNotAllowed` guarantee (regular Spaces are non-federated only by default).

## Grounding findings that shaped the lock (D-021 vs live code, J-376-verified)

Verified against the canonical main tree (`xgen-core`), line numbers confirmed:
1. **"accessible from any client / not device-local" — HARD (resolved by Node-side).** A client
   Identity is a keypair file in the client's own `data_dir` (`ClientIdentity::load`,
   session.rs:60) — device-local by construction. Node-side `self` makes reach a free property of
   Space-sync; no Node-side key-vending invented.
2. **"never registered… signs local Events" — HARD (resolved by relaxing the clause).**
   `validate_event` step 11 rejects any signer absent from the `IdentityRegistry` with
   `UnknownSender` (exchange.rs:202-209). A never-registered signer is rejected by its own local
   node → "never registered" cannot coexist with "signs stored Events"; B reuses the registered
   identity, so the clause is satisfied, not violated.
3. **"never broadcast" — NO CONFLICT, it is the opportunity.** DM Spaces already never federate
   (`DmFederationNotAllowed`; "no third-party node ever receives DM content", runtime.rs:2105);
   `dm_constraints_active` machinery is heavily built. **Caveat (the one real unknown):**
   `from_dm_space_create` (state.rs:342) has **no `invitee == creator` guard** (verified — only
   wrong-type / missing-field error returns), so a self-DM is neither rejected nor proven safe.

## Phase-0 scope (Clair audit agenda — Joe-LOCKED)

**Headline question (decides B vs C):** the **self-DM admissibility edge**. Trace
`from_dm_space_create` (state.rs:342) → `apply_join` when **invitee == creator** (one identity is
both owner and pending-invitee). Does it work cleanly, double-insert, role-collide, or error?
This single finding decides whether `self` stands as a self-DM (B) or is better realized as a
single-member regular Space (C).

**Supporting grounding (in order):**
1. **Registration cost** — confirm B reuses the already-registered identity → **zero new
   registration** (not even the local-registration path). If it holds, "local registration
   mandatory" drops out entirely (it was an artifact of the abandoned own-keypair option).
2. **DM-creation entry point** — how a client drives `StateDmSpaceCreate` today (the
   `invitee` / `home_node` / `auth_tier` content fields), and whether a client can issue a
   self-DM through the existing path unchanged.
3. **Reach** — confirm existing Space-sync-to-any-connecting-client delivers "every client on the
   Node sees the same thread" with no new work (the load-bearing claim behind Node-side; ground,
   don't assume).
4. **Client surface (the individual-side locus)** — in `xgen-client` / ch6, where `self` appears
   and the minimal client work to create + open the thread.

**Operator-confidentiality refinement (low priority):** Row-3 only proves *other* nodes never see
DM content; whether the *own* node operator can read it (E2E) is separate — but moot under the
"`self` lives on the user's own home_node" model. Ground only if it bears on the C-vs-B call.

## Named deliverable (doc, not a design item)

A short **ch6** (client-design) descriptive note — NOT a ch3 normative edit (no new wire types):
- what it is (personal single-user thread; messages + attachments; chronological history);
- **reuses the user's existing identity** (the anchor line — the clause that prevents the concept
  drifting back to "a separate account");
- never-federated / never-broadcast, by reference to the existing DM-non-federation constraint;
- **attachments as an inherited general capability** (same mechanism as any Space; see M12) —
  present-tense about the concept, forward-referenced about the mechanism;
- the boundary: not an account, not a Node-side service, no new protocol surface.
Authored at M11 close (so it reflects what Phase-0 confirmed).

## Forks (Joe-LOCKED at J-376)

- **F1 — shape:** target **B (self-DM, reuse existing keypair)**; **C (single-member regular
  Space)** is the named fallback if the admissibility edge resists. LOCKED.
- **F2 — registration:** expect **zero new registration** under B; Phase-0 grounding item 1
  confirms. LOCKED (provisional "none", confirmed by grounding).
- **F3 — scope:** **text-first** at M11; attachments inherited at M12. LOCKED.

The keypair fork and the data-shape fork **fold into the same unknown**: whether the DM machinery
tolerates `invitee == creator` decides if `self` is a self-DM (B) or a single-member Space (C).
That is the one thing Phase-0 settles, and it is contained.

## Lock-time canonical correction (applied at this bridge)

ROADMAP previously tagged M11 "identity-layer; rides the M10 / auth-module work." Under the locked
B shape M11 is a **client/Space feature reusing an already-registered identity** — it does **not**
touch the auth-module tier work. The ROADMAP M11 entries are corrected at this J-376 bridge.

## Post-M11 chain (J-357, authoritative)

M11 → M12 (attachments) → Round-2 final pre-UI whole-codebase audit (the UI gate) → UI →
Streams (standalone, post-UI). The Round-2 audit is **not** next — it sits after M11/M12,
immediately before UI.

## Routed-open items (named homes, non-blocking — keep in view, don't re-open)

- **MP-F12** — departed-signer (own home)
- **MP-F2-followon** — 7 unmapped wire-codes
- **MP-F15** — migration-depth arc (destination admission keyed on home-ownership)
- **MP-F16** — federation-endpoint inconsistency (`config.node.listen` raw vs
  `effective_endpoint`); low-sev, harness-cleared at J-375

## Entry (Rule 0)

CLAUDE.md PLAY → JOURNAL J-376 → this brief → `DECISIONS.md` D-021 → the live identity/DM machinery
(`from_dm_space_create` state.rs:342, `validate_event` step 11 exchange.rs:202-209,
`dm_constraints_active` runtime.rs:2105). No code until the M11 design is Joe-locked.
