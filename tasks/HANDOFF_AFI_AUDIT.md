# AFI Audit Handoff — Appendix F / Appendix I audit-against-code
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-18  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## 0. What this is
Runbook for the first pre-UI arc after the documentation-optimization phase (COMPLETE, J-396). Reconciles two doc surfaces to the as-built code, **code as ground truth**:
- **Appendix F** (`docs/xgen_appendix_f_en.md`, v1.12) — client CLI reference — vs the `xgen-client` verb surface.
- **Appendix I** (`docs/xgen_appendix_i_en.md`, v1.6) — data structures — vs the `xgen-common`/`xgen-core` serializable types + protocol event catalog.
Gates the UI build: UI couples to the verb surface and renders the data structures, so both must match reality first (D-071 — subsystem audit precedes the dependent milestone).

## 1. Rule-0 reads (first, in order)
1. `CLAUDE.md` PLAY head — doc-opt COMPLETE; next frontier = this audit.
2. Latest `JOURNAL.md` entry — J-396 (DO-5 close).
3. This handoff.
4. Then `docs/xgen_appendix_f_en.md` / `docs/xgen_appendix_i_en.md` and the code.

## 2. Scope (Joe-locked)
- **Q1** One arc, two sub-passes: **AF** then **AI**.
- **Q2** Order: **F first, then I** (F is UI-proximate + has a known gap).
- **Q3** Finding IDs arc-local (D-069): **AF-F##** / **AI-F##**.
- **Q4** Reconciliation default: fix the **doc** to match code; **Joe-route** only suspected **code** bugs (a real defect, not a doc drift).
- **Q5** Phase-0 first deliverable: read-only **as-built inventory** before any diffing.
- **Q6** Dev/test harness verbs: **in scope for Appendix F**, documented in a clearly-marked "developer / test harness" section (present-but-segregated).

## 3. As-built inventory (Phase-0, read-only)
**AF surface — canonical `xgen-client/src/app.rs`** (`ClientCommand` + `ThreadCommand` + `AiCommand`). clap kebab-case default; only `self` is name-overridden. 31 leaf verbs:
init, whoami, status, spaces, rooms, version, register, create-space, create-dm-space, self, create-room, invite, ban, room-update, thread {create, resolve, archive}, join, leave, send, history, fetch (alias fetch-attachments), redact, members, ai {delegate, revoke, status}.
Dev/test harness (Q6, segregated): smoke-test, stress-test, smoke-ph2, stress-complete.

**AI surface — canonical `xgen-common/src/`** = 57 serializable `pub struct`/`pub enum` (wire.rs 10, state.rs 9, trust_assertion.rs 7, event_trace.rs 5, envelope.rs 4, module.rs 4, cmd.rs 3, codes.rs 2, bindings.rs 2, clock.rs 2, others 1 each) + the protocol event-type catalog (message.*, state.*, membership.*, thread.*, identity.*, space.*, room.*).

## 4. Method
- **AF:** for each of the 31 verbs — (a) present in Appendix F? (b) args match the `*Args` struct? (c) all four D-092 dispatch arms exist (CLI / run-path / batch / aicontrol)? Then reverse: every Appendix F verb still exists in code. Findings table: AF-F## | verb | drift type (doc-missing / code-missing / arg-mismatch / arm-gap) | severity | route.
- **AI:** enumerate the 57 types + event catalog from code; diff vs Appendix I both directions (D-077 forward-drift AND backward-coherence). Findings table: AI-F## | type/event | drift type | severity | route.

## 5. Reconciliation + close discipline
- Default fix = doc edit (code is truth). Loop-to-green: every finding closed green-to-criterion or Joe-routed with reason; no row left in-process (round-close discipline).
- Suspected code bugs are NOT fixed here — filed as Joe-routed findings (this is a doc audit, not a code arc).

## 6. Milestones
- **Phase-0** ✅ (this open) — scope lock + method + as-built inventory.
- **AF** ✅ (J-397) — verb diff done; Appendix F reconciled (AF-F01/F02/F04/F06 + AF-F03 reframe + §F.2.1 cross-ref); v1.12→v1.13.
- **AI** ✅ (J-398) — Appendix I reconciled to as-built (v1.6→v1.7; AI-F01–F16 doc-side, F15 no-op, F17 Joe-routed); three fundamentals promoted to new appendices M/N/O; `event_trace` folded into Appendix G (v1.2).
- **Close** ✅ (J-398) — consolidated AF+AI ledger (§9 below); both appendices reconciled; D-074 canonical close (JOURNAL + ROADMAP + CLAUDE atomic). Next: mockup stock-take + reconcile-to-as-built.

## 7. Operational learnings (carried forward)
- `Filesystem:*` for E:\ reads/writes; never create_file (sandbox).
- New-file writes: PowerShell .NET writer (UTF-8 no BOM, LF): `$enc=New-Object System.Text.UTF8Encoding($false)`; `[System.IO.File]::WriteAllText(path, ($arr -join [char]10)+[char]10, $enc)`. `Filesystem:write_file` is unreliable here.
- read: `Get-Content -Encoding UTF8`. Keep verification in a SEPARATE call from the write.
- Doc edits: index-reassign in PowerShell with a guard assertion on the target line, or `Filesystem:edit_file` with ASCII-only anchors (em-dash anchors unreliable).
- Header MUST be refreshed (Last updated + version) on every appendix edit.

