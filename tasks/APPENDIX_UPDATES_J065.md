# Appendix Updates — J-065 Documentation Drift Cleanup
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Context

J-065 (2026-05-15) implemented D-059 / D-060 / D-061 in code: AI Identity, per-Space pacing, and the temperature property. The Ch3 spec was already current. Several appendices and supporting documents did not get updated in step and now drift from the implementation.

This task captures the punch list. Work is **Chat Claude scope** — documentation editing only, no code, no tests. Status PENDING; flip to COMPLETED when all sections below are addressed.

**Reference docs for this task:**
- J-065 entry in `JOURNAL.md` (full implementation summary with quoted test output)
- DECISIONS.md D-059 / D-060 / D-061
- `docs/xgen_ch3_specification.md` §3.6.10, §3.7.12, §3.7.13 (authoritative spec — these are correct)
- `docs/xgen_ch6_client_design.md` §6.12, §6.13, §6.14
- `tasks/AI_USERS_AND_PACING_ph2.md` (the implementation disposition, now COMPLETED)

**Authoritative source for what shipped:**
- `xgen-common/src/wire.rs` — all new EventType variants, content structs, constants
- `xgen-core/src/identity/registry.rs` — `IdentityRecord` with `is_ai` + `ai_capabilities`
- `xgen-core/src/identity/registration.rs` — error codes 3040 / 3041 (3042 in `xgen-core/src/message/exchange.rs`)
- `xgen-core/src/space/state.rs` — `SpaceState` with `human_pacing_ms`, `ai_pacing_ms`, `member_temperature_visibility`, `active_mutes`

---

## Recon already done (2026-05-15)

The recon below was completed in the session that wrote this file. Each finding lists the gap, the affected document, the verification source, and the recommended edit size.

### Finding 1 — Appendix I EventType registry incomplete (LARGE edit)

**File:** `docs/xgen_appendix_i_en.md` Part I §I.2 EventType Registry  
**Gap:** Five new event types are missing from the Phase 2 tables:

| New EventType | Wire string | Phase | Source |
|---|---|---|---|
| `MembershipMute` | `membership.mute` | Phase 2 (state) | `wire.rs` line 57 |
| `StateSpacePacing` | `state.space_pacing` | Phase 2 (state) | `wire.rs` line 139 |
| `StateSpaceTemperatureVisibility` | `state.space_temperature_visibility` | Phase 2 (state) | `wire.rs` line 144 |
| `StateAiOperatorDelegate` | `state.ai_operator_delegate` | Phase 2 (state) | `wire.rs` line 131 |
| `StateAiOperatorRevoke` | `state.ai_operator_revoke` | Phase 2 (state) | `wire.rs` line 134 |

All five are state events (stored in DAG, applied to SpaceState). They belong in the "Phase 2 — State events" subsection of Part I §I.2.

### Finding 2 — Appendix I IdentityMessage Register/Record schemas incomplete (MEDIUM edit)

**File:** `docs/xgen_appendix_i_en.md` Part IV §IV.1 `IdentityMessage`  
**Gap:**

- `identity.register` content table missing `is_ai` (bool, optional, default false) and `ai_capabilities` (object, required if `is_ai = true`, forbidden if `is_ai = false`)
- `identity.record` content table missing same two fields
- `identity.update` `changes` field description should note that updates to `is_ai` are rejected with error 3041 `ai_flag_immutable`

### Finding 3 — Appendix I IdentityRecord runtime object incomplete (SMALL edit)

**File:** `docs/xgen_appendix_i_en.md` Part V §V.1 `IdentityRecord`  
**Gap:** Field table missing `is_ai` (bool, default false, skipped from serialised output when false) and `ai_capabilities` (`Option<AiCapabilities>`).

### Finding 4 — Appendix I SpaceState runtime object incomplete (MEDIUM edit)

**File:** `docs/xgen_appendix_i_en.md` Part VI §VI.1 `SpaceState`  
**Gap:** Field table missing four fields:

- `human_pacing_ms` — `u64`, default 500 (DEFAULT_HUMAN_PACING_MS)
- `ai_pacing_ms` — `u64`, default 2000 (DEFAULT_AI_PACING_MS)
- `member_temperature_visibility` — `String` (open enum), default `"moderator"`
- `active_mutes` — `HashMap<String, String>` (target identity_id → RFC 3339 cooldown_until)

### Finding 5 — Appendix I Event Content Schemas incomplete (LARGE edit)

**File:** `docs/xgen_appendix_i_en.md` Part IX Event Content Schemas  
**Gap:** Five new content schemas needed (one per new EventType) plus updates to existing IX.1:

