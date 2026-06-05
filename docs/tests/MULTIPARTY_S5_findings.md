# Multiparty Test S5 — Findings (M8 / Wave 2 / C4)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this records

The **S5** (client rebind / identity portability) verdict for M8 Wave 2 / C4. A client
re-homes from one Node to another while keeping the same Identity. **Verdict: BLOCKED** — the
required CLI/wire surface is not present on B. This is a **capability gate**, not a protocol
bug, and it is exactly the gate the `MULTIPARTY_S0_intro.md` file warned about ("S5 — capability
gate (potentially blocking)"). Per **M8-D4**, a surfaced gap is a *success* that scopes M9; no
surface is built in M8. B stamp: `8b14aa8` (≡ `676b9c1`).

---

## Capability gate (grounded against the B tree)

S5 requires three surfaces (`MULTIPARTY_S5_client_rebind.md` M0.3, spec §3.6 / §3.13):

| # | Surface | Spec | State on B | Verdict |
|---|---|---|---|---|
| 1 | `re_registration` flag on `identity.register` | §3.6 (~L1803) | **Not exposed.** `RegisterArgs` (`xgen-client/src/app.rs:459`) has only `--name`; no `--re-registration` flag, and the wire `identity.register` path is not driven with it from the CLI. | **MISSING** |
| 2 | `identity.replicate` push (home → replicas) | §3.13.4 | **Wired.** `EventType`/`IdentityReplicateMessage` exist; `handle_identity_replicate_msg` ingests replicas. But CLI observability of the rebind is not surfaced. | partial |
| 3 | `identity.home_changed` observability | §3.13.8 (~L4428) | **No EventType.** There is no `IdentityHomeChanged` variant in `xgen-common/src/wire.rs::EventType`, so the new-home notification cannot be emitted/observed. | **MISSING** |

**Buildable surfaces: 1 of 3.** The two missing surfaces (the `re_registration` re-home flag
and the `identity.home_changed` notification EventType) are exactly the orphan-recovery /
re-home path; without them, a client cannot re-home keeping its Identity *and have it
observed* at the CLI level. Per the S0 anti-pattern rule ("Working around a missing
capability"), no manual JSON / low-level wire workaround is constructed.

---

## Verdict: **BLOCKED** (M8-D4 → M9 input)

S5 is **BLOCKED**, not FAIL — the protocol design (spec §3.13 identity replication / orphan
recovery) is coherent; the **CLI + one wire EventType are incomplete** for the rebind path.
This is recorded as an **M9-scoping input**:

- The missing surface to make S5 runnable (a future arc, not M8): a `--re-registration` flag
  on `xgen-client register` threaded to the `identity.register` wire field, **and** an
  `identity.home_changed` EventType + emission on Node re-home for observability.
- M8 does **not** build these (consistent with the M8-D2 non-goal spirit + "do not build
  resident mode / new wire surface" discipline). The identity-replication substrate that *does*
  exist (`identity.replicate`, surface #2) is noted as the foundation a future rebind arc
  builds on.

---

## The four metrics (M8-D2)

Not applicable — S5 did not execute (BLOCKED before M0 completes). No M1–M4 measured; recording
"not run, blocked on missing surface" honestly (Rule 1/5) rather than fabricating a result.

---

## Definition of Done — S5

- [x] Capability gate evaluated against the live B tree (3 surfaces).
- [x] Verdict recorded: **BLOCKED** — `re_registration` CLI flag + `identity.home_changed`
  EventType missing (1 of 3 surfaces present).
- [x] Recorded as an M8-D4 / M9-scoping input; no surface built in M8 (no workaround).
- [x] Cross-referenced from `MULTIPARTY_S4_findings.md`.

---

*End of MULTIPARTY_S5_findings.md — S5 BLOCKED (recorded as an M9 input).*