## 8. Hygiene
- `tasks/HANDOFF_DO5_JOURNAL_WINDOWING.md` work is pushed; flip to COMPLETED + archive to `tasks/archive/` (DO-2 convention) — fold into a close commit.

## 9. Close — consolidated AF + AI ledger (J-398)

**Verdict:** both surfaces reconciled to as-built; AFI arc CLOSED. Doc-only, no code.

### AF sub-pass — Appendix F v1.12→v1.13 (J-397)

| ID | Verb / item | Drift | Disposition |
|---|---|---|---|
| AF-F01 | `create-dm-space` | code-present, doc-missing (F.0.4 + F.3) | rows added |
| AF-F02 | `leave` | code-present, doc-missing | rows added |
| AF-F03 | `federate` | stale "Deferred to M6 Phase 7" | reframed `N/A — only node concept` + ch2 node-to-node note (not removed) |
| AF-F04 | `members` | stale "deferred / no data source" + Network=No | de-staled; Network No→Yes |
| AF-F06 | node `whoami` | code-present (NodeCommand), doc-missing (F.2) | row added |
| — | node-admin tree | F.2 lacked any pointer to ~35 `AdminCommand` verbs | new §F.2.1 group-summary + pointer (no duplication) |

### AI sub-pass — Appendix I v1.6→v1.7 (J-398)

Backward-coherence (doc behind code) except F17. Code-as-truth (Q4); all doc-side except F17.

| ID | Target | Resolution |
|---|---|---|
| AI-F01 | §I.2 | `thread.create`/`resolved`/`archived` event rows added |
| AI-F02 | §VI.9 | `ThreadStatus` (open/resolved/archived) documented |
| AI-F03 | §VI.9 | `ThreadState` struct documented |
| AI-F04 | §VI.1 | `SpaceState` +`jurisdiction`/`e2e_encryption`/`threads` |
| AI-F05 | §VI.2 | `RoomState` +`permission_overrides`/`mls_commit_tip` |
| AI-F06 | §VI.7 | `PendingInvite` section + `valid_until` |
| AI-F07 | §II.1 | 8 transport variants (`sync_complete`, `invite_bootstrap_request`, 4×`blob_upload_*`, 2×`blob_fetch_*`) + `sync_request.limit` |
| AI-F08 | §I.2 + §X.3 | `identity.home_changed` registry row + `IdentityReplicateMessage` variant table added |
| AI-F09 | §IX | `message.file/reaction/redact` + `thread.*` content = handler-defined; Part IX honesty note added (not fabricated) |
| AI-F10 | §V.1 | `IdentityRecord` +`revoked`/`revoked_at`/`revocation_reason` |
| AI-F11 | §VIII.1 + §VIII.2 | `FederationRelationship` +`state`; new `FederationState` enum section |
| AI-F12 | §VI.8 | `RoomPermission` + `Effect` documented |
| AI-F13 | Appendix M | TrustAssertion family promoted to new Appendix M |
| AI-F14 | §II.1 | `transport.auth_ok` +`node_id` |
| AI-F15 | §II.1 | `transport.error.event_id` — already documented (no-op) |
| AI-F16 | §IV.1 | `identity.register` +`re_registration` |
| **AI-F17** | §IV.1 | **Joe-routed (suspected code gap):** wire `identity.record` omits `is_ai`/`ai_capabilities` that §IV.1 documents + §3.6.10 transparency expects. §IV.1 left intact pending a code decision. |

**New appendices (single source of truth per fundamental):** **M** Trust Assertions & Auth-Tier evidence (`trust_assertion.rs`), **N** Auth-Module / Plugin framework descriptors (`module.rs`), **O** `--aicontrol` control-plane structures (`aicontrol/*`). **Appendix G** v1.1→v1.2 — `event_trace` typed enums folded as a *Source Types* section.

**Scope (Joe-locked):** S1 observability / event_trace value-strings → G / out · S2 aicontrol → O · S3 module descriptors → N (M10 shipped J-375) · S4 clock.rs → out (D-090) · S5 internal registries/helpers → out. Reverse cross-refs into C/L/ch3/aicontrol-impl skipped (option B) — M/N/O+I+G already establish authority.

**Open item for Joe:** F17 routing — expose AI status on `identity.record` (code fix) vs. trim the two rows from §IV.1 (doc fix).

> **Post-close update (2026-06-18, J-402):** F17 is **RESOLVED** — the code-fix direction was Joe-locked (J-400) and shipped by Clair (J-401): `IdentityMessage::Record` now carries `is_ai`/`ai_capabilities` (serde-skip when false/None, populated on `identity.get`, back-compat tested). Appendix I §IV.1 already matched the target shape, so no doc edit was needed. The fields were never deferred at the model level — the omission-when-`is_ai=false` is an intentional byte-identical back-compat serialization rule. Settled; do not re-research (see J-402).
