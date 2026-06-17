# M10.5 — The M10-Routed Carve-Outs (MP-C-16 re-run · MP-F6 fold · MP-C-06 re-home) — Phase-0 Framing Brief
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What M10.5 is

The **final M10 sub-arc**. When it closes, **M10 closes**. It discharges the three carve-outs the
multiparty test rounds routed to M10 — they are the rounds' deferred items *coming home*, not fresh
scope. A **fix-and-rerun arc** (loop-to-green, D-065 / MP-R1-D10), not a one-shot close. Opened J-372
(this brief); no code until the MP-C-06 design is Joe-locked.

**Scope flag (do not lose):** even after M10.5, the consolidated R1+R2+R3 ledger (a standing
deliverable) still waits on **MP-R3's own close** (its MP-F14 fix-phase). M10.5 discharges the
M10-routed carve-outs, **not** that ledger.

## 1. The three carve-outs (grounded from the findings record)

Their character differs sharply — that difference is the whole framing:

- **MP-C-16 / MP-F13 — a verification re-run, not a build.** The Layer-1 root fix already shipped at
  M10.4 (Shape B: the client writes the Node's pubkey `node_id` — via the additive `AuthOk.node_id`
  echo — into `content["home_node"]`, J-371). MP-F13 was deliberately **not** flipped at J-371 (no
  unobserved-result claim, J-352). M10.5 stands up the box-gated end-to-end re-home witness
  (`mp_r2_fixed::mp_c_16_live_migration_space_rehomes`; D3 from J-370 = `require_ok` +
  home_node-flip-on-both), observes green → flips **MP-F13 RESOLVED**. Admin migration: the operator
  supplies both node_ids, so no discovery/notify is needed. **Loop only if the re-run surfaces a
  fault.**
- **MP-F6 — a bounded node-side fold.** The `let _ = …` swallowed-apply-error at `runtime.rs:691`
  (+ no dispatch-level `banned` pre-check), surfaced by the ban arc (J-337/J-338). The end-state is
  correct (resolution is a second gate — `apply_join` consults `banned`, state.rs:1003); the
  dishonesty is in the **reply** (`is_ok=true` for an event resolution will drop). LOW severity.
- **MP-C-06 — the real iteration risk + the load-bearing Phase-0 question.** A client/Space **re-home**
  where members must be *told* the home moved — distinct from MP-C-16's operator-driven admin
  migration. It was deferred (MP-R1-D10 / J-323) on two needs: (1) production — the unbuilt
  `home_changed` client emit/broadcast (J-278 CP-5 / J-279); (2) harness — key continuity across
  `--init` clients + the aicontrol `node_override` drop (per-command `--node`).

## 2. Scope — Joe-LOCKED (J-372, by-recomms)

- **C1a — MP-C-16 = verification re-run, loop-on-fault.** The M10.4 fix already shipped; M10.5 observes
  it end-to-end. Not a one-shot pass: if the re-run surfaces a fault, fix + rerun (D-065).
- **C1b — MP-F6 = bounded fold.** Sweep `dispatch_event`'s apply sites; the load-bearing question is
  whether the swallow is load-bearing **elsewhere** — an apply site where no second gate catches a
  dropped error. Fix the reply-dishonesty (surface the apply-error / add the dispatch-level `banned`
  pre-check); route anything load-bearing-elsewhere the sweep finds.
- **C1c — MP-C-06 = NARROW-FIRST WITH ESCAPE** (the J-369 shape that worked for M10.4). Phase-0 grounds
  **how load-bearing the J-278 `home_changed` emit is for MP-C-06**. If it is a *thin* emit over the
  already-proven receive side + the now-resolved source (see §3), build it. If Phase-0 surfaces a real
  broadcast/fan-out dependency, **surface + re-lock depth** — D-065-honest, do not smuggle a heavy
  broadcast arc into a re-home witness.

