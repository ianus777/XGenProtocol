# Pre-UI Node↔Client Functional Gate — Phase-0 (M-RP6.0)
> **Status**: PENDING  
> Version: 0.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

A **live functional re-verification** of the node↔client surface, run **before** the main client UI panel is built, so that UI integration follows up on a confirmed-working base. This is a **D-071 subsystem audit** (the client UI panel is the dependent milestone; the node↔client channel is what it depends on) and the **live-functional slice** of the already-planned Round-2 whole-codebase audit. **Not** a re-run of the closed multiparty milestone (J-356) — it re-checks the *current* binaries after all Round-1 arc work.

*(Name provisional — `M-RP6.0` used as the gate preceding the client-UI-panel arc; rename on lock.)*

---

## Why now

The multiparty tests are fully closed (MP-R1 J-340 → MP-R2 J-348 → MP-R3 capstone J-356, ledger delivered) — but that evidence predates the entire UI/RP track and much arc work since. Before wiring a real client UI panel onto `ops::*`/state, a lean live pass confirms the node↔client channel still behaves, catching drift early rather than mid-UI-build (D-065 / "honest longer work over fast shortcuts").

## Scope (recommended — lock or adjust)

Single-node, client-facing surface (what the client UI panel will actually sit on). Federation topology stays out.

- **G1 — Connect + state.** Client↔node handshake; `get_state` transitions DISCONNECTED→CONNECTED.
- **G2 — State sync.** Client reads node-held identity/space/room state (the descriptor source the UI maps from).
- **G3 — Send/receive round-trip.** One message delivered end-to-end on a single node.
- **G4 — Multi-client / one node.** Two clients on one node see each other's effects (reuse `MULTIPARTY_S1`).
- **G5 — Client rebind.** Reconnect/resume after drop (reuse `MULTIPARTY_S5`).

**Out of scope (deferred to a later gate):** federation topology + multi-node propagation (`MULTIPARTY_S3`/`S4`) — not needed before a single-node client UI panel.

## Method

Reuse the `xgen-mptest` harness where it covers G1–G5; live-run the **real** `xgen-node`/`xgen-client` binaries (CDP 9222/9322 per the J-405 self-drive loop) for anything harness-external. Real output quoted per Rule 2; no fabricated results (Rule 1).

## Definition of Done

Each of G1–G5 green with **real quoted output**; a short findings list (any drift → logged, fixed or explicitly deferred); a **GO / NO-GO** verdict for opening the client UI panel arc. (No "commit pushed" checklist item — the `COMPLETED` header is the shipped signal.)

## Roadmap consequence

`M-RP6.0` (this gate, PENDING) → *(GO)* → **client UI panel arc** (main client UI, assembles the shipped `core`/dd/widget components onto real node↔client state). A NO-GO routes to fix-the-drift first.

---

*Test/verification Phase-0. No protocol/data change. Scope recommended, not yet locked — Joe locks G-set + name before the gate opens. Supersedes nothing; the multiparty milestone stays closed (J-356).*