1. New **§IX.12 `state.space_pacing` content** — `human_pacing_ms` (u64, req), `ai_pacing_ms` (u64, req); see `StateSpacePacingContent` in `wire.rs`
2. New **§IX.13 `state.space_temperature_visibility` content** — `member_temperature_visibility` (string, req, open enum)
3. New **§IX.14 `membership.mute` content** — `target_identity` (string, req), `reason` (string, req — recognises `auto_temperature`), `cooldown_until` (string RFC 3339, req)
4. New **§IX.15 `state.ai_operator_delegate` content** — `space_id`, `ai_identity_id`, `new_operator_identity_id` (all string, req)
5. New **§IX.16 `state.ai_operator_revoke` content** — `space_id`, `ai_identity_id` (both string, req)
6. Update **§IX.1 `state.space_create` content** — add optional fields: `human_pacing_ms`, `ai_pacing_ms`, `member_temperature_visibility` (all settable at creation; defaults apply when absent)

### Finding 6 — Appendix I should add new auxiliary struct sections (MEDIUM edit)

**File:** `docs/xgen_appendix_i_en.md`  
**Gap:** Three new auxiliary structs warrant their own sections:

- `AiCapabilities` — fields `dm_initiate: bool`, `spontaneous_post: bool`, plus `extra: BTreeMap<String, Value>` for forward-compat. Logical home: new §V.3 after `DeviceRecord`.
- `TemperatureThresholds` — fields `warm`, `hot`, `fiery` (all f64). Constraint: `0.0 < warm < hot < fiery <= 1.0`. Logical home: extend Part VI or new section near Room metadata response.
- Reserved `meta_atts` keys for temperature — `xgen.room_temperature`, `xgen.member_temperature` (both float, clamped to `[0.0, 1.0]`). Reserved reason value: `auto_temperature` on `membership.kick` and `membership.mute`. Logical home: add to Part I or as a new section listing reserved namespace constants.

### Finding 7 — Appendix C conceptual class diagrams need extension (MEDIUM edit, optional fidelity)

**File:** `docs/xgen_appendix_c_en.md`  
**Header date:** 2026-05-06 (pre-J-065)  
**Gap:** Appendix C is a **conceptual model** (idealised event names like `room.member.kick`, not wire-format). Updating for J-065 means:

- Identity class gains `is_ai: bool`, `ai_capabilities: AiCapabilities` (or similar conceptual representation)
- New `AiCapabilities` class with `dm_initiate`, `spontaneous_post`
- Space/Room class gains pacing fields (`human_pacing_ms`, `ai_pacing_ms`) and `member_temperature_visibility`
- New temperature primitives surfaced (Room and Member temperature as conceptual properties)
- EventType enum gains conceptual entries (e.g. `room.member.mute`, `space.pacing.change`, `space.temperature.config`, `space.ai.operator.delegate`, `space.ai.operator.revoke`) — naming should follow the existing conceptual conventions in this appendix, NOT mirror the wire strings 1:1
- Header date bump

**Decision needed:** how faithfully to mirror wire format in this conceptual diagram. Discuss before editing.

### Finding 8 — Appendix D Privacy & Storage Identity Records table incomplete (SMALL edit)

**File:** `docs/xgen_appendix_d_en.md` §2.1 Identity Records  
**Gap:** Field table missing `is_ai` and `ai_capabilities`. The "What is NOT stored" subsection probably needs no change, but the new fields should appear in the stored-fields table with their privacy framing:

- `is_ai`: required for AI Identities, public, immutable. No privacy concern.
- `ai_capabilities`: required for AI Identities, public. Describes declared behavioural constraints. No privacy concern.

§2.2 Event DAG should note the new EventType `membership.mute` alongside `membership.kick` / `membership.ban` in the membership list (or update phrasing if it uses a wildcard).

Consider also adding a brief note about `xgen.member_temperature` visibility — it's stored on the Node but filtered per recipient per the `member_temperature_visibility` setting. This is unusual (Node-side enforcement of per-recipient filtering); worth a sentence in §2 or §3.

### Finding 9 — Appendix G Log Line Convention — NOT YET CHECKED

**File:** `docs/xgen_appendix_g_en.md`  
**Status:** Not opened during recon (ran out of attention budget). Likely needs new log line entries for AI rejection (errors 3040 / 3041 / 3042), pacing queue activity, and `auto_temperature` consequences. **First step of the next session:** open this file and assess.

### Finding 10 — Appendix F CLI Reference — LIKELY NO UPDATE

**File:** `docs/xgen_appendix_f_en.md`  
**Header date:** 2026-05-13  
**Status:** Scanned. The CLI surface did not gain new batch commands for pacing or temperature in J-065. Mention of `is_ai` flag on `register` is a possibility but D-059 framing says AI registration is an admin/operator decision, not a CLI flag — so leave alone.

**Final check needed:** confirm no new `--batch` commands or invocation forms shipped that would need documenting. Quick sanity-check, not a real edit.