**Out (recorded):** the consolidated R1+R2+R3 ledger (waits on MP-R3's MP-F14 close, not M10.5); the
M8 S5 re-bind broadcast-observability half beyond what MP-C-06 needs; Layer-2 production
identity→home-node *discovery* of a stranger (M10.4-D4, separately-routed, never an MP-C-16/MP-C-06
dependency).

## 3. The Phase-0 crux (MP-C-06) — what grounding sharpens

The `home_changed` history makes the escape/narrow call concrete:

- **The receive side is already built (M8.5-C).** `identity.home_changed` applier / builder / sign /
  verify — a version-guarded signed delta; the EventType is registered (ch3 §3.13.9 / §3.13.10). Proven.
- **The client emit was deferred for exactly one reason (CP-5 amendment, M8.5-C v1.2).** The client
  *could not source* `new_home_node_id` — it only knew the WS URL (the J-278 gap). So the emit + a
  node-id echo source rode a follow-on "re-home notify" arc.
- **M10.4 just dissolved that blocker.** The `AuthOk.node_id` echo + the `SessionState` stash mean the
  client now learns the Node's pubkey id. The single thing CP-5 said the emit was waiting on now exists.

So the M10.5 Phase-0 grounding question is **not** "build a big broadcast arc." It is: *given the
receive side is proven and the source is now resolved by M10.4, how thin is the `home_changed` client
emit — and what does MP-C-06 additionally need on the harness side (key continuity across `--init`
clients, the aicontrol `node_override` drop / per-command `--node`)?* That answer decides whether
M10.5 is **"two re-runs + a fold"** or **"+ a thin emit arc."**

**Phase-0 (Clair, D-071) must ground:**
1. **MP-C-16 re-run rails** — the box-gated `mp_r2_fixed::mp_c_16_live_migration_space_rehomes` against
   final HEAD; confirm the M10.4 fix flows end-to-end (both stall sites clear: Site 1 `MIG_6010`
   admin_ops.rs:2096; Site 2 cutover `6009` exchange.rs:717 + applier re-check state.rs:1158).
2. **MP-F6 apply-site sweep** — enumerate `dispatch_event`'s `let _ = …apply…` sites; for each, ask
   D-077-bidirectionally: is the swallow load-bearing forward (a later caller bypasses a needed error)
   or backward (a current caller depends on the silence)? Pin the honest fix shape.
3. **MP-C-06 emit load-bearing-ness** — the `home_changed` emit path from the client (now that the
   source is resolved): does it ride existing fan-out (the receive side + Space federation set), or
   does it need new broadcast machinery? Plus the harness re-home rails (key continuity / `--node`).
   Ground the verb/aicontrol surface MP-C-06's re-home step drives.

D-065 / D-078: ground symbols in production code; if grounding contradicts a locked call, surface +
re-lock.

## 4. Close criterion

**All-green, no carve-out** — these three *are* the carve-outs, coming home. C1a green (MP-F13 RESOLVED
on observed green) + C1b folded + C1c green (MP-C-06 witnessed, or its escape re-locked and the
re-locked depth shipped) → **M10.5 closes → M10 closes.** R1/R2/R3 loop-to-green rerun character
applies (J-322/J-344/J-351): a faced fault gets fixed and rerun, not papered over.

**Post-M10 sequence (unchanged, J-357):** M11 → M12 (attachments) → Round-2 final pre-UI gate → UI →
Streams (standalone, post-UI). The consolidated R1+R2+R3 ledger rides MP-R3's MP-F14 close, on its own
track.

## 5. Entry (Rule 0)

`CLAUDE.md` PLAY → `JOURNAL.md` J-372 → this brief → `tasks/MP_findings.md`
(MP-F13 / MP-F6 / MP-C-06 source rows) → `tasks/M10_4_HOME_NODE_DISCOVERY_DESIGN.md` (the shipped
Layer-1 fix MP-C-16 verifies) → `tasks/M8_5_C_S5_REBIND_DESIGN.md` (the `home_changed` receive side +
the CP-5 emit deferral MP-C-06 picks up).

**Next-active: Clair opens the M10.5 D-071 Phase-0 audit** — ground §3.1 (MP-C-16 re-run rails) + §3.2
(MP-F6 apply-site sweep) + §3.3 (MP-C-06 emit load-bearing-ness + harness rails) → design (lock the
MP-C-06 build-vs-escape + the MP-F6 fix shape + the witness set) → Joe-lock → runbook → impl → close.
No code until the M10.5 design is Joe-locked.
