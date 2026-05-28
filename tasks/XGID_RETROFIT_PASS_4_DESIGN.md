# XGID Retrofit Pass 4 — Design Document
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-28 (J-139 — Pass 4 design phase opened. §1 framing + §2 surface enumeration shipped at design-phase kickoff. Seven subsystems enumerated in dependency order: M5 Ops Layer (ops.rs Result structs + serde-transparent boundary; HIGHEST IMPACT — 16 struct types + 45 String slots); CLI Dispatcher (app.rs format + integration); Batch Pipe Dispatch (batch.rs get_dag_tips + space_id param + 3 slots); Tauri Shell (desktop.rs 3 commands + lifecycle state machine); Session State (session.rs + lifecycle.rs ClientIdentity); AI Resident (ai_service.rs + ai_behavior.rs AiBehavior trait + EchoPlugin); Pacing + Temperature (pacing.rs + temperature.rs HashMap keys + event payloads). §3 governing principle + §4 architectural decisions + §5 layered-B3 expected answer + §6 historical/future-pointer entries + §7 discipline-notes deferred to subsequent walk-and-lock sessions per Joe-lock at session open. Pre-walk reconnaissance via parallel Explore subagent recorded 192 xgen-client compilation errors per Path A inheritance from Pass 1+2+3 (three categories: Result struct field retypes; method availability on Xgid newtypes; HashMap key-value slot mismatches); zero forward-looking `// Pass 4` markers in workspace — three-instance sparsity chain at Pass-arc level now durable per N+1 design discipline; Appendix F (1193 lines) + xgen_aicontrol_implementation.md (544 lines) + Ch6 §6.15 (lines 1326-1388+) doc surfaces confirmed. Four-file atomic per D-074 (thirty-sixth instance) + Lock #3 per-commit cadence: this design doc NEW v1.0 + ROADMAP visual tree row + Past entry + Near future Pass 4 line update + CLAUDE.md PLAY block flip "Pass 3 milestone CLOSED" → "Pass 4 design phase ACTIVE; §1 + §2 shipped at v1.0" + JOURNAL J-139 body entry. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Framing

### §1.1 What this document is

The Pass 4 design closure for the XGID Retrofit milestone series. Pass 1 closed at J-122 (2026-05-26) with the core data structures in `xgen-common` + `xgen-core` retyped to typed XGID flavours. Pass 2 closed at J-126 (2026-05-27) with the xgen-core algorithm-bearing functions retyped (`validate_event`, `dispatch_event`, `PendingBuffer` arrival hooks, registry APIs, `accept_message`). Pass 3 closed at J-138 (2026-05-28) with the xgen-node binary surface retyped (federation_session, fanout, app handlers, reconnect scheduler, six per-space HashMap keys at NodeRuntime, Appendix D doc retypes). Pass 4 retypes the **xgen-client binary consumer surfaces** that close the Path A inherited break state across xgen-client + the **heaviest doc-work pass** across Appendix F (CLI reference + batch reply schemas), `xgen_aicontrol_implementation.md` (AI Control protocol), and Ch6 §6.15 (AI Client spec).

This is **Phase A** of Pass 4's audit → design → runbook → implementation → close arc. Like Pass 2 and Pass 3, the audit phase is absorbed into prior reconnaissance: a pre-walk Explore subagent at design-phase open enumerated 42 String identifier slots across 7 subsystems + 192 xgen-client compilation errors per Path A inheritance + zero Pass 4 forward-looking markers + three doc surfaces (Appendix F 1193 lines + `xgen_aicontrol_implementation.md` 544 lines + Ch6 §6.15 lines 1326-1388+). Per D-065 honest framing, this design doc opens directly at design with the question set pre-surfaced via reconnaissance.

### §1.2 Precedent-positioning relative to Pass 3

Pass 3's design doc was ~75 KB (locked at v1.2 J-127 across two same-day sessions). Pass 4 v1.0 ships at design-phase kickoff with §1 + §2 surface enumeration; §3-§7 deferred to subsequent walk-and-lock sessions per Joe-lock at session open. Final v1.x size depends on Q-table walk outcomes but expected mid-band sibling-shape to Pass 3 (~50-80 KB) per Pass-internal-consistency framing inherited from Pass 2 §7.7 + Pass 3 §7.2.

Three precedent-positioning notes:

1. **Pass 4 is the doc-heavy Pass.** Per the J-095 XGID Adoption v1 Phase 2 doc-tree sweep classification at `tasks/XGID_DOC_SWEEP.md`: Pass 4 has zero new classification-table rows but substantial per-section work in two already-pointer-tagged docs (Appendix F 890 → 1193 lines after Pass 1-3 work + xgen_aicontrol_implementation.md 372 → 544 lines after M4-M5 work). Plus Ch6 §6.15 spec annotation. **Pass 4's runbook should anticipate it is the heaviest doc-work pass despite no new classification-table rows.** This is flagged here for runbook authoring downstream.

2. **Zero Pass 4 forward-looking markers in workspace.** Per pre-walk recon: `// Pass 4 widens` + `// Pass 4 retypes` + `Pass 4` (case-sensitive) grep across all four crates returned ZERO matches. **Three-instance sparsity chain at Pass-arc layer now durable**: Pass 1 → Pass 2 had 33 inline markers (per J-125 audit); Pass 2 → Pass 3 had only 1 marker (xgen-core/src/node/runtime.rs:588); Pass 3 → Pass 4 has zero markers. Pattern per N+1 not N+2 design discipline (recorded at J-136 Sub-section 7 data point A); now established at three instances meeting D-077/D-078 promotion-threshold framing. Pass 4 design + runbook authoring cannot rely on pre-walk marker scaffolding from Pass 3; surface enumeration is independent (this document does that).

3. **Path A inherited break state at 192 errors all xgen-client.** Pass 1 + Pass 2 + Pass 3 deliberately broke xgen-client compilation per Path A discipline; xgen-client builds at workspace level for the first time when Pass 4 + Pass 5 close. The 192 errors organise into three structural categories (per recon §3): Result struct field type-mismatches (ops.rs returning typed XGIDs against Result fields still declared String); method availability on Xgid newtypes (calls to `.is_empty()`, `.as_deref()` against newtypes without those methods); HashMap/collection key-value slot mismatches (pacing.rs + temperature.rs expecting String keys against retyped sources). The Path A discipline is now three-instance-durable per J-138 Pass 3 close cross-Pass discipline carry-overs.

### §1.3 What this document is NOT

- **Not a re-audit of Pass 1 / Pass 2 / Pass 3 retypes.** Pass 1+2+3 COMPLETED locks stand authoritative.
- **Not the Pass 5 test-fixture / trace-field / Debug-Display sweep.** Trace event fields, Display impls, Debug impls in xgen-client handlers defer to Pass 5 per honest-broadening (named explicitly at the per-surface walks in §2 where applicable). Pass 5 also restores `cargo build --workspace` to GREEN.
- **Not a runbook.** The implementation sequencing, commit shape, Joe-lock checkpoints, and verification rigour live in `tasks/XGID_RETROFIT_PASS_4_IMPL.md` authored at runbook phase per Pass 1 + Pass 2 + Pass 3 precedent.
- **Not the AI Control protocol redesign.** The M7 design phase (which AI Control v1 spec at `xgen_aicontrol_implementation.md` ultimately resolves) is independent of Pass 4. Pass 4 annotates the existing spec with typed XGID slot callouts; it does not reshape the protocol.
- **Not the M6 (new) Node admin write path work.** That milestone is unblocked but unselected per Joe at J-138 close; Pass 4 does not touch M6 (new) scope.

---

## §2 Surface enumeration (dependency order)

Seven subsystems walked in dependency order. Surfaces enumerated at v1.0 with structural framing + foundational position + initial Q-anchors. Full Q-tables locked in subsequent walk-and-lock sessions per the v1.0 → v1.1 → v1.2 amendment pattern used at Pass 3.

**Dependency rationale:**
- Surface #1 (Ops Layer) is the foundational xgen-client retype: its Result structs are the serde-transparent boundary that every dispatcher (CLI, batch, Tauri) consumes. Retyping the Result struct field types first means downstream dispatcher retypes consume already-typed Result types.
- Surface #2 (CLI Dispatcher) consumes Surface #1 Result types + adds CLI-specific formatting + arg-parsing identifier material.
- Surface #3 (Batch Pipe Dispatch) consumes Surface #1 + carries the canonical `get_dag_tips` impl + pipe transport boundary.
- Surface #4 (Tauri Shell) consumes Surface #1 + adds Tauri-emit boundary for UI; lifecycle state machine identifier slots.
- Surface #5 (Session State) is consumed by all four prior surfaces; structurally narrow but foundational for ops:: ↔ dispatcher interaction.
- Surface #6 (AI Resident) consumes Surfaces #1 + #5 + adds AI-mode-specific plugin trait + EchoPlugin reference impl; M4 scope.
- Surface #7 (Pacing + Temperature) is the per-(space, sender) HashMap surface + event payloads emitted to UI; structurally similar to Surface #4 fanout in Pass 3 but at xgen-client scope.

Plus three doc surfaces (Appendix F + xgen_aicontrol_implementation.md + Ch6 §6.15) consolidated into Surface #8 doc-tree sweep.

### §2.1 Surface #1 — M5 Ops Layer Result structs (xgen-client/src/ops.rs)

**What this is.** The shared M5 ops layer at `xgen-client/src/ops.rs` (1260 LOC), introduced at J-078 Multiparty M5 milestone close as the "one canonical function per verb, three dispatchers (CLI arm, batch arm, Tauri arm) all thin shims" architectural lock per D-067 no-drift-surface discipline. Sixteen pub Result struct types carry approximately 45 String slots covering: `identity_id`, `display_name`, `home_node`, `space_id`, `event_id`, `owner_identity_id`, `room_id`, `ai_identity_id`, `sender_identity_id`, plus derivative slots in nested payloads. Pure data types; serde-transparent boundary for all dispatcher outputs.

**Foundational position.** Every dispatcher (CLI in app.rs format paths, batch in batch.rs pipe sink, Tauri in desktop.rs invoke return) consumes these Result types as the canonical data shape. Retyping Result struct field types first means downstream dispatcher retypes consume already-typed Result types; retyping dispatchers first against still-String Result types would generate sweep churn at the Result-type retype landing.

**Wire-format vs in-memory split (load-bearing finding from recon).** Result struct serde-derived encoding hits multiple wire formats: stdout-display by CLI (string-projection at format time); batch-reply JSON by pipe sink (canonical batch format per Appendix F); Tauri-IPC by Tauri command return (Tauri's bincode-or-JSON convention). The format-boundary preservation principle (Pass 3 §4.3 consolidated v1.2 — wire OR persistence) applies here: serde-derived output retains String format on the wire; typed XGID newtypes serialise transparently per `#[serde(transparent)]` discipline already in place at Pass 1.

**Initial Q-anchors (full Q-table locked at subsequent walk):**

- **Q1.1** — What retypes? All 16 pub Result struct types' identifier String fields → typed XGID flavour. Estimated 45 field-type retypes per recon.
- **Q1.2** — Serde transparency boundary holds? Pass 1's serde-transparent newtype design ensures Result struct serialisation format does NOT change on the wire. Test: round-trip JSON through pre-Pass-4 vs post-Pass-4 Result types must produce byte-identical output (wire-format invariance witness per Appendix J §J.5 invariance 2). Pre-Pass-4 client compat: pre-Pass-4 batch consumers see identical wire-format.
- **Q1.3** — OpContext String slots? `OpContext` struct carries identity material; identifier slots retype.
- **Q1.4** — Module-internal projections? Per-verb function bodies bind identifier variables; internal projection via `Borrow<str>` inherited from Pass 1.
- **Q1.5** — Test-fixture sweep impact? `ops.rs` has no in-tree test mod; tests live in xgen-client/tests/. Path A inherited state means recompilation surfaces against retyped Result types at fixture sweep boundary.
- **Q1.6** — What defers to Pass 5? Debug/Display impls on Result types (formatter audit at Pass 5); trace field formatter audit.

**Pass-4-specific finding worth recording for §3 governing principle walk:** Surface #1 instantiates the **dispatcher-fanout-from-canonical-data-shape pattern** — one Result struct fans out to three dispatcher outputs. Pass 3 design doc §4.5 ClientSenders + FederationPeerSenders xgen-node-internal-only call surfaces this conceptually; Pass 4 Surface #1 is the consumer side. Possible candidate D-NNN: "canonical-data-shape vs format-boundary projection at multi-dispatcher fanout" — flagged-not-promoted per D-069 (one instance at this design walk; promotion-watch opens at Pass 5 if a sibling fires at fan-in-to-canonical-data-shape).

### §2.2 Surface #2 — CLI Dispatcher (xgen-client/src/app.rs)

**What this is.** The xgen-client CLI entry-point dispatcher at `xgen-client/src/app.rs` (5255 LOC) hosting: Cli arg parsing via clap derive; subcommand routing for 13 verbs (init, register, status, connections, peers, spaces, identity, whoami, create-space, create-room, send, history, version); result formatting for stdout; keypair/config loading; CLI integration material for ops:: layer + session.rs state. Recon §1 reports 42 String slot occurrences across app.rs.

**Consumes Surface #1.** Every subcommand handler calls ops::* and consumes the returned Result struct; format paths project identifier-typed fields back to &str for stdout formatting.

**Format-boundary at stdout.** CLI output is plain-text stdout — display-time projection happens at format!() / println!() sites; identifier values project via Display impl on Xgid (inherited from Pass 1 — Xgid Deref<Target = String> projection per Pass 1 D-073 framing). Pass 4 format paths consume Display calls without modification; the underlying String construction at format-site uses Display rather than `.to_string()` to preserve flavour discipline.

**Wire-format vs in-memory split.** No wire format at CLI dispatcher — stdout is byte stream + plain-text. CLI arg parsing (Cli derive) parses String from argv per clap convention; arg values retype to typed XGID at the boundary entry where the value enters ops:: layer (boundary projection inherited from Pass 1).

**Initial Q-anchors (full Q-table locked at subsequent walk):**

- **Q2.1** — CLI arg shapes from clap parse: stay String at clap-derive boundary; project to typed XGID at ops:: layer entry. Or retype at clap-derive boundary directly via `FromStr` impl on flavour wrappers? Joe-lock at design walk.
- **Q2.2** — Result formatting paths consume typed Result struct fields via Display projection.
- **Q2.3** — Identity material loaded from disk (keypair, identity_id from xgen-client_state.json): retype at load-time entry point.
- **Q2.4** — Configuration loading: which config fields are identifier-typed vs descriptive?
- **Q2.5** — Tauri-integration sub-paths in app.rs? Recon notes some Tauri-spawn paths in app.rs; surface enumerated at Surface #4.

### §2.3 Surface #3 — Batch Pipe Dispatch (xgen-client/src/batch.rs)

**What this is.** The batch-mode pipe dispatcher at `xgen-client/src/batch.rs` (814 LOC) implementing the D-043 Windows named-pipe IPC server + per-verb pipe-arm dispatchers. Hosts the canonical `get_dag_tips` impl that all three xgen-client dispatchers (CLI, batch, Tauri future) consume per D-067 no-drift-surface lock at J-078 (M5 ops:: refactor). Recon §1 reports 3 String slots (space_id at `get_dag_tips` param).

**Consumes Surface #1.** Batch verb dispatchers call ops::* and serialise the Result struct to the pipe sink per Appendix F batch reply format.

**Wire-format at pipe boundary.** Batch replies serialise as JSON per Appendix F batch reply schema. Pass 1's serde-transparent newtype discipline preserves wire-format identity (typed XGID newtypes serialise as plain String). Wire-format invariance witness: a pre-Pass-4 batch client consuming a post-Pass-4 batch reply sees byte-identical output.

**Initial Q-anchors:**

- **Q3.1** — `get_dag_tips(space_id: String)` parameter: retype to `SpaceXgid`? At pipe-side dispatch entry, the pipe boundary receives String (JSON-decoded payload); projection to typed XGID at the dispatch entry boundary.
- **Q3.2** — Batch reply schema annotation in Appendix F: needs typed-XGID-in-memory + String-on-wire note per §4.3 format-boundary preservation. Folded into Surface #8 doc-tree sweep.
- **Q3.3** — Pipe protocol error handling: error replies carry identifier material via Result rejection paths; format-boundary stays String.

### §2.4 Surface #4 — Tauri Shell (xgen-client/src/desktop.rs)

**What this is.** The Tauri UI shell at `xgen-client/src/desktop.rs` (241 LOC) hosting: lifecycle state machine (11 client lifecycle states per Appendix E); systray integration; 3 Tauri commands at line numbers 54 (`get_state`), 63 (`get_pacing_state`), 90 (Tauri emit surface for state changes). Recon §1 reports 0 direct String slots in desktop.rs but identifier material flows via Tauri command return types from session.rs + ops::*.

**Consumes Surfaces #1 + #5.** Tauri commands return ops::* Result types serialised to JS frontend; lifecycle state events emit ClientStateEvent payloads from lifecycle.rs.

**Wire-format at Tauri IPC boundary.** Tauri IPC serialises via JSON-or-bincode per Tauri convention. Same format-boundary preservation as Surface #1: serde-transparent newtypes preserve wire format; UI consumes plain-String identifier values.

**Initial Q-anchors:**

- **Q4.1** — Tauri command signatures: `get_state` returns ClientStateEvent → identifier slots retype via lifecycle.rs Surface #5.
- **Q4.2** — Tauri emit surface for ClientStateEvent: serialisation format preserves String wire identity per Pass 1 serde-transparent.
- **Q4.3** — Lifecycle state machine String fields: identifier slots in state-tracking structures retype; descriptive slots (state names, transitions) stay String per D-073 field-name-vs-type discipline.
- **Q4.4** — Pipe server in desktop.rs (Node-style pipe for Tauri lifecycle): wire-format boundary stays String.

### §2.5 Surface #5 — Session State (xgen-client/src/session.rs + lifecycle.rs)

**What this is.** The per-invocation session state cache at `xgen-client/src/session.rs` (172 LOC) introduced at J-078 M5 milestone — ClientIdentity + SessionState + idempotent `ensure_identity` / `ensure_connected` helpers + M7-shape extension fields (bindings + spaces present-but-empty). Plus lifecycle state events at `xgen-client/src/lifecycle.rs` carrying ClientStateEvent payloads to Tauri emit surface.

**Foundational position consumed by all four prior surfaces.** Surfaces #1 (ops::* OpContext consumes session), #2 (CLI dispatcher loads session), #3 (batch dispatcher session-resumes per invocation), #4 (Tauri emit session lifecycle events) all consume Surface #5.

**Initial Q-anchors:**

- **Q5.1** — ClientIdentity struct: `identity_id` + `home_node` fields retype to typed XGID.
- **Q5.2** — SessionState struct: identifier slot retype + M7-shape extension field types (bindings map, spaces cache) — descriptive-vs-identifier classification per slot.
- **Q5.3** — Lifecycle state event payloads at lifecycle.rs: ClientStateEvent identifier slots retype; state-name + transition-label fields stay String.
- **Q5.4** — On-disk persistence at xgen-client_state.json: serde-transparent preserves wire format; on-disk JSON shape unchanged.
- **Q5.5** — Idempotent ensure_* helpers: parameter signatures retype.

### §2.6 Surface #6 — AI Resident (xgen-client/src/ai_service.rs + ai_behavior.rs)

**What this is.** The AI Client resident mode introduced at M4 (J-077) — `xgen-client --ai-mode --service` runs a long-running headless resident consuming inbound events through an `AiBehavior` plugin trait at `xgen-client/src/ai_behavior.rs` (305 LOC, 10 in-tree #[test] functions) + emitting replies under existing pacing + mute constraints via `xgen-client/src/ai_service.rs` (661 LOC, 8 in-tree #[test] functions). Plus EchoPlugin reference impl per D-065 honest-behaviour-over-polite-behaviour.

**Consumes Surfaces #1 + #5.** AI resident wraps ops::* calls + session state.

**AI-specific identifier surface.** AI Identity Extension at D-059 (Ch3 §3.6.10) introduces `ai_identity_id` slot at protocol layer; threads through ai_service + ai_behavior + ops:: layer + back to UI. Pacing + temperature scope per Surface #7.

**Initial Q-anchors:**

- **Q6.1** — AiBehavior trait method signatures: `on_event` / `propose_reply` etc. identifier slots in param/return types.
- **Q6.2** — AiPacingTracker per-Space pacing key: HashMap<SpaceXgid, _> per D-060 (Ch3 §3.7.12).
- **Q6.3** — EchoPlugin reference impl deterministic reply format: `ai_identity_id` + `sender_identity_id` slots in reply path retype.
- **Q6.4** — AI mode `__HEALTH__` extension `operator_known=N/M` identifier accounting: retype.
- **Q6.5** — D-059 / D-060 / D-061 protocol-spec annotations: Ch6 §6.15 documentation surface (folded into Surface #8).

### §2.7 Surface #7 — Pacing + Temperature (xgen-client/src/pacing.rs + temperature.rs)

**What this is.** The per-(space, sender) outbound message pacing module per D-060 Ch3 §3.7.12 at `xgen-client/src/pacing.rs` (2 String slots: space_id + sender_identity_id) + the temperature event payload per D-061 Ch3 §3.7.13 at `xgen-client/src/temperature.rs` (2 String slots: space_id + room_id + subject_id). HashMap-key surface at xgen-client scope; sibling-shape to Pass 3 Surface #4 fanout.rs ClientSenders + FederationPeerSenders HashMap-key retype.

**Consumes Surface #5 (session state) + emits to Surface #4 (Tauri UI via emit).**

**Initial Q-anchors:**

- **Q7.1** — pacing.rs HashMap<(String, String), _> key composite (space_id, sender_identity_id): retype to (SpaceXgid, IdentityXgid).
- **Q7.2** — temperature.rs event payload struct: identifier slots retype; descriptive slots (temperature value, subject_id classification) — subject_id stays String per D-061 spec or retypes per general principle?
- **Q7.3** — Sibling-shape to Pass 3 Surface #4 fanout HashMap-key retype: same Borrow<str> projection mechanism inherited; Pass-arc consistency framing.

### §2.8 Surface #8 — Doc-tree sweep (Appendix F + xgen_aicontrol_implementation.md + Ch6 §6.15)

**What this is.** The heaviest doc-work pass per J-095 XGID Adoption v1 Phase 2 doc-tree sweep classification. Three doc surfaces:

- **`docs/xgen_appendix_f_en.md`** (1193 lines, ACTIVE v1.3, last-updated 2026-05-20) — CLI Reference + Usage Examples + batch reply schema. Pre-walk recon notes: header note already flags "retype of identifier-carrying fields and batch reply schemas to XGID flavour types pending Retrofit Pass 4 per ROADMAP.md". Surface #8 closes this annotation arc.
- **`docs/xgen_aicontrol_implementation.md`** (544 lines) — AI Control protocol spec from M4 J-077. Per-section annotation for typed XGID slots; M7's future protocol redesign session is independent (Pass 4 annotates existing v1; M7 reshapes).
- **`docs/xgen_ch6_client_design.md`** §6.15 (lines 1326-1388+) — AI Client (resident mode) spec covering Mode selection + Configuration + AiBehavior trait + EchoPlugin reference. Per-section typed-XGID slot callouts.

**Foundational position.** Doc surfaces are atomic with their corresponding code surfaces per Pass 3 Surface #7 Appendix D Q7.5 minimal-touch precedent. Surface #8 doc edits ship in the same commit as the code surfaces they annotate, NOT as a standalone doc-only commit.

**Initial Q-anchors:**

- **Q8.1** — Appendix F per-section annotation strategy: classification-table-row Pass 4 marker (sibling-shape to Pass 3 Surface #7 Appendix D pattern) vs inline §-by-§ annotation?
- **Q8.2** — xgen_aicontrol_implementation.md: M7-future-redesign demarcation — current annotations are Pass 4 typed-XGID scope, not Pass 4 doing M7's job.
- **Q8.3** — Ch6 §6.15 spec annotation: AI Client resident mode + AiBehavior trait + EchoPlugin reference impl get typed XGID slot callouts.
- **Q8.4** — Coordination with code surfaces: each doc surface ships in the same atomic commit as its code surface (Surface #1 → Appendix F batch reply schema; Surface #6 → §6.15 + xgen_aicontrol_implementation.md; etc.) OR consolidated into a Surface-#8-only doc-pass commit?

### §2.9 Out-of-scope enumeration (honest broadening per D-065)

Six surfaces deliberately out-of-scope at Pass 4:

1. **Pass 5 test-fixture sweep across xgen-client.** Test files at `xgen-client/src/**/*.rs` with `#[cfg(test)]` (ai_behavior.rs 10 tests + ai_service.rs 8 tests = 18 in-tree tests minimum) + `xgen-client/tests/` 4 files defer to Pass 5 test-fixture-sweep per Pass-arc precedent (Pass 1 Commit 4a + Pass 2 Commit 2a + Pass 3 Commit 2a precedents). Pass 4 leaves test-fixture errors at Pass 4 Commit 2 lib-clean boundary; Pass 5 closes both Pass 4 + Pass 5 test-fixture sweep + `cargo build --workspace` restoration.
2. **Pass 5 trace-field formatter audit across xgen-client.** Pass 5 audits all `tracing::` invocations at xgen-client for typed-XGID Display projection vs raw String literal formatting.
3. **Pass 5 Debug + Display impl audit on xgen-client public types.** Pass 5 enumerates all `Debug` / `Display` impls on xgen-client public types and locks projection discipline.
4. **M7 (--aicontrol v1 covering both binaries).** AI Control protocol redesign is M7 scope, not Pass 4. Pass 4 annotates existing M4 v1 spec.
5. **M6 (new) Node admin write path.** Unblocked-but-unselected per J-138 close; not Pass 4 scope.
6. **D-071 future-removal arc for `validate_steps_8_13` + `accept_event`.** Pass 2 §4.2 Q5.b deprecation attributes remain pending; surface-driven per D-071.

### §2.10 Pre-walk reconnaissance summary (recorded for §3 governing principle walk)

Recon delivered via parallel Explore subagent at design-phase open under "very thorough" search level. Five honest data points worth recording for §3 + §4 walks:

1. **192 xgen-client compilation errors per Path A.** Three categories: (a) Result struct field type-mismatches (ops.rs typed returns vs String-declared fields); (b) method availability on Xgid newtypes (`.is_empty()` + `.as_deref()` on newtypes); (c) HashMap key-value slot mismatches (pacing.rs + temperature.rs typed sources vs String-declared maps). Substantive Surface #1 + Surface #7 work expected at impl phase to close these.
2. **Zero `// Pass 4 widens` markers in production.** Three-instance sparsity chain at Pass-arc level (Pass 1 → 33; Pass 2 → 1; Pass 3 → 0) — N+1 design discipline durable.
3. **42 String identifier slots across 7 subsystems.** Highest density: ops.rs (45 occurrences carrying ~20+ identifier-shaped slots in 16 pub Result struct types); app.rs (42 occurrences across CLI dispatch + integration). Lowest density: desktop.rs (0 direct String slots; identifier material flows via Tauri command return types).
4. **Test infrastructure at xgen-client.** 4 test files in xgen-client/tests/ (common shared utilities, not xgen-client-specific integration tests) + 18+ in-tree #[test] functions (ai_behavior.rs 10 + ai_service.rs 8). Tests won't compile at Pass 4 boundary per Path A; Pass 4 Commit 2a (test-fixture sweep) addresses fixture-level errors; Pass 5 close restores workspace.
5. **Doc surfaces sized.** Appendix F 1193 lines + xgen_aicontrol_implementation.md 544 lines + Ch6 §6.15 lines 1326-1388+. Heaviest doc-work pass; doc-vs-code split shape decision deferred to §4 architectural walk.

---

## §3 Governing principle (deferred to subsequent walk-and-lock session)

To be walked at next session. Expected outcome per Pass 2 §3 + Pass 3 §3 precedent: governing principle inherited from Pass 2 unchanged across Pass 4 surfaces (identifier slots retype to typed XGIDs; descriptive-string slots stay String; internal variables bind as typed references; &str projection happens at the call-site boundary via Borrow<str>; no Deref<Target = str> shortcuts). If a Pass-4-specific wrinkle surfaces at §3 walk, recorded amendment-in-place per Pass 3 §3 v1.1 → v1.2 precedent.

---

## §4 Architectural decisions (deferred to subsequent walk-and-lock session)

To be walked at next session. Initial candidates surfaced at §2:

- **§4.1** Result struct field type retype + serde-transparent boundary preservation (Surface #1 + Surface #8 Q1.2 + Q8.1 coordination).
- **§4.2** Format-boundary preservation extended to Tauri IPC + pipe IPC + stdout (Pass 3 §4.3 wire OR persistence consolidated; Pass 4 extends to client-side serialisation formats per D-NNN-format-boundary promotion-watch from J-138 Sub-section 8 — third-instance threshold opens here if Pass 4 client-side serialisation slot instantiates).
- **§4.3** CLI arg parsing boundary (Q2.1 — Cli derive clap parse stays String vs FromStr on flavour wrappers).
- **§4.4** Doc-vs-code commit-shape decision (Q8.4 — atomic doc-with-code vs consolidated Surface-#8-only doc-pass commit).
- **§4.5** Async-spawned task captures sub-rule extension if xgen-client async surfaces (Tauri commands, AI service spawns, batch dispatcher workers) instantiate D-NNN-ε structurally-different fifth instance per J-138 Sub-section 8 promotion-watch.

---

## §5 Layered-B3 expected answer (deferred to subsequent walk-and-lock session)

Initial expectation per three-instance no-finding chain durability (Pass 1 J-122 + Pass 2 J-126 + Pass 3 J-138 all zero layered surfaces): **expected null** at Pass 4 scope. Pass-arc work whose scope is data-structure-or-function-signature shape (not algorithm validation) naturally avoids the layered-B3 surface. Runbook §6.5 + this design doc §5.5 (when authored) require Clair to perform the layered-B3 audit at Commit 2 verification per honesty-over-assumption discipline.

---

## §6 Historical / future-pointer entries (deferred to subsequent walk-and-lock session)

To be filled at design close per Pass 3 §6.1 Shape α precedent.

---

## §7 Discipline notes (deferred to subsequent walk-and-lock session)

To be authored at design close per Pass 3 §7 + Pass 2 §7 precedent.

---

## §8 Cross-references

- **Pass 1 close**: J-122 (2026-05-26).
- **Pass 2 close**: J-126 (2026-05-27).
- **Pass 3 close**: J-138 (2026-05-28).
- **Pre-walk reconnaissance**: J-139 (this design open) parallel Explore subagent recon at design-phase kickoff.
- **Pass 3 design doc**: `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` v1.4 COMPLETED at J-127.
- **Pass 3 runbook**: `tasks/XGID_RETROFIT_PASS_3_IMPL.md` v1.6 COMPLETED at J-138.
- **J-095 doc-tree sweep classification**: `tasks/XGID_DOC_SWEEP.md` v1.2 COMPLETED.
- **Appendix F**: `docs/xgen_appendix_f_en.md` ACTIVE v1.3.
- **xgen_aicontrol_implementation.md**: `docs/xgen_aicontrol_implementation.md`.
- **Ch6**: `docs/xgen_ch6_client_design.md` ACTIVE v0.3.
- **DECISIONS.md**: D-065 + D-067 + D-069 + D-071 + D-073 + D-074 + D-076 v1.1 + D-077 + D-078 + D-079.
- **JOURNAL J-138** Sub-section 2 cross-Pass discipline carry-overs + Sub-section 8 four candidate D-NNNs promotion-watch (γ + δ + ε + format-boundary).

Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079.