### Finding 11 — Ch4 Implementation test count drift (TINY edit)

**File:** `docs/xgen_ch4_implementation.md`  
**Gap:** Currently states "300 tests" (J-058 figure). After J-065 the workspace is at 387 tests (352 xgen-core + 12 xgen-node + 23 xgen-client-lib). Update the count and add a brief note that D-059 / D-060 / D-061 are implemented.

### Finding 12 — Ch0 TOC has wrong appendix titles (BONUS, pre-existing — TINY edit)

**File:** `docs/xgen_ch0_content.md`  
**Pre-existing bug, not related to J-065:** Ch0 TOC lists incorrect titles for several appendices.

| Appendix | TOC says | Actual title in file |
|---|---|---|
| A | "Glossary" | "Why XGen Protocol Must Be Its Own Protocol — Not Built on Someone Else's" |
| B | "References" | "How XGen Protocol Funds Itself Without Selling Out" |
| C | "Data Model Diagrams" | "Primitive Schemas & Inheritance Diagrams" (close enough — could stand) |
| D | "Privacy & Storage" | "Node Data, Privacy, and Storage" (close enough — could stand) |

Recommend fixing at least A and B; C and D are acceptable shortenings.

There is no actual Glossary or References appendix at present. If Joe wants those, they'd be new files (likely as further appendices, J or K).

---

## Recommended work order

1. **Appendix G recon** (Finding 9) — open and assess. 5-minute task. Resolves the one unknown.
2. **Appendix I edits** (Findings 1–6) — single document, multiple distinct sections. Largest single-doc workload. Do in order: EventType registry → IdentityMessage → IdentityRecord → SpaceState → Event Content Schemas → auxiliary structs.
3. **Appendix D §2.1** (Finding 8) — small targeted edit.
4. **Ch4 test count + implementation note** (Finding 11) — tiny edit.
5. **Appendix F sanity-check** (Finding 10) — quick scan, likely no edit.
6. **Appendix G edits** (depends on Finding 9 outcome) — likely small.
7. **Discuss Appendix C scope** (Finding 7) — discussion required first; how faithfully to mirror wire format in a conceptual diagram is a design call.
8. **Ch0 TOC title corrections** (Finding 12) — separate small cleanup; bundle into the same commit or push separately.

---

## Definition of Done

- [x] Appendix G read and any required updates applied
- [x] Appendix I §I.2 EventType registry: 5 new event types added under "Phase 2 — State events"
- [x] Appendix I §IV.1 IdentityMessage: `is_ai` / `ai_capabilities` added to `register` and `record`; `update` notes 3041
- [x] Appendix I §V.1 IdentityRecord: 2 new fields added
- [x] Appendix I §V.x: new section for `AiCapabilities` struct
- [x] Appendix I §VI.1 SpaceState: 4 new fields added
- [x] Appendix I Part IX: 5 new content schemas added (IX.12–IX.16); IX.1 updated with optional fields
- [x] Appendix I: section for `TemperatureThresholds`; reserved `meta_atts` keys + `auto_temperature` reason value documented
- [x] Appendix D §2.1: `is_ai` + `ai_capabilities` added to Identity Records table
- [x] Appendix D §2.2: `membership.mute` noted alongside other membership events
- [x] Ch4 implementation: test count updated to 387, D-059/D-060/D-061 noted as implemented
- [x] Appendix F: sanity-checked, no edits required (or edits applied)
- [x] Appendix C: scope discussed; updates applied if scope agreed
- [x] Ch0 TOC: Appendix A and B titles corrected
- [x] All edited files: `Last updated` header date bumped
- [x] JOURNAL.md entry written documenting this cleanup pass (next J-NNN)
- [ ] Commit pushed; this file's status flipped to COMPLETED

---

## Notes for next-session Chat Claude

- Joe's preferences (from `userPreferences`): discuss before writing for substantial edits; explain approach first; suggest milestones. For mechanical table-row additions, just do them — Joe doesn't need a discussion gate for "add this row to this table".
- Use `Filesystem` MCP `edit_file` with `dryRun: true` to preview before committing. Verify each header `Last updated` date is bumped where required.
- Two trailing spaces on every `> ...` header line — mandatory.
- All four agent personas (this conversation's convention as of 2026-05-15): formal names in documents are **Chat Claude** / **Code Claude** / **Design Claude**. Informal nicks in conversation are Claude (Chat Claude) / Clair (Code Claude) / Ms Design (Design Claude). Don't retroactively rewrite historical journal entries.
- The findings above include line numbers from a particular point-in-time read; line numbers may shift slightly between sessions. Always re-grep by content (struct names, field names, section headers) rather than relying on line numbers.

---

*This file is the Chat Claude work item for cleaning up documentation drift after J-065. Mark Definition of Done items only when verified by reading back the edited file.*
