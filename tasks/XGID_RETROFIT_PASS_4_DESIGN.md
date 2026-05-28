# XGID Retrofit Pass 4 — Design Document
> **Status**: COMPLETED  
> Version: 1.2  
> Date: May 2026  
> **Last updated**: 2026-05-28 (v1.0 → v1.1 in-place at §3 + §4.1 walk-and-lock session opened post-J-139. §3 governing principle locked inherited-unchanged from Pass 2 §3 + Pass 3 §3 — four-instance Pass-arc inheritance established. §4.1 Surface #1 M5 Ops Layer locked: composite of §4.1.0 honest recon corrections (15 pub structs not 16; 46 String slots not 45; pre-guessed borderlines absent — discipline data point for Pass-5 + future Pass-arc recon expectations) + §4.1.a field retype scope (31 mechanical identifier retypes + 12 mechanical descriptive stays + 3 borderline locks: 2 NodeXgid for `home_node`/`node` + 1 String stay for `source` operator-source enum-tag) + §4.1.b Pass 1 additive-API extension Option β locked (inherent `.is_empty()` on flavour wrappers + Option `.as_deref()` per Pass 1 Commit 4 precedent over per-site rewrite) + §4.1.c serde-transparent wire-neutrality confirmed at Surface #1 boundary. §4.2 format-boundary preservation locked Option γ split — A (Surface #1 stdout) + B (Surface #3 pipe JSON) consolidate under Pass 3 wire-shape boundary class (no new instance count); C (Surface #4 Tauri IPC) recognised as fresh boundary class at Pass 4. D-NNN-format-boundary promotion-watch from J-138 Sub-section 8 STAYS OPEN — three structurally-distinct instances across two Pass-arc but Pass 4 boundary class is fresh-at-Pass-4 so D-077 multi-Pass-arc durability not yet met. Fourth structurally-distinct instance at Pass 5 OR cross-milestone closes durability gap. §4.3 CLI arg parsing boundary locked Option α (clap parse stays String; project at dispatcher arm via Pass 1 wrapper constructor chain; 16 identifier-shaped Args slots enumerated at walk-time). FromStr explicitly deferred per D-071 audit-design-impl-arc framing — validated FromStr is the rung above per D-079 honest-framing precedent. §4.4 doc-vs-code commit-shape pre-frame locked Option γ hybrid split (per-surface code commits carry their atomic doc fragments; cross-surface and content-shape doc surfaces consolidate at milestone-close per D-074) — runbook commit-sequence candidate 8-9 commits at next session-arc authoring inherits the lock. §4.5 async-spawned task captures sub-rule extension locked Option γ honest framing closure — D-NNN-ε promotion-watch CLOSED at honest framing per D-065 + D-079 (rule is Rust language idiom not XGen-specific decision; ubiquity confirms language-idiom framing; D-NNN slot preserved for actual XGen-specific decisions); Pass 3 §4.2 v1.2 third row sibling-shape rule table extended at canonical-design-doc layer. Cross-Pass discipline carry-over: Pass 3 §4.5 + Pass 4 §4.5 establish two-instance pattern of honest-framing-resolution of promotion-watches at Pass-arc design close. All five §4 anchors (§4.1 + §4.2 + §4.3 + §4.4 + §4.5) LOCKED at this walk. §5 layered-B3 expected answer LOCKED null at full eight-surface scope — four-instance Pass-arc no-finding chain established (Pass 1 + Pass 2 + Pass 3 + Pass 4 — all null). §6 historical/future-pointer entries filled in Shape α pointer-style per Pass 3 §6.1 precedent (§6.1 Pass 4 design phase historical record + §6.2 two-session walk shape precedent inheritance). §7 discipline notes consolidated five sub-sections (§7.1 honest recon corrections per Rule 5 + D-065; §7.2 format-boundary preservation Option γ split as honest-framing-resolution shape; §7.3 doc-vs-code commit-shape at design phase per D-069 + D-071; §7.4 two-instance pattern of honest-framing-resolution of promotion-watches at Pass-arc design close — promote/close-by-honest-framing/hold-open three shapes; §7.5 two-session walk shape Pass-internal precedent inheritance at second instance); §7.6 candidate skipped per minimal-broadening discipline (Pass 2 §7.2 already codifies inline-lock pattern). **Status flipped ACTIVE → COMPLETED at this design close commit.** §6.1 implementation milestone-close J-NNN placeholder frozen at runbook close per J-108 codification. v1.0 J-139 design-phase-open chain entry preserved below.)  
> Last updated (v1.0 J-139): 2026-05-28 (J-139 — Pass 4 design phase opened. §1 framing + §2 surface enumeration shipped at design-phase kickoff. Seven subsystems enumerated in dependency order: M5 Ops Layer (ops.rs Result structs + serde-transparent boundary; HIGHEST IMPACT — 16 struct types + 45 String slots); CLI Dispatcher (app.rs format + integration); Batch Pipe Dispatch (batch.rs get_dag_tips + space_id param + 3 slots); Tauri Shell (desktop.rs 3 commands + lifecycle state machine); Session State (session.rs + lifecycle.rs ClientIdentity); AI Resident (ai_service.rs + ai_behavior.rs AiBehavior trait + EchoPlugin); Pacing + Temperature (pacing.rs + temperature.rs HashMap keys + event payloads). §3 governing principle + §4 architectural decisions + §5 layered-B3 expected answer + §6 historical/future-pointer entries + §7 discipline-notes deferred to subsequent walk-and-lock sessions per Joe-lock at session open. Pre-walk reconnaissance via parallel Explore subagent recorded 192 xgen-client compilation errors per Path A inheritance from Pass 1+2+3 (three categories: Result struct field retypes; method availability on Xgid newtypes; HashMap key-value slot mismatches); zero forward-looking `// Pass 4` markers in workspace — three-instance sparsity chain at Pass-arc level now durable per N+1 design discipline; Appendix F (1193 lines) + xgen_aicontrol_implementation.md (544 lines) + Ch6 §6.15 (lines 1326-1388+) doc surfaces confirmed. Four-file atomic per D-074 (thirty-sixth instance) + Lock #3 per-commit cadence: this design doc NEW v1.0 + ROADMAP visual tree row + Past entry + Near future Pass 4 line update + CLAUDE.md PLAY block flip "Pass 3 milestone CLOSED" → "Pass 4 design phase ACTIVE; §1 + §2 shipped at v1.0" + JOURNAL J-139 body entry. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079.)  
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

## §3 Governing principle

### §3.1 Lock — inherited from Pass 2 §3 + Pass 3 §3 unchanged

**Locked at v1.1 walk-and-lock session post-J-139.** The governing principle for Pass 4 inherits verbatim from Pass 2 §3 (J-123) and Pass 3 §3 (J-127):

> Identifier slots retype to typed XGIDs; descriptive-string slots stay `String`; internal variables bind as typed references; `&str` projection happens at the call-site boundary via `Borrow<str>` (Pass 1's additive API at Commit 4 implementation-kickoff lock); no `Deref<Target = str>` shortcuts.

**Four-instance Pass-arc inheritance** at this lock (Pass 1 implicit at runbook authoring; Pass 2 explicit at J-123; Pass 3 explicit at J-127; Pass 4 explicit at this v1.1 walk). The governing principle's stability is durable at Pass-arc layer — three-instance threshold per D-077/D-078 promotion-framing was met at Pass 3; fourth instance at this lock confirms the principle as load-bearing cross-Pass discipline carry-over per JOURNAL J-138 Sub-section 2.

### §3.2 Sanity-check across the seven xgen-client surfaces + Surface #8

The §2 surface enumeration surfaced no per-surface wrinkle requiring §3 amendment. Per-surface inheritance recorded honestly for §4 walk anchoring:

| Surface | Inherits §3 unchanged? | Application surface |
|---|---|---|
| #1 M5 Ops Layer (ops.rs Result structs) | ✅ | Typed in-memory; serde-transparent newtypes serialise as plain String at CLI/batch/Tauri boundary. Application of §3 at §4.1 (Result-struct field retype) + §4.2 (format-boundary preservation extended). |
| #2 CLI Dispatcher (app.rs) | ✅ | Clap parse boundary at CLI arg entry is "call-site boundary" per §3; String → typed projection at parse. Application of §3 at §4.3. |
| #3 Batch Pipe Dispatch (batch.rs) | ✅ | JSON serde over named pipe — same format-boundary application as Surface #1. Inherited mechanism. |
| #4 Tauri Shell (desktop.rs) | ✅ | Tauri command return types cross IPC to JS/TS frontend as String via serde-transparent. Same format-boundary shape; no §3 amendment. |
| #5 Session State (session.rs + lifecycle.rs) | ✅ | Pure in-memory cache; no boundary; typed throughout. Cleanest §3 application surface in Pass 4. |
| #6 AI Resident (ai_service.rs + ai_behavior.rs) | ✅ | Tokio spawns surface D-NNN-ε async-spawn captures watch but as §4.5 application of Pass 3 §4.2 sibling-shape rule table, not §3 amendment. |
| #7 Pacing + Temperature (HashMap keys) | ✅ | `Borrow<str>` lookup mechanism explicitly named in §3 principle (Pass 1 Commit 4 additive API). Pass 3 Surface #4 fanout HashMap-key retype is structural sibling — inherited mechanically. |
| #8 Doc-tree sweep | ✅ | Documentation; §3 principle's typed-in-memory + String-on-format-boundary framing already documented in Appendix F + xgen_aicontrol_implementation.md + Ch6 §6.15. Pass 4 doc-pass annotates per-section without principle amendment. |

**Verdict**: all eight surfaces inherit §3 cleanly. No Pass-4-specific wrinkle surfaces at §3 layer.

### §3.3 Three wrinkle-candidates considered honestly per D-065

Recorded for §4 walk anchoring + future-Pass design-phase reference:

1. **Result-struct serde-transparent at Surface #1.** Whether the transparent-newtype-over-String pattern (used in xgen-common at Pass 1 Commit 1 + extended across Pass 2/3) constitutes a "format-boundary preservation" rule worth lifting into §3 rather than §4.2. **Deferred to §4 turf.** Pass 3 §4.3 v1.2 consolidated format-boundary (wire OR persistence) as a §4 application; Pass 4 extends the same §4 framing to client-side serialisation formats (Tauri IPC + pipe JSON + stdout). §3 governs the typing-discipline rule; §4.2 governs the structural mechanism by which the rule projects to format boundaries. Keeping the separation preserves Pass 2 + Pass 3 §3-vs-§4 layering.
2. **Async-spawn captures at Surface #4 + Surface #6.** Whether the "spawned tasks force owned parameters" Tokio-idiom-as-discipline rule (Pass 3 §4.2 v1.2 sibling-shape rule table third row, instantiated at Pass 3 Surface #5 `handle_federation_incoming` + Surface #6 reconnect.rs three spawned functions) surfaces a §3-level wrinkle if a structurally-different fifth instance fires at xgen-client async surfaces. **Deferred to §4 turf per D-NNN-ε promotion-watch framing from J-138 Sub-section 8.** §3 governs the typing discipline as identifier-slot retype rule; spawned-task ownership is a Rust-language `'static` constraint (per D-065 honest framing), not an XGen typing rule. §4.5 surfaces this as a sibling-shape rule table extension.
3. **Tauri IPC frontend (JS/TS) reads typed XGIDs as plain strings.** Whether the frontend boundary (Rust → JS via Tauri IPC + serde-transparent) changes the governing principle. **No wrinkle.** Same format-boundary application as wire/persistence surfaces walked at Pass 3 §4.3 v1.2; serde-transparent makes the Rust-side typed without changing JS-side shape. §4.2 picks this up as the third instance of the format-boundary preservation rule application surface (alongside wire IPC + persistence) — third-instance threshold for D-NNN-format-boundary promotion-watch from J-138 Sub-section 8 fires at §4.2 walk, not §3.

### §3.4 Cross-Pass discipline carry-over implication

§3 inherited unchanged at four instances confirms the governing principle is **load-bearing cross-Pass discipline carry-over** per JOURNAL J-138 Sub-section 2 framing. Future Pass-arc design-phase walks (if any post-Pass-5) inherit §3 without re-derivation; Pass-arc-internal §3-amendment-in-place precedent (Pass 3 §3 v1.1 → v1.2) remains available if a future surface-class surfaces a wrinkle the four-instance Pass-arc inheritance did not anticipate.

---

## §4 Architectural decisions

### §4.1 Surface #1 M5 Ops Layer — Result-struct field retype + Pass 1 additive-API extension + serde-transparent wire-neutrality

**Locked at v1.1 walk-and-lock session post-J-139.** Foundational decision per dependency order (Surface #1 consumed by §4.2 + §4.3 + §4.4). Composite of three sub-locks (§4.1.a — field retype scope; §4.1.b — Pass 1 additive-API extension for method availability; §4.1.c — serde-transparent boundary preservation).

#### §4.1.0 Honest recon corrections per Rule 5 + D-065

Three corrections to §2.10 recon data points surfaced at walk-time grep-vs-design reconciliation (sibling-shape to Pass 3 §6.7 J-127 Sub-section 8 data point (e)):

1. Recon claimed **16 pub Result struct types** in ops.rs. Actual: **13 Result structs + 2 non-Result pub structs** (`OpContext<'a>` for shared call context + `HistoryMessage` as `HistoryResult` row shape) = **15 pub structs total**. Recon over-counted by 1-3 depending on what's considered "Result-struct-shaped."
2. Recon claimed **~45 String slots**. Actual: **46 String slots** across the 13 Result structs + `HistoryMessage`. Off by 1.
3. Recon pre-guessed borderline candidates as `since` / `tip_event_id` / `pubkey_uri`. **None of those fields exist in ops.rs Result structs.** Borderline list surfaces differently at walk-time (§4.1.a.iii below).

Honest data points recorded per Rule 5 (test counts + file counts must come from actual command output) and D-065 (honest framing over polite framing) — recon is pre-walk reconnaissance under "very thorough" search-level guard-rails, not verbatim authoritative source. Pass 5 + future Pass-arc design-phase opens should expect single-digit drift between recon estimates and walk-time actuals; the discipline value of recon is shape-grounding, not verbatim accuracy.

#### §4.1.a Field retype scope (composite — three sub-locks across 46 String slots)

##### §4.1.a.i — Identifier-shaped slots: mechanical retype per §3 (31 slots, no sub-question)

| Field | Count | Retype |
|---|---|---|
| `identity_id` | ×4 (Whoami, Status, Register; ai_identity_id below) | `IdentityXgid` |
| `space_id` | ×9 (CreateSpace, CreateRoom, Invite, Join, Send, History, AiDelegate, AiRevoke, AiStatus) | `SpaceXgid` |
| `event_id` | ×7 (CreateSpace, CreateRoom, Invite, Join, Send, AiDelegate, AiRevoke) | `EventXgid` |
| `room_id: String` | ×3 (CreateRoom, Send, History) | `RoomXgid` |
| `room_id: Option<String>` | ×1 (Join) | `Option<RoomXgid>` |
| `target_identity` | ×1 (Invite) | `IdentityXgid` |
| `owner_identity_id` | ×1 (CreateSpace) | `IdentityXgid` |
| `ai_identity_id` | ×3 (AiDelegate, AiRevoke, AiStatus) | `IdentityXgid` |
| `new_operator` | ×1 (AiDelegate) | `IdentityXgid` |
| `owner_id` | ×1 (AiStatus) | `IdentityXgid` |
| `operator: Option<String>` | ×1 (AiStatus) | `Option<IdentityXgid>` |
| `ai_invited_by: Option<String>` | ×1 (AiStatus) | `Option<IdentityXgid>` |
| `sender` | ×1 (HistoryMessage) | `IdentityXgid` |

Total: **31 slots**, all mechanical per §3 governing principle. No per-slot sub-question.

##### §4.1.a.ii — Descriptive-string slots: mechanical stay per §3 (12 slots, no sub-question)

| Field | Count | Stays | Reasoning |
|---|---|---|---|
| `display_name` | ×3 (Whoami, Status, Register) | `String` | Human-readable name; not identifier. |
| `version` | ×1 (Status) | `String` | Version-string descriptor. |
| `name` | ×2 (CreateSpace, CreateRoom) | `String` | Space/Room display name. |
| `role` | ×1 (Invite) | `String` | Role-tag enum-like descriptor. |
| `registered_at` | ×1 (Register) | `String` | RFC3339 timestamp string per protocol. |
| `timestamp` | ×1 (HistoryMessage) | `String` | RFC3339 timestamp string. |
| `text` | ×1 (HistoryMessage) | `String` | Message body content (NOT identifier; descriptive payload). |
| `ai_member_role: Option<String>` | ×1 (AiStatus) | `Option<String>` | Role-tag descriptor sibling to `role`. |

Total: **12 slots**, all mechanical stay-as-String per §3 governing principle. No per-slot sub-question.

##### §4.1.a.iii — Borderline slots: 2 `NodeXgid` retype + 1 `String` stay (3 slots, locked)

| Field | Count | Lock | Reasoning |
|---|---|---|---|
| `home_node: String` | ×3 (Whoami, Status, Register) | **`NodeXgid`** | Per protocol §3.6.1, an Identity's home is a Node identifier (not transport URL). Pass 3 Surface #2 already retyped `node_id: NodeXgid` at xgen-node side; xgen-client side `home_node` is the same identifier flavour. Wire-format-neutral via serde-transparent. |
| `node: String` | ×1 (AiStatus) | **`NodeXgid`** | Same semantic as `home_node` — the Node hosting the AI Identity. Sibling-shape lock. |
| `source: Option<String>` | ×1 (AiStatus) | **stays `String`** | M3 fall-upward resolution tag — values are `"delegation"` / `"inviter"` / `"owner"` per D-064 lock. Enum-like tag descriptor, not identifier. Sibling-shape to §4.1.a.ii `role` + `ai_member_role` mechanical stay. (Could be lifted to a dedicated `OperatorSource` enum at a future hygiene pass; out-of-scope at Pass 4 per §2.9 honest broadening.) |

Total: **3 slots** Joe-locked. 2 retype to `NodeXgid`; 1 stays `String`.

**Slot count verification**: 31 (§4.1.a.i) + 12 (§4.1.a.ii) + 3 (§4.1.a.iii) = **46 slots**. Matches §4.1.0 corrected actual. No slots unaccounted for.

#### §4.1.b Pass 1 additive-API extension for method availability

Recon §2.10 data point 1 category (b): `.is_empty()` + `.as_deref()` method-availability errors on Xgid newtypes. Two structural options walked:

- **Option α — rewrite callers**: replace `xgid.is_empty()` with `xgid.as_str().is_empty()` at all sites. Per-site rewrite churn at every method call.
- **Option β — additive-API extension on flavour wrappers**: `impl XxxXgid { pub fn is_empty(&self) -> bool { self.as_str().is_empty() } }` + analogous `as_deref` for `Option<XxxXgid>` via Option-method extension. Sibling-shape to Pass 1 Commit 4 implementation-kickoff `Borrow<str>` additive-API lock (Pass 1 chose additive-API over per-call rewrite for `HashMap::get(&str)`).

**Locked**: **Option β** (additive-API extension) per Pass 1 precedent. Inherent `.is_empty()` on flavour wrappers + Option-method extension where call-sites use `Option<XxxXgid>::as_deref()` returning `Option<&str>`-equivalent. Discipline data point: Pass-arc additive-API extension is the load-bearing carry-over from Pass 1 Commit 4 — preserves call-site ergonomics without per-site rewrite churn; mirrors the cross-Pass discipline carry-over per JOURNAL J-138 Sub-section 2 framing.

**Surface scope**: Inherent methods added to xgen-common flavour wrappers (`IdentityXgid`, `SpaceXgid`, `EventXgid`, `RoomXgid`, `NodeXgid`, `TrustAssertionXgid` — six wrappers per Pass 1 Commit 1 set). Wire-format-neutral via no-op (inherent methods don't affect serde derive).

#### §4.1.c serde-transparent boundary preservation wire-format-neutrality

All flavour wrappers are `#[serde(transparent)]` per Pass 1 design. Result struct retypes at §4.1.a do not change JSON wire shape — the typed newtype serialises as the inner String, byte-identical to the pre-retype wire form. **Surface #1 Pass 4 retype is wire-format-neutral by mechanism.**

Format-boundary preservation as a general principle (wire OR persistence per Pass 3 §4.3 v1.2 consolidation) extends to client-side serialisation surfaces at §4.2 — Tauri IPC + pipe JSON + stdout per the §4 stub. §4.1.c is the wire-neutrality confirmation at Surface #1's serde derive boundary; §4.2 surfaces the cross-surface format-boundary application.

### §4.2 Format-boundary preservation extended to client-side serialisation surfaces — Option γ split (D-NNN-format-boundary promotion-watch stays OPEN)

**Locked at v1.1 walk-and-lock session post-J-139.** Composite lock across three Pass 4 candidate instances + promotion-threshold recount per D-077/D-078 surface-diversity framing.

#### §4.2.1 Pass 3 §4.3 v1.2 framing inherited

Format-boundary preservation rule (Pass 3 §4.3 v1.2 verbatim consolidation):

> At any byte-serialisation boundary (wire OR persistence), the format wire shape is plain string regardless of in-memory type. Typed newtypes in Rust; transparent serde derive (`#[serde(transparent)]`); same JSON on the wire.

Two Pass 3 instances catalogued at J-138 Sub-section 8:
- **Pass 3 Instance 1** — wire IPC at Surface #3 federation_session.rs handshake message format
- **Pass 3 Instance 2** — persistence at Surface #5 app.rs filesystem JSON + `replay_spaces_from_dir` + wire-message destructure

#### §4.2.2 Pass 4 candidate instance enumeration

Three Pass 4 candidates surface across xgen-client serialisation surfaces:

| Candidate | Surface | Format-boundary shape | Structural classification |
|---|---|---|---|
| **A** | Surface #1 — CLI stdout JSON | `serde_json::to_string_pretty(&result)` from `ops::*` dispatchers; output to terminal | Sibling-shape to Pass 3 wire IPC — JSON serde over byte stream (stdout instead of socket). **Same boundary class as Pass 3 Instance 1.** |
| **B** | Surface #3 — pipe JSON | Named-pipe IPC for `--batch` between client + spawned process per D-043 | Sibling-shape to Pass 3 wire IPC — JSON serde over named-pipe byte stream. **Same boundary class as Pass 3 Instance 1.** |
| **C** | Surface #4 — Tauri IPC | Tauri command return types serialised to JS/TS frontend via Tauri's serde bridge | **Structurally distinct** — not byte-stream wire or filesystem persistence; Rust↔JS process-internal IPC over Tauri runtime. **New boundary class at Pass 4.** |

#### §4.2.3 Promotion-watch outcome — Option γ split locked

Three options walked at this Q-anchor (full reasoning preserved per D-065 honest framing):

- **Option α — Promote to D-080 at Pass 4 design close** treating A + B + C as three structurally-different instances meeting D-077 threshold.
- **Option β — Hold promotion-watch open** treating A + B + C as a single "client-side serialisation surfaces" instance (consolidating with Pass 3 §4.3 v1.2 wire-OR-persistence consolidation precedent).
- **Option γ — Split**: consolidate A + B under existing Pass 3 wire-shape boundary class (no new count); recognise C as standalone "Tauri IPC bridge as serialisation surface" — one new structurally-distinct boundary class at Pass 4. Promotion-watch stays open at three structurally-distinct instances across two Pass-arc instances (Pass 3 wire-generalised + Pass 3 persistence + Pass 4 Tauri IPC). Fourth structurally-distinct instance at Pass 5 OR at cross-milestone (M6/M7 admin write path; possible future gRPC / WebRTC / HTTP API surfaces) closes the durability gap and promotes to D-080.

**Locked**: **Option γ**. Reasoning (four grounds):

1. **Surface-diversity discipline per D-077/D-078**: A (stdout) and B (pipe) are clearly wire-shape generalisations of Pass 3 Instance 1 — same boundary class, different transport. Calling them new instances would over-count and weaken the surface-diversity discipline that D-077 + D-078 promotion-threshold framings rely on.
2. **Tauri IPC (C) is genuinely structurally different**: Rust↔JS process-internal bridge over Tauri's serde marshalling is a third boundary class distinct from byte-stream wire and filesystem persistence. But one instance at one Pass-arc ≠ three-instance durable cross-Pass discipline per D-077.
3. **Pass 3 §4.3 v1.2 consolidation precedent supports**: consolidating wire OR persistence under one §4 rule already established that "instance count" means "structurally distinct boundary class," not "every code site." A + B don't introduce a new boundary class; C does.
4. **Honest framing per D-069 + D-065 + D-079**: promoting D-NNN-format-boundary at Pass 4 with three instances at the same Pass-arc would record a project decision based on insufficient cross-Pass durability evidence. Holding the watch open is the discipline data point Pass 5 + cross-milestone work can validate.

**§4.2 application surfaces at this lock**:

- **Surface #1 stdout** — Pass 3 Instance 1 wire-shape application; serde-transparent newtypes serialise as plain string. Confirmed wire-format-neutral at §4.1.c.
- **Surface #3 pipe JSON** — Pass 3 Instance 1 wire-shape application; same mechanism.
- **Surface #4 Tauri IPC** — new boundary class at Pass 4; Tauri command return types use serde-transparent newtypes; JS/TS frontend reads plain strings; **wire-format-neutral by same mechanism**, but boundary class is fresh.

**Promotion-watch state at this lock**:

| Instance | Pass-arc | Boundary class | Status |
|---|---|---|---|
| Pass 3 Instance 1 (wire-generalised) | Pass 3 | byte-stream serialisation | counted |
| Pass 3 Instance 2 | Pass 3 | filesystem persistence | counted |
| Pass 4 Instance C (Tauri IPC) | Pass 4 | Rust↔JS process-internal IPC | counted |
| **Total structurally-distinct instances** | | | **3** |
| **Pass-arc instance count for durability** | | | **2 (Pass 3 + Pass 4)** |
| **D-077 three-instance durability across multiple Pass-arc?** | | | **NO — Pass 4 boundary class fresh-at-Pass-4** |
| **D-NNN-format-boundary status** | | | **promotion-watch stays OPEN** |
| **Promotion trigger** | | | Fourth structurally-distinct instance at Pass 5 OR cross-milestone (M6/M7 + future surfaces) closes durability gap |

#### §4.2.4 Surface #2 CLI stdout — relationship to §4.3

Surface #2 (CLI dispatcher) reads CLI args at the parse boundary (§4.3 turf) and emits results to stdout at the format boundary (§4.2.2 Instance A). Two distinct boundaries at the same surface — separate Q-locks. §4.3 covers the parse-boundary direction; §4.2 covers the emit-boundary direction.

### §4.3 CLI arg parsing boundary — Option α (clap parse stays String; project at dispatcher arm)

**Locked at v1.1 walk-and-lock session post-J-139.** Surface #2 (CLI dispatcher, app.rs) parse boundary discipline.

#### §4.3.0 Scope — 16 identifier-shaped clap-Args slots enumerated

Walk-time enumeration of clap-derive Args structs surfaced **16 identifier-shaped String slots** at parse boundary:

| Args struct | Identifier slots | Target flavour |
|---|---|---|
| `AiDelegateArgs` | `space` + `ai` + `to` | SpaceXgid + IdentityXgid + IdentityXgid |
| `AiRevokeArgs` | `space` + `ai` | SpaceXgid + IdentityXgid |
| `AiStatusArgs` | `space` + `ai` | SpaceXgid + IdentityXgid |
| `CreateRoomArgs` | `space` | SpaceXgid |
| `InviteArgs` | `space` + `identity` | SpaceXgid + IdentityXgid |
| `JoinArgs` | `space` + `room: Option<String>` | SpaceXgid + Option<RoomXgid> |
| `SendArgs` | `space` + `room` | SpaceXgid + RoomXgid |
| `HistoryArgs` | `space` + `room` | SpaceXgid + RoomXgid |

Plus 5 descriptive slots that stay String per §3 (`name` ×3, `role`, `text`) + 4 transport/config slots at Cli top-level (`node` WS URL, `instance` label, `log_level`, `config` path) that stay String per §3.

#### §4.3.1 Three structural options walked

- **Option α** — clap parse stays `String`; project to typed XGID at dispatcher arm via existing Pass 1 wrapper constructor chain (`SpaceXgid::from_xgid(Xgid::new(args.space))`).
- **Option β** — add `FromStr` impl on flavour wrappers; clap parse types directly to `SpaceXgid` / etc.; parse-time error UX.
- **Option γ** — clap parse stays `String`; project at `ops::*` entry (functions take `&str`; wrap internally before lib calls).

#### §4.3.2 Option α locked — four grounds

1. **Pass 1 additive-API precedent**: flavour wrappers at xgen-common authored without `FromStr` intentionally per Pass 1 Commit 1 lock. Validation story is hash-anchored constructors (`from_canonical_bytes`, `from_pubkey`); ad-hoc `Xgid::new(String)` un-validated wrapper for known-good runtime strings is the call-site bridge. Adding `FromStr` at Pass 4 would either (a) wrap un-validated path (trivial, zero parse-time UX value) or (b) require Pass-4-scope agreement on "what constitutes a valid XGID string format at parse-time" — substantive protocol-level design surface out of Pass 4 scope.
2. **Pass 3 dispatcher-entry retype precedent**: Pass 3 Surface #2 retyped `dispatch_event(peer_node_id: Option<&str>)` → `Option<&NodeXgid>` at function-signature boundary, not at any upstream parse layer. Sibling-shape applies at xgen-client: typed at `ops::*` entry, projection at dispatcher arm.
3. **Honest scope-discipline per D-065 + §2.9 honest broadening**: Option β lifts Pass 4 scope into "design FromStr validation surface" — scope creep onto a substantive protocol-design question. Option γ pushes wrap-call further inward (worse layering — `ops::*` functions should receive typed inputs per §3 governing principle, not project internally; preserves §3-vs-§4 layering).
4. **Cheap projection at dispatcher arm**: 16 sites × ~1 line per projection (`let space = SpaceXgid::from_xgid(Xgid::new(args.space));` at dispatcher arm before `ops::create_room(ctx, space, …)`). No per-site rewrite churn beyond what's already required at the consumer boundary; consistent with Pass 1's "projection at boundary, not at every call" mechanic.

#### §4.3.3 Option α composite lock

- **§4.3.a** — Clap-derive Args structs keep all 16 identifier-shaped String slots as `String` at parse boundary. Wire-format-neutral (clap parses argv text; not a serde wire surface).
- **§4.3.b** — Dispatcher arm projects `String` → flavour wrapper via Pass 1 `Xgid::new(s) → XxxXgid::from_xgid(...)` constructor chain at call site to `ops::*`.
- **§4.3.c** — Pass 4 explicitly does NOT add `FromStr` to flavour wrappers. Deferred to future audit-design-impl arc per D-071 if protocol-level "valid XGID string format at parse-time" lock surfaces as a substantive Pass-arc design decision.
- **§4.3.d** — Honest framing per D-079 + D-065: validated `FromStr` is the rung above Option α (sibling-shape to Pass 3 §4.3 v1.2 rung-above-(a).iii.β framing where formal verification rungs were named above the locked solution). Deferring preserves optionality without compromising Pass 4 deliverable.

#### §4.3.4 Cross-reference to §4.1 and §4.2

§4.3 is the **parse-boundary direction** at Surface #2 (CLI input). §4.2.4 noted Surface #2 has a paired **emit-boundary direction** (stdout JSON at §4.2 Instance A consolidated under Pass 3 wire-shape boundary class). Both boundaries at the same surface; Option α at parse + serde-transparent at emit means Surface #2 is wire-format-neutral on both directions per §3 governing principle.

### §4.4 Doc-vs-code commit-shape decision — Option γ hybrid split (per-surface atomic + consolidated milestone-close)

**Locked at v1.1 walk-and-lock session post-J-139.** Runbook-shape pre-frame decision per D-069 + D-071 audit-precedes-dependent-design framing — doc-tree coupling shape locked at design phase, runbook commit-sequence is downstream consequence.

#### §4.4.0 Doc-tree sweep surface size

Surface #8 doc-tree sweep at Pass 4 covers three documentation surfaces totalling ~1800 lines (per §2.10 recon data point 5):

| Doc surface | Lines | Tight-coupling to code surface |
|---|---|---|
| `docs/xgen_appendix_f_en.md` | 1193 | Surface #1 (M5 ops layer) primary; Surface #2 + #3 secondary |
| `docs/xgen_aicontrol_implementation.md` | 544 | Surface #6 (AI resident) primary |
| `docs/xgen_ch6_client_design.md` §6.15 | ~60 (lines 1326-1388+) | Surface #7 (pacing + temperature) primary |

#### §4.4.1 Three structural options walked

- **Option α — Per-surface atomic-with-doc**: each code-surface commit carries the relevant doc-tree section updates in the same atomic commit. Atomic discipline at finest grain; high commit count; each commit has its own per-surface doc fragment.
- **Option β — Single consolidated doc-pass commit + per-surface code commits**: doc-tree sweep ships as a single dedicated commit (sibling to Pass 3 Commit 1 doc-pass shape per J-131) ahead of or after the code commits. Doc-pass commit is large but coherent; code commits are doc-free and focused.
- **Option γ — Hybrid split**: doc surfaces split by tight-coupling — per-section doc fragments tightly coupled to one code surface ship atomic with that code commit; doc surfaces that cross multiple code surfaces or are content-shape rather than per-surface ship in a consolidated final doc-pass commit.

#### §4.4.2 Option γ locked — three grounds

1. **Per-surface-coupled doc fragments ship atomic with their code surface**: changes to Appendix F that reflect ops.rs Result-struct retypes belong in the same commit as the ops.rs retype — atomic discipline preserves doc-code coherence at the per-surface boundary. Sibling-shape to Pass 3 §6.7 J-127 Sub-section 8 data point (b) "consolidation in same atom over split-into-sibling" reasoning extended to doc-with-code coupling per D-076 v1.0 → v1.1 amend-in-place precedent.
2. **Cross-surface or content-shape doc fragments ship in consolidated milestone-close commit**: high-level architecture intros + ROADMAP + CLAUDE PLAY flips + JOURNAL J-NNN entries are project-management surfaces consolidated at milestone-close per D-074 standard practice. Sibling-shape to Pass 3 Commit 3 milestone-close five-file atomic (J-138).
3. **Pass 3 Option β was light**: J-131 noted Pass 3 Commit 1 doc-pass was honestly two-file rather than three-file (post-strip-the-chain discipline). Pass 4 doc surface is ~30× larger than Pass 3's so single consolidated Option β would balloon to ~12-file doc-pass — heavier than is healthy for one commit. Option γ scales without compromising atomicity.

#### §4.4.3 Option γ composite lock

- **§4.4.a** — Per-surface doc-tree fragments ship atomic with their code surface commit:
  - Appendix F §F.0.6 + M5 ops section fragments → with Surface #1 code commit.
  - Appendix F CLI / batch sections → with Surface #2 + #3 code commits.
  - xgen_aicontrol_implementation.md AI resident + AiBehavior sections → with Surface #6 code commit.
  - Ch6 §6.15 pacing + temperature subsections → with Surface #7 code commit.
- **§4.4.b** — Cross-surface or content-shape doc fragments consolidated in milestone-close commit per D-074:
  - High-level architecture intros referencing the XGID retrofit at protocol layer (if any survive Pass 1-3 doc passes).
  - ROADMAP version bump + visual tree row flip + Past entry.
  - CLAUDE PLAY block flip Pass 4 implementation → Pass 4 milestone CLOSED + Pass 5 implementation next.
  - JOURNAL J-NNN milestone-close body entry.
  - Design doc + runbook Status flips ACTIVE → COMPLETED.
- **§4.4.c** — Runbook §3 commit sequence pre-frame: Pass 4 runbook authoring at next session-arc anticipates a heavier-than-Pass-3 sequence — candidate shape is **per-surface code commit + atomic doc fragment** × 7 surfaces + Commit 2a test-fixture sweep contingent split (sibling-shape to Pass 3 §4a contingent-split posture pre-locked) + milestone-close commit. Total candidate commit count: 8-9 (heavier than Pass 3's 4 commits but lighter than the trilogy's ~12-commit pattern per Pass-internal-consistency framing).
- **§4.4.d** — Discipline data point recorded for future Pass-arc design phase per D-079: doc-vs-code split shape decision belongs at design phase (not runbook authoring phase) per D-069 + D-071 audit-precedes-dependent-design framing — runbook commit-sequence is a downstream consequence of doc-tree coupling shape; locking it earlier prevents runbook-authoring drift at Pass 5 + future Pass-arc design phases.

#### §4.4.4 Pre-frame implication for Pass 4 runbook authoring

Pass 4 runbook authoring at next session-arc (post-design-close) inherits Option γ as the commit-sequence default. Runbook §3 commit-sequence framing draws per-surface atomic-with-doc as Commits 2-8 candidate shape (8-9 commits total expected); Commit 1 doc-pass shape from Pass 3 precedent does NOT apply at Pass 4 since per-surface doc fragments ship atomic with code (no need for a separate Commit 1 doc-pass beyond ROADMAP + CLAUDE PLAY + JOURNAL bumps which consolidate at milestone close per §4.4.b). Honest discipline data point: runbook author at next session-arc should verify this pre-frame against Pass 4 doc-tree walking-its-own-content reconciliation (sibling-shape to Pass 3 J-133+J-134+J-135 triple-canonical-record-amendment arc that surfaced at runbook walking its own content layer).

### §4.5 Async-spawned task captures sub-rule extension at Surface #4 + #6 — Option γ honest framing closure (D-NNN-ε promotion-watch CLOSED)

**Locked at v1.1 walk-and-lock session post-J-139.** Pass 3 J-138 Sub-section 8 promotion-watch resolved by honest framing per D-065 + D-079.

#### §4.5.0 Pass 4 xgen-client async surface enumeration

Walk-time grep of xgen-client/src/ (production scope, excluding stress-test harness spawns at app.rs in S0-S5 scenario blocks) surfaced two structurally-different async boundary classes at Pass 4:

| Surface | Async sites | Boundary class |
|---|---|---|
| **Surface #4 Tauri Shell** (desktop.rs) | 3 `#[tauri::command]` handlers (lines 54, 63, 90) | Rust↔JS Tauri IPC dispatch — runtime spawns tasks for command handlers |
| **Surface #6 AI Resident** (ai_service.rs + service.rs) | 4 `tokio::spawn` sites (ai_service.rs:554+575; service.rs:183+202) | Long-running headless resident loop spawns |

Total Pass 4 instances: **7 async-spawn sites across 2 structurally-different boundary classes**.

#### §4.5.1 J-138 Sub-section 8 honest framing inherited

Pass 3 J-138 Sub-section 8 locked the honest position on D-NNN-ε candidate per D-065:

> D-NNN-ε (async-spawned task captures force owned parameters — Tokio idiom) — four instances at one xgen-node module-family. Three instances at one module-family is weaker durability evidence than three across structurally different surfaces (per D-077 + D-078 surface-diversity framing). Per D-065 honest framing: the rule is a Rust language idiom (`'static` bound on `tokio::spawn`), not a XGen-specific call. Promotion would record a language fact rather than a project decision. Promotion-watch opens at Pass 4 surfacing structurally different fourth instance at xgen-client async surfaces.

#### §4.5.2 Three options walked

- **Option α** — Promote D-NNN-ε to D-080 at Pass 4 design close: combined instance count is now 6 (Pass 3) + 7 (Pass 4) = 13 across 3+ structurally-different surfaces (xgen-node federation_session + reconnect + Tauri commands + AI resident). D-077 surface-diversity threshold structurally met.
- **Option β** — Hold promotion-watch open at Pass 4 close; wait for Pass 5 or cross-milestone instance.
- **Option γ** — Honest framing closure: extend Pass 3 §4.2 v1.2 third row sibling-shape rule table to record Pass 4 instances; close the promotion-watch by honest framing per D-065 + D-079 — rule is ubiquitous Rust language idiom, not XGen-specific decision; D-NNN slot preserved for actual XGen-specific decisions.

#### §4.5.3 Option γ locked — four grounds

1. **J-138 Sub-section 8 honest framing pre-answers the substantive question**: ubiquity strengthens the "Rust idiom" framing rather than promoting to project decision. The threshold being structurally met at Pass 4 doesn't change the rule's nature.
2. **Pass 4 surface diversity confirms ubiquity**: 2 new structurally-different boundary classes (Tauri command spawns + AI resident long-running spawns) demonstrate the rule applies everywhere async-spawn happens at xgen-client — that's the definition of a language idiom, not a project decision. The rule's stability comes from `'static` bound on `tokio::spawn`, not from XGen-specific design.
3. **Pass 3 §4.2 v1.2 sibling-shape rule table is the right canonical record**: extending the rule table to cover Pass 4 instances achieves the documentation goal at the right layer (canonical-design-doc rule table extension), not at DECISIONS.md. Sibling-shape to D-079 honest-framing precedent (record-at-canonical-document-not-D-NNN-if-rule-is-mechanical).
4. **D-NNN slot preserved for XGen-specific decisions**: future structurally-different format-boundary surface at Pass 5 / M6 / M7 (per §4.2 D-NNN-format-boundary promotion-watch) would be a genuine XGen project decision worth a D-NNN. Keeping D-080 (or similar) slot for that promotion serves the canonical-record discipline per D-079 better than spending it on a Rust idiom record.

#### §4.5.4 Option γ composite lock

- **§4.5.a** — Surface #4 Tauri commands (3 sites) + Surface #6 AI service spawns (4 sites) confirm async-spawned-task-captures sub-rule as **ubiquitous Rust language idiom** across xgen-{node, client} async surfaces.
- **§4.5.b** — Pass 3 §4.2 v1.2 third row sibling-shape rule table **extended at canonical design-doc layer** to record Pass 4 instances; **D-NNN-ε promotion-watch closed by honest framing** per D-065 (rule is Rust language idiom `'static` bound on `tokio::spawn`, not XGen-specific decision).
- **§4.5.c** — Discipline data point recorded per D-079: ubiquity confirms language-idiom framing, doesn't trigger D-NNN promotion. Pass 5 + future async surfaces inherit closed-watch state without re-derivation.
- **§4.5.d** — **D-NNN slot preserved** for XGen-specific decisions; this closure precedent (close-by-honest-framing) instantiates D-079 at promotion-watch-closure layer (vs D-079's original promotion-by-honest-framing instantiation at Pass 3 J-134).

#### §4.5.5 Cross-Pass discipline carry-over implication

Pass 3 + Pass 4 establish a **two-instance pattern of honest-framing-resolution of promotion-watches at Pass-arc design close**:

- Pass 3 §4.5 J-127 — D-NNN-γ (small-cardinality vs large-cardinality identifier-keyed maps per-Pass call-site density) — held open with two instances per D-069.
- Pass 4 §4.5 this lock — D-NNN-ε — closed by honest framing per D-065 + D-079.

Both are honest-framing operations at promotion-watch boundary; the second instance establishes the pattern that promotion-watch close-by-honest-framing is a valid discipline action alongside promotion-by-honest-framing (Pass 3 J-134's D-079 promotion atom). Pass 5 + future Pass-arc design phase walks inherit both shapes as load-bearing canonical-record-discipline operations.

---

## §5 Layered-B3 expected answer — null at full eight-surface scope (four-instance Pass-arc no-finding chain established)

**Locked at v1.1 walk-and-lock session post-J-139.** Layered-B3 expected null **confirmed at full eight-surface scope at Pass 4**.

### §5.1 Three-instance Pass-arc no-finding chain inherited

Pass 1 J-122 + Pass 2 J-126 + Pass 3 J-138 all returned zero layered surfaces at milestone close. Three-instance chain durable per D-077/D-078 promotion-threshold framing. Mechanism inherited verbatim from Pass 3 §5.5:

> Pass-arc work whose scope is data-structure-or-function-signature shape (not algorithm validation) naturally avoids the layered-B3 surface; the `Borrow<str>` projection mechanism (Pass 1 Commit 4 implementation-kickoff additive-API lock) handles type-projection at call-site boundaries uniformly without forcing secondary encodings of the same invariant across all retyped functions.

### §5.2 Per-surface audit at design phase (Rule 5 + D-065 honest)

| Surface | Layered-B3 audit | Reasoning |
|---|---|---|
| #1 M5 Ops Layer (ops.rs) | ✅ null | Result structs are flat data carriers; serde-transparent at format boundary per §4.1.c; no secondary encoding of the typing invariant. |
| #2 CLI Dispatcher (app.rs) | ✅ null | Clap parse boundary + projection at dispatcher arm per §4.3 Option α; no secondary validation surface. Sibling-shape to Pass 3 Surface #2 dispatch_event signature retype which closed at null. |
| #3 Batch Pipe Dispatch (batch.rs) | ✅ null | JSON serde over named pipe — same mechanism as wire (§4.2 Instance B consolidated under Pass 3 wire-shape boundary class); no second surface. |
| #4 Tauri Shell (desktop.rs) | ✅ null | Tauri commands return serde-transparent types; JS/TS frontend sees plain strings; no secondary encoding. §4.2 Instance C fresh boundary class is structurally distinct boundary, not layered-B3 surface (different audit dimension). |
| #5 Session State (session.rs + lifecycle.rs) | ✅ null | Pure in-memory cache; no validation surface; cleanest §3 application surface per §3.2 sanity-check table. |
| #6 AI Resident (ai_service.rs + ai_behavior.rs) | ✅ null | Spawned tasks own typed XGIDs per §4.5 Option γ; same mechanism as Pass 3 Surface #5/#6 reconnect.rs + federation_session.rs spawned functions which closed at null. |
| #7 Pacing + Temperature (pacing.rs + temperature.rs) | ✅ null | HashMap keys use `Borrow<str>` projection from Pass 1 Commit 4 additive-API; sibling-shape to Pass 3 Surface #4 fanout.rs HashMap-key retype which closed at null. |
| #8 Doc-tree sweep | ✅ null | Documentation; no algorithm-validation surface by construction. |

### §5.3 Four-instance Pass-arc no-finding chain established

Pass 4 confirms the layered-B3 expected-null posture at fourth instance:

- Pass 1 J-122 — five-surface scope (xgen-common + xgen-core data-structure retypes) closed null.
- Pass 2 J-126 — five-surface scope (xgen-core algorithm-bearing function retypes) closed null.
- Pass 3 J-138 — seven-surface scope (xgen-node + Appendix D + per-space HashMap keys) closed null.
- Pass 4 J-140 — eight-surface scope (seven xgen-client subsystems + Surface #8 doc-tree) **expected null at design phase**; verification at runbook §6.5 audit by Clair at implementation boundary per Rule 0 + Rule 5 honest-audit-not-honest-assumption discipline.

Four-instance durability strengthens the structural finding: **Pass-arc layered-B3 expected-null is load-bearing cross-Pass discipline carry-over** per JOURNAL J-138 Sub-section 2 framing. Pass 5 + future Pass-arc design phases inherit the expected-null posture without re-derivation.

### §5.4 Discipline data point — design-phase audit vs runbook-implementation-phase audit

§5 audit at design phase is **expectation-grounding** per §5.2 per-surface table — Chat Claude's read of the eight surfaces against the layered-B3 finding-shape. Runbook §6.5 + DoD verification at Pass 4 Commit 2 + milestone-close commit boundaries require Clair to **re-run** the audit honestly per Rule 0 + Rule 5 + D-065 framing — not take design-phase expectation on faith.

Sibling-shape to Pass 3 design doc §5.5 + runbook §6.5 split: design phase locks expectation; runbook verifies at implementation boundary. If implementation-phase audit surfaces a layered-B3 finding that design phase did not anticipate, that is a Pass-arc-internal Joe-lock surface (sibling-shape to topo-sort Commit 2a Option E unification per D-067 + the J-101 first-instance layered-B3 close).

**Pass 5 inheritance**: Pass 5 design phase opens with expected-null at Pass 5 scope per four-instance chain durability; Pass 5 runbook authoring inherits the §6.5 audit-at-implementation-boundary discipline without re-derivation.

---

## §6 Historical / future-pointer entries

### §6.1 Pass 4 design phase historical record (Shape α pointer-style)

Pointer-style entry per Pass 3 §6.1 + Pass 2 §6.7 precedent. Implementation J-NNN milestone-close placeholder frozen at runbook close per J-108 codification.

- **Design open**: J-139 (2026-05-28). Four-file atomic per D-074 thirty-sixth instance. Design doc v1.0 ships §1 framing + §1.2 precedent-positioning + §1.3 NOT scope + §2 surface enumeration via parallel Explore subagent reconnaissance under "very thorough" search level + no-file-modification guard-rail (sibling-shape to Pass 3 Commit 2a parallel-subagent discipline at runbook §9.7 but at design-phase open layer rather than test-fixture sweep). Seven xgen-client subsystems + Surface #8 doc-tree enumerated in dependency order.
- **Design close (single-session full close)**: J-140 (2026-05-28). Four-file atomic per D-074 thirty-seventh instance. Design doc v1.0 → v1.2 + Status ACTIVE → COMPLETED. §3 governing principle locked inherited unchanged from Pass 2 + Pass 3 — four-instance Pass-arc inheritance established. All five §4 anchors locked: §4.1 Surface #1 M5 Ops Layer composite (§4.1.0 honest recon corrections + §4.1.a 46-slot classification + §4.1.b Pass 1 additive-API Option β + §4.1.c serde-transparent wire-neutrality); §4.2 format-boundary preservation Option γ split (D-NNN-format-boundary promotion-watch STAYS OPEN); §4.3 CLI arg parsing Option α (clap stays String); §4.4 doc-vs-code commit-shape Option γ hybrid split (runbook commit-sequence pre-framed at 8-9 commits); §4.5 async-spawned task captures Option γ honest framing closure (D-NNN-ε promotion-watch CLOSED). §5 layered-B3 confirmed null at full eight-surface scope — four-instance Pass-arc no-finding chain established. §6 (this entry) Shape α historical-pointer. §7 discipline notes consolidate five Pass-4-specific data points. **Two-session split was eligible per Pass 3 J-127 Sub-section 8 data point (c) but not exercised** at Pass 4 per "let us move ahead" mid-session pivot from Option II pause to Option I continue (recorded honestly per D-065 + §6.2 + §7.5).
- **Implementation runbook authoring**: opens in fresh session post-design-close per Pass 2 J-124 + Pass 3 J-128 design-then-runbook precedent. Runbook §3 commit-sequence inherits §4.4 Option γ hybrid-split pre-frame: per-surface code+doc atomic × 7 + Commit 2a test-fixture sweep contingent split (sibling-shape to Pass 3 §4a contingent-split posture pre-locked) + milestone-close commit → 8-9 commits expected total.
- **Implementation milestone close**: J-NNN [TO BE FROZEN AT MILESTONE CLOSE per J-108 codification]. Pass 4 milestone close commit lands when all per-surface code+doc commits + Commit 2a contingent + milestone-close commit ship.

### §6.2 Two-session walk shape — eligible-but-not-exercised at Pass 4 per honest framing

Pass 2 design close at J-123 fit single session; Pass 3 spanned two same-day sessions per J-127 Sub-section 8 data point (c); **Pass 4 was eligible for two-session split (8 surfaces > 5 threshold) but landed as single same-day session** at J-139 design-open + J-140 design close per "we are good in session capacity, let us move ahead" continuation directive after §3 + §4 walk-and-lock first segment.

**Honest framing per D-065**: precedent eligibility ≠ precedent exercise. Pass 4 surfaces 8 boundaries (7 code + 1 doc-tree) which structurally meets the Pass 3 J-127 Sub-section 8 data point (c) threshold for two-session split as deliberate scaffolding shape. Session-capacity at the §3 + §4 walk-and-lock close was sufficient to continue, so the split was not exercised. Future Pass-arc design phases may exercise or not exercise the split per same-session-capacity assessment without violating the precedent — the precedent names two-session split as **available scaffolding shape**, not mandatory.

Cross-reference to §7.5 for discipline implication.

---

## §7 Discipline notes

Pass-4-specific discipline data points surfaced across §3 + §4 + §6 walks. Five sub-sections per Pass 3 §7 + Pass 2 §7 precedent shape; §7.6 candidate (Joe-locks-by-recommendation as inline-lock pattern fifth recurrence) skipped per minimal-broadening discipline (Pass 3 §7.10 sibling-shape — codification at Pass 2 §7.2 already covers the pattern; Pass 4 §7.6 would over-document a Pass-arc-stable discipline).

### §7.1 Honest recon corrections per Rule 5 + D-065

Pre-walk reconnaissance delivered via parallel Explore subagent at J-139 design-open under "very thorough" search level + no-file-modification guard-rail. At §4.1.0 walk three corrections surfaced honestly per Rule 5:

1. Recon claimed 16 pub Result struct types in ops.rs — actual 13 Result + 2 non-Result = 15 pub structs.
2. Recon claimed ~45 String slots — actual 46.
3. Recon pre-guessed borderline candidates `since` / `tip_event_id` / `pubkey_uri` — none of those fields exist in ops.rs Result structs; actual borderlines surface differently at walk time (`home_node` ×3 + `node` + `source: Option<String>`).

**Discipline data point for Pass 5 + future Pass-arc design-phase opens**: expect single-digit drift between recon estimates and walk-time actuals. The value of pre-walk reconnaissance under parallel-subagent delegation is **shape-grounding** (boundary class identification + structural category enumeration + size estimation), **not verbatim accuracy**. Pass-arc design walks should re-ground numeric claims at walk-time via grep against the production source rather than recon-as-authoritative.

Sibling-shape to Pass 3 J-127 Sub-section 8 data point (e) "structural calls emerged at walk time from grep-vs-design-anticipation reconciliation" — confirms the pattern at Pass-arc layer. Reconnaissance shape-grounds; walk-time grep verifies. Pass-arc precedent now durable at second instance.

### §7.2 Format-boundary preservation Option γ split as honest-framing-resolution shape

§4.2 walked three options for the D-NNN-format-boundary promotion-watch resolution from J-138 Sub-section 8:

- Option α — promote D-080 treating Pass 4 instances A + B + C as three structurally-different instances.
- Option β — hold open treating Pass 4 surfaces as single consolidated instance.
- Option γ — split: consolidate A + B under Pass 3 wire-shape boundary class (no new count) + recognise C as fresh boundary class at Pass 4.

Option γ locked on four grounds: (1) surface-diversity discipline per D-077/D-078 — A and B are wire-shape generalisations; (2) Tauri IPC (C) is genuinely structurally different — but one instance at one Pass-arc ≠ three-instance durable cross-Pass discipline; (3) Pass 3 §4.3 v1.2 consolidation precedent — "instance count" means "structurally distinct boundary class"; (4) honest framing per D-069 + D-065 + D-079 — promoting at three same-Pass-arc instances would record decision based on insufficient cross-Pass durability.

**Discipline data point**: **honest-framing-resolution of promotion-watches at Pass-arc design close is itself a discipline action shape**. The promotion-watch boundary admits three resolution shapes — promote (Pass 3 J-134 D-079), close-by-honest-framing (Pass 4 §4.5 D-NNN-ε), or hold-open-by-surface-diversity-threshold (Pass 4 §4.2 D-NNN-format-boundary). All three are valid; selection requires honest assessment of instance count + structural diversity + Pass-arc durability against D-077/D-078 framing.

Promotion-watch state recorded transparently in canonical record at §4.2.3 promotion-watch state table — future Pass-arc design phases inherit the table shape for subsequent watch resolutions.

### §7.3 Doc-vs-code commit-shape decision at design phase (not runbook authoring phase)

§4.4 locked Option γ hybrid split: per-surface-coupled doc fragments ship atomic with code surface commit; cross-surface or content-shape doc fragments consolidated at milestone-close commit per D-074. Runbook commit-sequence pre-frame: 8-9 commits expected at next session-arc runbook authoring.

**Discipline data point**: doc-vs-code split shape decision belongs at design phase, not at runbook authoring phase, per D-069 + D-071 audit-precedes-dependent-design framing. Runbook commit-sequence is downstream consequence of doc-tree coupling shape; locking earlier prevents runbook-authoring drift at Pass 5 + future Pass-arc design phases.

Sibling-shape to D-071 own-arc discipline (audit → design → impl) applied at runbook layer: commit-sequence shape is a design-phase deliverable that the runbook inherits, not a runbook-authoring-time decision. Pass 5 design phase opens with doc-vs-code commit-shape as a §4-anchor candidate; if Pass 5 doc surface is similar in shape to Pass 4 (per-surface-coupled fragments dominant), the Option γ pre-frame at Pass 4 inherits cleanly.

### §7.4 Two-instance pattern of honest-framing-resolution of promotion-watches at Pass-arc design close

Two instances at this Pass 4 design close pair with Pass 3 design close at J-127:

- Pass 3 §4.5 J-127 — D-NNN-γ (small-cardinality vs large-cardinality identifier-keyed maps per-Pass call-site density) — held open with two instances per D-069.
- Pass 4 §4.5 J-140 this lock — D-NNN-ε (async-spawned task captures force owned parameters — Tokio idiom) — closed by honest framing per D-065 + D-079.

Plus the promotion-by-honest-framing precedent from Pass 3 J-134:

- Pass 3 J-134 — D-NNN-κ (Design-doc Q-table grounded by symbol-definition grep) — promoted to D-079 by honest framing after three-instance threshold met across two distinct catch-events.

**Discipline data point**: **honest-framing-resolution at promotion-watch boundary admits three shapes** — promote (D-079 promotion atom precedent), close-by-honest-framing (D-NNN-ε at this Pass 4 §4.5 lock), hold-open-by-surface-diversity-threshold (D-NNN-format-boundary at this Pass 4 §4.2 lock).

Both close-by-honest-framing and hold-open shapes preserve the D-NNN slot for actual XGen-specific decisions. The discipline value is: not every Pattern-X candidate becomes a D-NNN. D-NNN promotion is reserved for project decisions that survive both surface-diversity threshold AND honest framing as XGen-specific (not language idiom or mechanical structural fact). D-079 + sibling close-by-honest-framing shapes together codify the discipline.

Pass 5 + future Pass-arc design phase walks inherit all three shapes as load-bearing canonical-record-discipline operations.

### §7.5 Two-session walk shape — eligible-but-not-exercised at Pass 4 + revised discipline framing

Pass 2 design close at J-123 fit single session; Pass 3 spanned two same-day sessions per J-127 Sub-section 8 data point (c) foundational-first (#1-#4) + top-of-stack-second (#5-#7) split; **Pass 4 was eligible for two-session split (8 surfaces > 5 threshold) but landed as single same-day session** per "let us move ahead" continuation directive after §3 + §4 walk-and-lock close (mid-session pivot from Option II pause to Option I continue).

**Discipline data point — revised framing**: future Pass-arc design walks with > 5 surfaces should treat two-session split as **available scaffolding shape**, NOT mandatory. The Pass 3 J-127 Sub-section 8 data point (c) framing of "deliberate scaffolding" stands at availability layer; exercise depends on session-capacity assessment at the §3 + §4 walk-and-lock close boundary.

**Honest framing per D-065**: Pass 4 J-140 was originally scoped as the first session of a two-session split (§3 + §4 walk-and-lock close as the J-140 atomic; §5 + §6 + §7 + design close commit at J-141 next session). Mid-session "we are good in session capacity, let us move ahead" directive shifted scope to single-session full design close — J-140 absorbs the full design close. Recorded honestly here so future Pass-arc authors understand: precedent eligibility names options, session-capacity at the boundary chooses among them.

**Pass-arc precedent state**: single-session shape at Pass 2 + Pass 4 (J-123 + J-140); two-session shape at Pass 3 (J-127). Pattern is **bimodal across the Pass-arc trilogy** — neither shape is durable as default at three-instance threshold per D-077/D-078. Pass 5 design phase author makes the call at session boundary per same-session-capacity assessment.

**Cross-reference to design doc §6.2**: §6.2 records the Pass-internal-precedent inheritance + Pass 4 honest framing at canonical-historical layer; §7.5 records the discipline implication at canonical-discipline layer. Sibling-shape to D-079 promotion atom's two-layer recording (canonical-record + DECISIONS.md).

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
