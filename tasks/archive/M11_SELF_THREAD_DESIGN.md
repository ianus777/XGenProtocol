# M11 — `self` Thread: Design (D-021)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

The M11 design, authored after the Phase-0 audit (`tasks/M11_SELF_THREAD_PHASE0_AUDIT.md`
v1.0, grounded @ `345a461`) returned its verdict: **B is admissible — C not needed.** Carries
the design decisions M11-D1..D5 (arc-local, D-069) for Joe's lock. No code until lock. The
runbook is Clair's next step after lock.

## Locked concept (recap)

M11 `self` = a **Node-side, never-federated, never-broadcast personal thread reusing the user's
existing keypair** (single identity — `self` is the user, not a second account; D-021), realized
as a **self-DM** (shape B) on the existing Space/Room/Event/DAG + DM-non-federation machinery.
Text-first; M12 attachments inherited. See the Phase-0 brief + D-021.

**Why B over C (faithfulness, recorded):** B reuses the convergence-proven DM primitive with zero
protocol/applier delta (project discipline: reuse the proven path, don't grow a parallel one);
`self` is the user's real registered identity on both endpoints (no-anonymity pillar); and B
inherits the **hard** `DmFederationNotAllowed` guarantee (privacy as a structural property, not a
default). C (single-member regular Space) would need a new creation path and has only a *default*
non-federation posture. The lone count C wins — conceptual "place not conversation" — is satisfied
by the keypair-reuse, and the D1 guard makes B behave as a clean single-member room anyway.

## Phase-0 findings this design rests on (audit-grounded)

- `from_dm_space_create` (state.rs:342) and node-side `from_dm_space_create_node` (state.rs:487)
  both **admit `invitee == creator`** — no error, no guard, no break.
- The creator is the sole **Owner** member + **dm-Room** member the instant the create chain
  lands (`apply_room_create` auto-inserts the sender, state.rs:792); **no `membership.join` is
  ever needed.**
- The only artifact is a **vestigial `pending_invites[self]`** that is never consumed and never
  errors (`apply_join` short-circuits `AlreadyMember`, state.rs:1000-1001).
- The creator passes every step-11 gate (registered 629 / Space member 672 / Room member 676) →
  can post and read. **It's a usable thread today.**
- `CreateDmSpaceArgs = { invitee }`, no self-guard (app.rs:543); `create-dm-space --invitee
  <own-id>` works on current `main`. The auto-invite's swallowed reject is accept-either and is
  the behaviour of every DM (ops.rs:786-787, 740-742) — self introduces no new failure.
- Reach: member-gated `collect_sync_history` (fanout.rs:457) serves the self-DM to any client
  authenticated as the user.
- Non-federation doubly contained: hard `DmFederationNotAllowed` (state.rs:660-661) + degenerate
  `{this_node}` party set (runtime.rs:2101).
- Identity reuse: the session identity, no new registration (ops.rs:670-677).

## Decisions (M11-D1..D5, arc-local D-069)

### M11-D1 — vestigial self-invite: guard at construction (LOCKED)
Skip the auto-invite when `invitee == creator`, in **both** constructors (`from_dm_space_create`
state.rs:342 and `from_dm_space_create_node` state.rs:487). The creator is still seated as Owner +
dm-Room member via `apply_room_create` (unchanged). Result: a self-DM is a **clean single-member
room by construction** — no vestigial `pending_invites[self]`.
- **Scope:** constructor-only. **No `apply_join` belt-and-suspenders check** — `apply_join`
  already short-circuits `AlreadyMember` (state.rs:1000-1001), so even a stray self-invite would be
  inert; adding a second guard is redundant and widens the touch. **(LOCKED: constructor-only.)**
- **This is the entire applier/protocol delta for M11.** A few lines, two sites, no wire change,
  no new event type, no new reject code.

### M11-D2 — reach wording (LOCKED as wording, no code)
State reach **precisely** in the design, the ch6 note, and any help text: a self thread is
reachable from **"any client authenticated as the user (their own devices)"** — it is
**Node-resident, not device-local.** Do not write "any client on the Node" unqualified (it would
overclaim — only the user's own authenticated clients see it). This is an honesty/wording lock
(D-065), not a behaviour.

### M11-D3 — client surface (thin, client-only)
A thin `xgen-client` convenience to create + open the self thread, plus a **"self" / "Saved
Messages"** display label. Built entirely over existing surfaces — `create-dm-space` (app.rs:543),
`Send`, `History` — which already work. **No new wire, no applier change beyond D1.** Per D-092 the
verb-add carries its dispatch arms only insofar as it is a real new verb (see D5); a pure label +
auto-target convenience over `create-dm-space` may not need all four arms — the runbook grounds the
exact surface and arms.

### M11-D4 — ch6 descriptive note (the close deliverable)
At M11 close, add a short **ch6** (client-design) note — **NOT** a ch3 normative edit (no new
protocol surface). Contents: what it is (personal single-user thread; messages + chronological
history); **reuses the user's existing identity** (the anchor line that prevents drift back to "a
separate account"); never-federated / never-broadcast **by reference** to `DmFederationNotAllowed`;
**attachments as an inherited general capability** (same mechanism as any Space; see M12), present-
tense about the concept, forward-referenced about the mechanism; and the boundary — not an account,
not a Node-side service, no new protocol surface. Authored at close so it reflects the shipped shape.

### M11-D5 — self-target UX (LOCKED)
Don't make the user type their own identity id into `--invitee`. Auto-resolve the session
identity as the invitee. **The remaining choice is the surface shape:**
- **(a)** a dedicated `self` convenience verb (e.g. `self open` — creates-if-absent, then opens);
- **(b)** a `--self` flag on the existing `create-dm-space` (fills invitee = session identity);
- **(c)** document the raw `--invitee <own-id>` form only (zero new surface).

**Recommendation: (a) a `self` convenience verb** — it reads as the feature ("open my self
thread"), is idempotent (create-if-absent), and is the natural label home; the raw form (c) stays
the documented floor beneath it. (b) is the cheapest real code; (c) is the thinnest but the worst
UX. **LOCKED: (a) the `self` convenience verb** (create-if-absent → open; auto-resolves the session identity, no typed id); the raw `--invitee <own-id>` form (c) stays the documented floor.

## Witness set (RED-on-revert obligations for the runbook)
- **W1 (D1 core):** a self-DM create yields **no** `pending_invites[self]` entry. RED-on-revert:
  remove the guard → the vestigial entry reappears.
- **W2 (functional):** post-create, the creator is Owner + dm-Room member and can post + read.
- **W3 (non-federation):** the self-DM never federates (`DmFederationNotAllowed` holds; party set
  degenerate `{this_node}`).
- **W4 (reach):** a second client authenticated as the same user sees the thread (member-gated
  sync) — proves D2's "own devices" reach.

## Out of scope (named)
- Attachments → M12 (inherited; the ch6 note forward-references).
- Operator-confidentiality / E2E → moot for B (audit §7; self lives on the user's own node).
- Renaming the internal `DM` primitive for the one-party case → **named, not fixed** (an
  internal-vocabulary wart that never reaches the user or the wire; renaming a core primitive for
  a thin client feature would violate scope discipline, D-069).
- Any new wire type, event kind, reject code, or ch3 normative edit — there are none.

## Next
On Joe-lock (D1 confirmed + D5 chosen), Clair authors `tasks/M11_SELF_THREAD_IMPL.md` — the D1
guard (two sites) + the D5 surface + W1–W4 witnesses + the ch6 note as the close deliverable →
implement → Chat doc-bridge → close. No code until the runbook lands.

## Entry (Rule 0)
CLAUDE.md PLAY → JOURNAL (latest) → this design → `tasks/M11_SELF_THREAD_PHASE0_AUDIT.md` →
`tasks/M11_SELF_THREAD_PHASE0_BRIEF.md` → `DECISIONS.md` D-021.
