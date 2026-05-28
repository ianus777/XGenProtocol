# XGID Retrofit Pass 4 — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-28  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Framing

### §1.1 What this runbook is

This runbook is Clair's complete pickup specification for XGID Retrofit Pass 4 implementation. It is the authoritative entry-point file for Clair's session at Pass 4 implementation kickoff.

Pass 4's scope: retype the **seven xgen-client subsystem surfaces** locked at design doc §2 + §4 + the **Surface #8 doc-tree sweep** (Appendix F + xgen_aicontrol_implementation.md + Ch6 §6.15) per the design doc §4.4 Option γ hybrid-split pre-frame — per-surface-coupled doc fragments ship atomic with their code surface commit; cross-surface and content-shape doc fragments consolidate at milestone close. Design doc COMPLETED v1.2 at J-140 is the canonical anchor (`tasks/XGID_RETROFIT_PASS_4_DESIGN.md`).

The seven code surfaces in dependency order per design doc §2:

1. **Surface #1** — M5 Ops Layer Result structs at `xgen-client/src/ops.rs` (design doc §2.1 + §4.1 — 46-slot classification + Pass 1 additive-API extension Option β + serde-transparent wire-neutrality)
2. **Surface #2** — CLI Dispatcher at `xgen-client/src/app.rs` (design doc §2.2 + §4.3 — Option α clap parse stays String + project at dispatcher arm; 16 identifier-shaped Args slots)
3. **Surface #3** — Batch Pipe Dispatch at `xgen-client/src/batch.rs` (design doc §2.3 + §4.2 Instance B — pipe JSON consolidated under Pass 3 wire-shape boundary class)
4. **Surface #4** — Tauri Shell at `xgen-client/src/desktop.rs` (design doc §2.4 + §4.2 Instance C — Tauri IPC fresh boundary class at Pass 4)
5. **Surface #5** — Session State at `xgen-client/src/session.rs` + `lifecycle.rs` (design doc §2.5)
6. **Surface #6** — AI Resident at `xgen-client/src/ai_service.rs` + `ai_behavior.rs` (design doc §2.6 + §4.5 — async-spawn captures Option γ honest framing closure)
7. **Surface #7** — Pacing + Temperature at `xgen-client/src/pacing.rs` + `temperature.rs` (design doc §2.7 — sibling-shape to Pass 3 Surface #4 fanout HashMap-key retype)

Plus **Surface #8** doc-tree sweep distributed atomic with code surfaces per §4.4.a:
- `docs/xgen_appendix_f_en.md` §F.0.6 + M5 ops sections → with Surface #1 commit
- `docs/xgen_appendix_f_en.md` CLI / batch sections → with Surface #2 + #3 commits
- `docs/xgen_aicontrol_implementation.md` AI resident + AiBehavior sections → with Surface #6 commit
- `docs/xgen_ch6_client_design.md` §6.15 pacing + temperature subsections → with Surface #7 commit

### §1.2 Precedent-departure self-defense (sibling-shape to Pass 3 §1.2)

This runbook lands at ~50-70 KB target candidate, mid-band per Pass-internal-consistency framing inherited from Pass 2 §7.7 + Pass 3 §7.2. Three structural differences from Pass 3's runbook shape:

- **Eight surfaces vs Pass 3's seven** — one additional surface (Surface #8 doc-tree sweep distributed atomic with code).
- **Per-surface commits vs Pass 3's seven-surface atomic** — design doc §4.4 Option γ hybrid-split locks per-surface-coupled doc fragments ship atomic with their code surface commit. Result: Commits 1-7 are per-surface atomic (each commit ships code + tightly-coupled doc fragment + per-surface tests in-tree), Commit 7a [CONTINGENT] test-fixture sweep, Commit 8 milestone close = **8-9 commits total expected** vs Pass 3's 4.
- **No Commit 1 doc-pass** per design doc §4.4.4 — per-surface doc fragments ship atomic with code, ROADMAP + CLAUDE PLAY + JOURNAL bumps consolidate at milestone close per §4.4.b. The J-141 runbook-shipping commit IS the kickoff atomic (this commit set). Commit 1 starts at Surface #1 code+doc atomic.

Pass-internal-consistency framing per design doc §7.7 + JOURNAL J-138 Sub-section 2 cross-Pass discipline carry-overs: when Pass 4's structural novelty (per-surface commits + heaviest doc-work pass) conflicts with Pass 3's lighter framing (single Commit 2 seven-surface atomic), Pass-internal consistency wins. The trilogy-internal ~80-100 KB target band is respected at mid-band; Pass 4 lands lighter than the trilogy precedent on grounds of design doc's exhaustive §4.1 + §4.3 + §4.5 walks doing the architectural work upstream.

### §1.3 What this runbook does NOT do

- Does NOT touch xgen-node at Pass 4. Pass 3 closed it (J-138).
- Does NOT promote D-NNN-format-boundary at Pass 4 close. Per design doc §4.2.3 Option γ split: promotion-watch stays OPEN at three structurally-distinct instances across two Pass-arc (D-077 multi-Pass-arc durability not yet met). Fourth structurally-distinct instance at Pass 5 OR cross-milestone closes the gap.
- Does NOT undo D-NNN-ε closure. Per design doc §4.5.3 Option γ honest framing: D-NNN-ε CLOSED by honest framing per D-065 + D-079 (Rust language idiom not XGen-specific decision); D-NNN slot preserved for actual XGen-specific decisions.
- Does NOT add `FromStr` to flavour wrappers. Per design doc §4.3.c: deferred to future audit-design-impl arc per D-071 if protocol-level "valid XGID string format at parse-time" lock surfaces as substantive Pass-arc design decision.
- Does NOT close Pass 5 deferred items: test-fixture sweep at `xgen-client/tests/` integration tests; trace-field formatter audit; Debug + Display impl audit on xgen-client public types; `cargo build --workspace` restoration. Per design doc §2.9.
- Does NOT touch M7 (--aicontrol v1 covering both binaries) scope. Per design doc §2.9.5 + §1.3.
- Does NOT touch M6 (new) Node admin write path scope. Per design doc §2.9.5 + §1.3.
- Does NOT modify the design doc §4.1.a (46-slot classification) + §4.3.0 (16 clap Args) + §4.5.0 (7 async-spawn sites) classification tables. If Clair surfaces a structural gap mid-implementation, STOP per Rule 3 + Lock 1 Trigger (a) and surface for Joe-lock before continuing. Any deviation from the verbatim classification tables requires Joe-lock checkpoint #1 re-approval.
- Does NOT amend DECISIONS.md at Pass 4 milestone close. Two candidate D-NNN promotion-watch states (format-boundary OPEN + ε CLOSED) stay as locked at design close per D-069.

---

## §2 Sequence overview

### §2.1 Per-surface commit sequence (Option B honest §4.4.4 application)

| Commit | Surface / Scope | Files (target) | Atomic posture | Joe-lock checkpoint |
|--------|-----------------|----------------|----------------|---------------------|
| 1 | Surface #1 — M5 Ops Layer (ops.rs Result-struct retype + Pass 1 additive-API extension at xgen-common + Appendix F fragments + per-surface tests) | 4-6 | D-074 atomic | #2 fires post-ship |
| 2 | Surface #2 — CLI Dispatcher (app.rs 16 clap Args projection + format paths + Appendix F CLI section fragments + per-surface tests) | 3-4 | D-074 atomic | — |
| 3 | Surface #3 — Batch Pipe Dispatch (batch.rs get_dag_tips + pipe-side dispatch entry + Appendix F batch reply schema fragments + per-surface tests) | 3-4 | D-074 atomic | — |
| 4 | Surface #4 — Tauri Shell (desktop.rs Tauri command return types + lifecycle state machine + per-surface tests; no per-surface doc fragment) | 2-3 | D-074 atomic | — |
| 5 | Surface #5 — Session State (session.rs ClientIdentity + lifecycle.rs ClientStateEvent + per-surface tests; no per-surface doc fragment) | 2-3 | D-074 atomic | — |
| 6 | Surface #6 — AI Resident (ai_service.rs + ai_behavior.rs + EchoPlugin + xgen_aicontrol_implementation.md fragments + per-surface tests) | 4-5 | D-074 atomic | — |
| 7 | Surface #7 — Pacing + Temperature (pacing.rs + temperature.rs HashMap-key retype + Ch6 §6.15 fragments + per-surface tests) | 3-4 | D-074 atomic | #3 fires post-ship |
| 7a | [CONTINGENT] Test-fixture projection sweep at xgen-client/tests/ if checkpoint #3 fires split | varies | D-074 atomic | — |
| 8 | Milestone close (runbook + design doc J-NNN freeze + JOURNAL J-NNN body + CLAUDE PLAY flip + ROADMAP visual tree row ✅ + Past entry) | 5-6 | D-074 atomic | — |

**Commit-sequence shape Joe-locked-by-recommendation Option B at runbook-authoring J-141** (sibling-shape Pass-2 §7.2 inline-lock pattern). Honest §4.4.4 application: no Commit 1 doc-pass (collapses into per-surface code+doc atomic + milestone-close consolidation per §4.4.b). The J-141 runbook-shipping commit IS the kickoff atomic — ROADMAP + CLAUDE PLAY + JOURNAL J-141 + runbook NEW v1.0 ship now; Commits 1-7 + 7a + 8 ship per Clair's session-arc against the locked classification tables.

### §2.2 Two split triggers (Lock 1 enumeration)

Two triggers documented at this §2.2 mirror Pass 3's pre-locked contingent-split posture per design doc §4.4.c sibling-shape inheritance. Each trigger fires Joe-lock STOP per Rule 3 + Lock 1.

- **Trigger (a)** — non-existent production contract per design doc §4.1.a + §4.3.0 + §4.5.0 verbatim classification tables. If Clair grep at Commit 1 prep (or any subsequent Commit prep) finds a named field, type, method, or async-spawn site does not exist in production code (sibling-shape to J-129 Pass 3 runbook surface-ordering drift + J-133 Q5.14 v1.3 amendment), STOP and surface for Joe-lock canonical-record amendment. **D-078 applies** — production-grounded verification at Joe-lock checkpoint #1 BEFORE any code touches. Pass 3 §7.11 discipline data point ("design-doc-grounded surface enumeration at runbook authoring") instantiates here at table-grounded-verification layer.
- **Trigger (b)** — family-boundary size split if any individual Commit 1-7 exceeds ~600 lines diff (excluding test additions + doc fragments). Family-boundary not arbitrary line count; sibling-shape to Pass 3 §2.2 Trigger (c). Per-surface commits are pre-bounded by their surface's slot count + tests + doc fragment scope; if any surface unexpectedly exceeds boundary, candidate sub-commit-split surfaces at runbook re-walk layer.

### §2.3 Three Joe-lock checkpoints

- **Checkpoint #1 — pre-Commit-1 verbatim classification-table approval.** Clair extracts the design doc §4.1.a (46-slot classification: 31 identifier retypes + 12 descriptive stays + 3 borderline locks) + §4.3.0 (16 identifier-shaped clap Args slots + 5 descriptive stays + 4 transport/config stays) + §4.5.0 (7 async-spawn sites across Surface #4 + #6) verbatim and surfaces them to Joe by name. Joe approves the full table content before any production code lands. This is the LOAD-BEARING D-078 application surface for Pass 4; Trigger (a) fires here if any named field or method does not exist in production. Sibling-shape to Pass 3 checkpoint #2 (pre-Commit-2 verbatim seven-surface Q-tables) but moved to pre-Commit-1 because Pass 4 has no Commit 1 doc-pass per §1.2.
- **Checkpoint #2 — post-Commit-1 first-surface drift check + wire-format invariance witness verification.** Three drift-detection points: (1) ops.rs Result struct retypes landed atomically with their Appendix F doc fragment (no doc-vs-code drift surface); (2) Pass 1 additive-API extension shipped at xgen-common flavour wrappers per §4.1.b Option β (`.is_empty()` + Option `.as_deref()` inherent methods); (3) serde-transparent wire-format invariance witness test (T2 below at §3.4) passes — pre-Pass-4 batch consumer reads byte-identical JSON from post-Pass-4 Result types. Joe approves before Commit 2 begins.
- **Checkpoint #3 — post-Commit-7 split-trigger decision.** Clair runs `cargo test -p xgen-client --tests` and reports test-fixture error count. Joe locks single-Commit-7 (absorb sweep into Commit 7 itself) if errors ≤ ~50, or split (Commit 7 lib-clean + Commit 7a sweep atomic) if errors > ~50. Sibling-shape to Pass 2 checkpoint #3 split-trigger which fired at 93 errors + Pass 3 checkpoint #3 which fired at 638 errors. Pre-locked contingent-split posture is durable cross-Pass discipline per JOURNAL J-138 Sub-section 2.

---

## §3 Commit 1 — Surface #1 M5 Ops Layer

### §3.1 Scope

Commit 1 ships the foundational xgen-client surface per design doc §2.1 + §4.1 dependency-order anchor. All three sub-locks atomic per drift surface uniformity (D-067):

- **§4.1.a Field retype scope** — 46 String slots across 13 Result structs + `HistoryMessage` + `OpContext`: 31 identifier-shaped mechanical retypes per §3 governing principle (e.g. `identity_id` ×4 → `IdentityXgid`, `space_id` ×9 → `SpaceXgid`, `event_id` ×7 → `EventXgid`); 12 descriptive-string mechanical stays (e.g. `display_name`, `version`, `name`); 3 borderline locks (2 × `home_node` / `node` → `NodeXgid` + 1 × `source: Option<String>` operator-source enum-tag stays String).
- **§4.1.b Pass 1 additive-API extension** — inherent `.is_empty()` on six flavour wrappers (`IdentityXgid`, `SpaceXgid`, `EventXgid`, `RoomXgid`, `NodeXgid`, `TrustAssertionXgid`) at xgen-common per Option β; analogous Option `.as_deref()` for `Option<XxxXgid>::as_deref()` returning `Option<&str>`-equivalent. Sibling-shape to Pass 1 Commit 4 `Borrow<str>` additive-API lock — second instance of Pass-arc additive-API extension as load-bearing carry-over.
- **§4.1.c serde-transparent wire-neutrality** — confirmed via wire-format invariance witness test (T2 below). All flavour wrappers are `#[serde(transparent)]` per Pass 1 design; Result struct retypes do not change JSON wire shape.

### §3.2 Narrow scope clarifications

**What §4.1.a retype atomic means.** All three layers retype in same commit per drift surface uniformity (D-067):
- Field types on 13 Result struct declarations at `xgen-client/src/ops.rs` (~46 slots).
- `HistoryMessage` row shape struct field types.
- `OpContext` shared call context struct field types where applicable per Q1.3.

Mid-implementation single-layer retype would create a drift surface where Result-struct fields and call-site projections disagree on flavour. D-067 forbids this. All atomic or none.

**What Pass 1 additive-API extension means.** Six inherent methods + Option-method extension added to xgen-common per §4.1.b:
```rust
impl IdentityXgid {
    pub fn is_empty(&self) -> bool { self.as_str().is_empty() }
}
// Sibling impls for SpaceXgid, EventXgid, RoomXgid, NodeXgid, TrustAssertionXgid.
```

Wire-format-neutral by mechanism (inherent methods don't affect serde derive). Closes the recon §2.10 data point 1 category (b) `.is_empty()` + `.as_deref()` method-availability errors on Xgid newtypes without per-site rewrite churn.

**What serde-transparent wire-neutrality confirms.** All 13 Result structs + HistoryMessage already round-trip through serde with String wire-format pre-Pass-4. Post-Pass-4 typed newtype fields serialise transparently. T2 wire-format invariance witness pins this discipline at the Surface #1 boundary.

### §3.3 Files in this commit (target 4-6 atomic per D-074)

1. `xgen-client/src/ops.rs` — Surface #1 §4.1.a 46-slot retype atomic across 13 Result structs + HistoryMessage + OpContext.
2. `xgen-common/src/xgid/flavours.rs` (or wherever flavour wrappers live) — Surface #1 §4.1.b Pass 1 additive-API extension: inherent `.is_empty()` on six flavour wrappers + Option `.as_deref()` extension. **Cross-crate atomic** — xgen-common edit ships in same commit as xgen-client edit per drift surface uniformity (D-067).
3. `xgen-client/src/ops.rs` (cont) — per-surface tests T1-T3 in-tree `#[cfg(test)] mod pass_4_commit_1_tests` block.
4. `docs/xgen_appendix_f_en.md` — Surface #8 fragment: §F.0.6 M5 ops layer Result-struct field-classification annotation per design doc §4.4.a + §4.1.a. Mechanical edit; annotate per-field typed-XGID-in-memory + String-on-wire per §4.1.c serde-transparent confirmation.
5. This runbook header chain entry recording Commit 1 landed (Status stays ACTIVE v1.0).
6. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

Five-to-six file atomic. Sibling-shape to Pass 3 Commit 2 ten-file atomic but narrower because Pass 4 Commit 1 is one-surface scope vs Pass 3's seven-surface atomic.

### §3.4 Per-surface tests (T1-T3, 3 tests target)

**Joe-lock checkpoint #1 includes per-surface test list approval by name** alongside the design doc §4.1.a + §4.3.0 + §4.5.0 verbatim classification tables. Test naming follows Pass 3 §4.7 precedent (`<surface>_<flavour>_<scenario>`).

- **T1**: `ops_result_struct_field_retype_46_slots_compile` — compile-time witness that all 46 slot retypes per §4.1.a hold (31 identifier mechanical + 12 descriptive stays + 3 borderline locks). Constructs each of the 13 Result struct variants + HistoryMessage with typed-XGID values at every identifier slot + String at every descriptive slot; verifies type checker accepts the construction.
- **T2**: `ops_result_struct_serde_transparent_wire_invariance` — round-trip JSON pre-Pass-4 vs post-Pass-4 byte-identical witness (§4.1.c + Q1.2). Serialises a Result struct with typed XGID fields via `serde_json::to_string`; verifies output matches the canonical pre-Pass-4 String-field shape (e.g. `{"space_id":"xgen://space/sha256:abc..."}`). Wire-format invariance witness per Appendix J §J.5 invariance 2; **load-bearing** for checkpoint #2 verification.
- **T3**: `flavour_wrapper_is_empty_and_as_deref_additive_api_works` — Pass 1 additive-API extension at six wrappers verifies per §4.1.b Option β. Constructs each flavour wrapper from String; calls inherent `.is_empty()`; verifies behaves identically to `.as_str().is_empty()`. Constructs `Option<XxxXgid>`; calls `.as_deref()`; verifies returns `Option<&str>`-equivalent.

**Total Surface #1 test target: 3 tests.**

### §3.5 Verification at Commit 1 boundary

- `cargo build -p xgen-common -p xgen-client --lib` — should be CLEAN at Surface #1 retype scope (Pass 1 additive-API extension + Result struct retypes compile against existing Pass 1+2+3 xgen-core + xgen-node).
- `cargo test -p xgen-common -p xgen-client --lib` — verify T1+T2+T3 pass + xgen-common existing tests still pass (T3 may live in xgen-common if flavour wrapper impls + tests co-located; T1+T2 live in xgen-client/src/ops.rs).
- `cargo clippy -p xgen-common -p xgen-client --lib --all-features -- -D warnings` — clean.
- `cargo build --workspace` deliberately broken at xgen-client + Pass 4 downstream consumer sites only (Surfaces #2-#7 not yet retyped). Per Path A inherited from Pass 1+2+3 per JOURNAL J-138 Sub-section 2 cross-Pass discipline carry-over (three-instance durability).

### §3.6 Layered-B3 audit at Commit 1 verification

Per design doc §5 + §5.4: layered-B3 expected null at full eight-surface scope (four-instance Pass-arc no-finding chain). However Commit 1 still requires Clair to perform the per-surface audit per Rule 5 + D-065 honest-audit-not-honest-assumption discipline. If a layered-B3 surface unexpectedly emerges at Surface #1 implementation time, STOP per Rule 3 and surface for Joe-lock; flag at JOURNAL J-NNN body as discipline data point breaking the Pass-arc pattern.

Surface #1 expected-null reasoning per design doc §5.2: Result structs are flat data carriers; serde-transparent at format boundary per §4.1.c; no secondary encoding of the typing invariant.

---

## §4 Commit 2 — Surface #2 CLI Dispatcher

### §4.1 Scope

Commit 2 ships the CLI Dispatcher surface per design doc §2.2 + §4.3 Option α (clap parse stays String; project at dispatcher arm via Pass 1 wrapper constructor chain).

- **§4.3.a Clap-derive Args structs keep all 16 identifier-shaped String slots as String** at parse boundary per design doc §4.3.0 verbatim table (8 Args structs: AiDelegateArgs + AiRevokeArgs + AiStatusArgs + CreateRoomArgs + InviteArgs + JoinArgs + SendArgs + HistoryArgs).
- **§4.3.b Dispatcher arm projects** `String` → flavour wrapper via Pass 1 `Xgid::new(s) → XxxXgid::from_xgid(...)` constructor chain at call site to `ops::*`. 16 sites × ~1 line per projection.
- **§4.3.c Pass 4 explicitly does NOT add `FromStr` to flavour wrappers** — deferred per D-071 audit-design-impl-arc framing.
- **Format paths consume Display impl** per design doc §2.2 + §4.3.4 — display-time projection at format!() / println!() sites; identifier values project via Display impl on Xgid inherited from Pass 1 D-073 framing.

### §4.2 Files in this commit (target 3-4 atomic per D-074)

1. `xgen-client/src/app.rs` — Surface #2 16 clap-Args dispatcher-arm projections + format-path Display calls + per-surface tests T4-T5 in-tree.
2. `docs/xgen_appendix_f_en.md` — Surface #8 fragment: CLI Reference per-verb signature annotation per design doc §4.4.a + §4.3.b. Annotate per-arg typed-XGID-projected-at-dispatcher-arm.
3. This runbook header chain entry.
4. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

Four-file atomic.

### §4.3 Per-surface tests (T4-T5, 2 tests target)

- **T4**: `cli_args_clap_parse_stays_string_and_projects_typed_at_dispatch_arm` — verifies §4.3.b Option α projection mechanism. Constructs each of the 8 Args structs with String values; verifies the dispatcher arm projects to typed flavour wrappers before passing to `ops::*`. Covers all 16 identifier-shaped slots per design doc §4.3.0.
- **T5**: `cli_format_path_typed_result_displays_via_display_impl` — verifies format!() / println!() sites at format paths consume Display impl correctly per §2.2 + §4.3.4. Constructs a typed Result struct from Commit 1; passes through format!() macro; verifies output matches expected String form.

**Total Surface #2 test target: 2 tests.**

### §4.4 Verification at Commit 2 boundary

- `cargo build -p xgen-client --lib` — should be CLEAN at Surface #2 retype scope.
- `cargo test -p xgen-client --lib` — verify T4+T5 pass.
- `cargo clippy -p xgen-client --lib --all-features -- -D warnings` — clean.
- `cargo build --workspace` deliberately broken at Surfaces #3-#7 + Pass 5 downstream sites; verify breakage is at unretyped surfaces only.

### §4.5 Layered-B3 audit at Commit 2 verification

Surface #2 expected-null reasoning per design doc §5.2: clap parse boundary + projection at dispatcher arm per §4.3 Option α; no secondary validation surface. Sibling-shape to Pass 3 Surface #2 dispatch_event signature retype which closed at null.

---

## §5 Commit 3 — Surface #3 Batch Pipe Dispatch

### §5.1 Scope

Commit 3 ships the Batch Pipe Dispatch surface per design doc §2.3 + §4.2 Instance B (pipe JSON consolidated under Pass 3 wire-shape boundary class).

- **Q3.1 retype** — `get_dag_tips(space_id: String)` parameter retype to `SpaceXgid` at pipe-side dispatch entry; projection from String at pipe boundary (JSON-decoded payload) → typed XGID at dispatch entry boundary per §4.3.b mechanism.
- **§4.2 Instance B wire-shape boundary** — pipe JSON serde over named-pipe byte stream preserves wire-format identity (typed XGID newtypes serialise as plain String per §4.1.c). Same mechanism as Pass 3 wire-shape boundary class — no new instance count per §4.2.3 Option γ split.
- **Q3.2 batch reply schema annotation in Appendix F** — typed-XGID-in-memory + String-on-wire note per §4.3 format-boundary preservation; folded into Surface #8 doc-tree fragment shipped atomic.
- **Q3.3 pipe protocol error handling** — error replies carry identifier material via Result rejection paths; format-boundary stays String per §4.2 Instance B.

### §5.2 Files in this commit (target 3-4 atomic per D-074)

1. `xgen-client/src/batch.rs` — Surface #3 `get_dag_tips` retype + pipe-side dispatch entry boundary projection + per-surface tests T6-T7 in-tree.
2. `docs/xgen_appendix_f_en.md` — Surface #8 fragment: batch reply schema annotation per design doc §4.4.a + Q3.2.
3. This runbook header chain entry.
4. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

Four-file atomic.

### §5.3 Per-surface tests (T6-T7, 2 tests target)

- **T6**: `batch_get_dag_tips_space_xgid_at_dispatch_entry_boundary` — verifies Q3.1 boundary projection at pipe-side dispatch entry. Constructs a pipe request with String `space_id`; verifies dispatcher entry projects to `SpaceXgid` before calling `ops::get_dag_tips`.
- **T7**: `batch_reply_json_serde_transparent_wire_invariance` — verifies reply serialisation preserves wire-format per §4.2 Instance B. Constructs a typed Result struct response; serialises via `serde_json::to_string`; verifies output JSON matches the canonical pre-Pass-4 String-field shape. Sibling-shape to T2 at Surface #1 but at pipe-reply boundary.

**Total Surface #3 test target: 2 tests.**

### §5.4 Verification at Commit 3 boundary

- `cargo build -p xgen-client --lib` — CLEAN at Surface #3 retype scope.
- `cargo test -p xgen-client --lib` — verify T6+T7 pass.
- `cargo clippy -p xgen-client --lib --all-features -- -D warnings` — clean.
- `cargo build --workspace` deliberately broken at Surfaces #4-#7 + Pass 5 downstream sites.

### §5.5 Layered-B3 audit at Commit 3 verification

Surface #3 expected-null reasoning per design doc §5.2: JSON serde over named pipe — same mechanism as wire (§4.2 Instance B consolidated under Pass 3 wire-shape boundary class); no second surface.

---

## §6 Commit 4 — Surface #4 Tauri Shell

### §6.1 Scope

Commit 4 ships the Tauri Shell surface per design doc §2.4 + §4.2 Instance C (Tauri IPC fresh boundary class at Pass 4) + §4.5 async-spawn captures sub-rule application at 3 `#[tauri::command]` handlers.

- **Q4.1 Tauri command signatures** — `get_state` + `get_pacing_state` + Tauri emit surface (lines 54, 63, 90 per recon §2.4 + design doc §4.5.0) return ClientStateEvent → identifier slots retype via lifecycle.rs Surface #5 (consumed in Commit 5).
- **Q4.2 Tauri emit surface for ClientStateEvent** — serialisation format preserves String wire identity per Pass 1 serde-transparent + §4.2 Instance C fresh boundary class confirmation.
- **Q4.3 Lifecycle state machine String fields** — identifier slots in state-tracking structures retype; descriptive slots (state names, transitions) stay String per D-073 field-name-vs-type discipline. Lifecycle.rs edits ship at Commit 5 (Surface #5); desktop.rs Tauri command return types consume the retyped lifecycle types.
- **§4.5 async-spawn captures** — 3 `#[tauri::command]` handler async sites confirm ubiquitous Rust language idiom per Option γ honest framing closure. No per-site code change required beyond the type-flow from Surface #5.

### §6.2 Files in this commit (target 2-3 atomic per D-074)

1. `xgen-client/src/desktop.rs` — Surface #4 Tauri command return type annotations (consuming Surface #5 retyped types) + per-surface tests T8-T9 in-tree. No direct String slot retypes per recon §2.10 (0 direct String slots); identifier material flows via Tauri command return types from session.rs + ops::*.
2. This runbook header chain entry.
3. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

Three-file atomic. **No per-surface doc fragment** — Tauri Shell internals not surfaced in Appendix F or other doc-tree docs at Pass 4 scope.

### §6.3 Per-surface tests (T8-T9, 2 tests target)

- **T8**: `tauri_command_return_serde_transparent_to_js_frontend` — verifies Q4.1 Tauri command returns project through serde-transparent boundary per §4.2 Instance C fresh boundary class. Constructs a ClientStateEvent with typed XGID identifier slots; serialises via Tauri's serde bridge (or representative serde_json call); verifies JS-frontend-visible JSON matches canonical String-shape pre-Pass-4.
- **T9**: `lifecycle_state_event_identifier_slots_retyped_descriptive_stays_string` — verifies Q4.3 lifecycle state machine field classification. Constructs ClientStateEvent variants; verifies identifier-shaped fields are typed XGID, descriptive fields (state names, transition labels) stay String per D-073.

**Total Surface #4 test target: 2 tests.**

### §6.4 Verification at Commit 4 boundary

- `cargo build -p xgen-client --lib --features tauri` (or whatever Tauri feature gating exists; verify at session-open against current Cargo.toml) — CLEAN at Surface #4 retype scope assuming Surface #5 retyped types are pre-staged from Commit 5.
- **Sequencing note**: Surface #4 depends on Surface #5 retyped types. Implementation order option α — ship Commit 4 + Commit 5 simultaneously as paired commits; option β — reorder to ship Surface #5 first then Surface #4. **Joe-lock at session-time turn** if Clair surfaces ordering tension. Recommended option α (atomic-pair via fast-follow ship) since both surfaces are narrow + tightly coupled at lifecycle.rs ClientStateEvent payload definition.
- `cargo test -p xgen-client --lib` — verify T8+T9 pass.
- `cargo clippy -p xgen-client --lib --all-features -- -D warnings` — clean.
- `cargo build --workspace` deliberately broken at Surfaces #6-#7 + Pass 5 downstream sites.

### §6.5 Layered-B3 audit at Commit 4 verification

Surface #4 expected-null reasoning per design doc §5.2: Tauri commands return serde-transparent types; JS/TS frontend sees plain strings; no secondary encoding. §4.2 Instance C fresh boundary class is structurally distinct boundary, not layered-B3 surface (different audit dimension).

---

## §7 Commit 5 — Surface #5 Session State

### §7.1 Scope

Commit 5 ships the Session State surface per design doc §2.5. Foundational surface consumed by Surfaces #1 + #2 + #3 + #4 + #6 + #7.

- **Q5.1 ClientIdentity struct** — `identity_id` + `home_node` fields retype to typed XGID (`IdentityXgid` + `NodeXgid`).
- **Q5.2 SessionState struct** — identifier slot retype + M7-shape extension field types (bindings map keys, spaces cache keys) — descriptive-vs-identifier classification per slot following §3 governing principle.
- **Q5.3 Lifecycle state event payloads** at `lifecycle.rs` — ClientStateEvent identifier slots retype; state-name + transition-label fields stay String per D-073 (consumed by Commit 4 Surface #4).
- **Q5.4 On-disk persistence at xgen-client_state.json** — serde-transparent preserves wire format; on-disk JSON shape unchanged per §4.2 wire-shape boundary class. Format-boundary preservation per Pass 3 §4.3 v1.2 sibling-shape application.
- **Q5.5 Idempotent ensure_* helpers** (`ensure_identity` / `ensure_connected`) — parameter signatures retype.

### §7.2 Files in this commit (target 2-3 atomic per D-074)

1. `xgen-client/src/session.rs` — Surface #5 ClientIdentity + SessionState + ensure_* helpers retypes + per-surface tests T10-T11 in-tree.
2. `xgen-client/src/lifecycle.rs` — ClientStateEvent identifier slot retypes per Q5.3.
3. This runbook header chain entry.
4. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

Three-to-four file atomic. **No per-surface doc fragment** — Session State internals not surfaced in doc-tree at Pass 4 scope.

### §7.3 Per-surface tests (T10-T11, 2 tests target)

- **T10**: `client_identity_identifier_slots_retyped` — verifies Q5.1 ClientIdentity `identity_id` + `home_node` retyped to typed XGID. Constructs ClientIdentity with typed fields; verifies field types via type checker witness.
- **T11**: `session_state_on_disk_persistence_format_round_trip_string_at_boundary` — verifies Q5.4 xgen-client_state.json serde-transparent preserves wire-format. Serialises SessionState with typed fields via serde_json; deserialises; verifies typed values reconstruct correctly + on-disk JSON shape matches canonical pre-Pass-4 String shape (sibling-shape to T2 at Surface #1 but at session-state-persistence boundary).

**Total Surface #5 test target: 2 tests.**

### §7.4 Verification at Commit 5 boundary

- `cargo build -p xgen-client --lib` — CLEAN at Surface #5 retype scope.
- `cargo test -p xgen-client --lib` — verify T10+T11 pass.
- `cargo clippy -p xgen-client --lib --all-features -- -D warnings` — clean.
- `cargo build --workspace` deliberately broken at Surfaces #6-#7 + Pass 5 downstream sites.

### §7.5 Layered-B3 audit at Commit 5 verification

Surface #5 expected-null reasoning per design doc §5.2: pure in-memory cache; no validation surface; cleanest §3 application surface per §3.2 sanity-check table.

---

## §8 Commit 6 — Surface #6 AI Resident

### §8.1 Scope

Commit 6 ships the AI Resident surface per design doc §2.6 + §4.5 async-spawn captures sub-rule application at 4 `tokio::spawn` sites (ai_service.rs:554+575; service.rs:183+202 per design doc §4.5.0).

- **Q6.1 AiBehavior trait method signatures** — `on_event` / `propose_reply` etc. identifier slots in param/return types retype per §3 governing principle.
- **Q6.2 AiPacingTracker per-Space pacing key** — `HashMap<SpaceXgid, _>` per D-060 (Ch3 §3.7.12) sibling-shape to Surface #7 pacing.rs per-(space, sender) HashMap key retype.
- **Q6.3 EchoPlugin reference impl** — `ai_identity_id` + `sender_identity_id` slots in reply path retype.
- **Q6.4 AI mode `__HEALTH__` extension `operator_known=N/M`** — identifier accounting retype.
- **§4.5 async-spawn captures** — 4 `tokio::spawn` sites confirm ubiquitous Rust language idiom per Option γ honest framing closure. Captured typed XGID parameters declared owned (not borrowed) at spawned-function signature; `Arc<TypedXgid>` shared-reference pattern if needed across multiple spawned tasks.

### §8.2 Files in this commit (target 4-5 atomic per D-074)

1. `xgen-client/src/ai_service.rs` — Surface #6 AiPacingTracker + AI mode `__HEALTH__` + 2 `tokio::spawn` sites (ai_service.rs:554+575) retypes + per-surface tests T12-T13 in-tree.
2. `xgen-client/src/ai_behavior.rs` — AiBehavior trait method signatures + EchoPlugin reference impl retypes.
3. `xgen-client/src/service.rs` (or wherever the 2 service.rs:183+202 `tokio::spawn` sites live; verify at session-open) — async-spawn forced-owned typed XGID captures.
4. `docs/xgen_aicontrol_implementation.md` — Surface #8 fragment: AI resident + AiBehavior trait + EchoPlugin sections gain typed-XGID slot callouts per design doc §4.4.a + Q8.2 (M7-future-redesign demarcation — current annotations are Pass 4 typed-XGID scope, not Pass 4 doing M7's job).
5. This runbook header chain entry.
6. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

Five-to-six file atomic.

### §8.3 Per-surface tests (T12-T13, 2 tests target)

- **T12**: `ai_behavior_trait_method_signature_identifier_slots_typed` — verifies Q6.1 AiBehavior trait `on_event` / `propose_reply` signatures retype to typed XGID per §3 governing principle. Implements a test plugin impl AiBehavior; verifies signature compiles with typed-XGID params/returns.
- **T13**: `ai_pacing_tracker_per_space_xgid_key` — verifies Q6.2 AiPacingTracker per-Space pacing key typed. Constructs AiPacingTracker; inserts entry with `SpaceXgid` key; retrieves via `Borrow<str>` projection from Pass 1 Commit 4 additive-API.

**Total Surface #6 test target: 2 tests.**

### §8.4 Verification at Commit 6 boundary

- `cargo build -p xgen-client --lib` — CLEAN at Surface #6 retype scope.
- `cargo test -p xgen-client --lib` — verify T12+T13 pass + existing ai_behavior.rs 10 + ai_service.rs 8 in-tree tests still pass.
- `cargo clippy -p xgen-client --lib --all-features -- -D warnings` — clean.
- `cargo build --workspace` deliberately broken at Surface #7 + Pass 5 downstream sites.

### §8.5 Layered-B3 audit at Commit 6 verification

Surface #6 expected-null reasoning per design doc §5.2: spawned tasks own typed XGIDs per §4.5 Option γ; same mechanism as Pass 3 Surface #5/#6 reconnect.rs + federation_session.rs spawned functions which closed at null.

---

## §9 Commit 7 — Surface #7 Pacing + Temperature

### §9.1 Scope

Commit 7 ships the Pacing + Temperature surface per design doc §2.7. Sibling-shape to Pass 3 Surface #4 fanout.rs ClientSenders + FederationPeerSenders HashMap-key retype.

- **Q7.1 pacing.rs HashMap key composite** — `HashMap<(String, String), _>` keyed by (space_id, sender_identity_id) per D-060 Ch3 §3.7.12 retype to `HashMap<(SpaceXgid, IdentityXgid), _>`. `Borrow<str>` lookup mechanism from Pass 1 Commit 4 additive-API handles call-site projection.
- **Q7.2 temperature.rs event payload struct** — identifier slots retype per §3 governing principle; descriptive slots stay String. Per design doc §2.7 Q7.2: `subject_id` classification open question (stays String per D-061 spec OR retypes per general principle); **default per §3 governing principle is identifier-shape → typed**. If walk-time grep surfaces ambiguity, STOP per Trigger (a) and surface for Joe-lock at session-time.
- **§4.4.a Surface #8 fragment** — Ch6 §6.15 pacing + temperature subsections gain typed-XGID slot callouts per design doc §4.4.a + Q8.3.

### §9.2 Files in this commit (target 3-4 atomic per D-074)

1. `xgen-client/src/pacing.rs` — Surface #7 HashMap key composite retype + per-surface tests T14 in-tree.
2. `xgen-client/src/temperature.rs` — temperature event payload identifier slots retype + per-surface tests T15 in-tree.
3. `docs/xgen_ch6_client_design.md` §6.15 — Surface #8 fragment: pacing + temperature subsections typed-XGID slot callouts per Q8.3.
4. This runbook header chain entry.
5. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

Four-to-five file atomic.

### §9.3 Per-surface tests (T14-T15, 2 tests target)

- **T14**: `pacing_per_space_sender_map_insert_retrieve_with_typed_key` — verifies Q7.1 `HashMap<(SpaceXgid, IdentityXgid), _>` retype + `Borrow<str>` projection mechanism. Insert entry with typed composite key; retrieve via String projection at call site per Pass 1 Commit 4 additive-API. Sibling-shape to Pass 3 T1 (`noderuntime_per_space_map_insert_retrieve_with_typed_key`).
- **T15**: `temperature_event_payload_identifier_slots_retyped` — verifies Q7.2 identifier slots typed + descriptive slots stay String per §3 governing principle. Constructs TemperatureEventPayload; verifies field types per type checker witness. If `subject_id` retypes per the locked classification at checkpoint #1, this test pins the retype; otherwise pins the stay-as-String per D-061 spec.

**Total Surface #7 test target: 2 tests.**

### §9.4 Verification at Commit 7 boundary

- `cargo build -p xgen-client --lib` — CLEAN at Surface #7 retype scope.
- `cargo test -p xgen-client --lib` — verify T14+T15 pass.
- `cargo clippy -p xgen-client --lib --all-features -- -D warnings` — clean.
- `cargo build --workspace` deliberately broken at Pass 5 downstream sites only.

### §9.5 Joe-lock checkpoint #3 fires post-Commit-7 ship

Clair runs `cargo test -p xgen-client --tests` after Commit 7 lib-clean verification. Reports test-fixture error count to Joe. Joe locks:

- **Single-Commit-7 (absorb sweep)** if errors ≤ ~50 — absorb test-fixture updates into Commit 7 itself; no Commit 7a. Re-ship Commit 7 with absorbed sweep.
- **Split (Commit 7a)** if errors > ~50 — separate atomic commit for test-fixture projection sweep per D-074 preservation of atomic discipline + Pass 1 + Pass 2 + Pass 3 precedent.

### §9.6 Layered-B3 audit at Commit 7 verification

Surface #7 expected-null reasoning per design doc §5.2: HashMap keys use `Borrow<str>` projection from Pass 1 Commit 4 additive-API; sibling-shape to Pass 3 Surface #4 fanout.rs HashMap-key retype which closed at null.

---

## §10 Commit 7a — Test-fixture projection sweep [CONTINGENT]

### §10.1 Fires at Joe-lock checkpoint #3 if error count > ~50

Sibling-shape to Pass 3 Commit 2a `0cdf0ad` (which fired at 638 errors) + Pass 2 Commit 2a `58b94a5` (93 errors) + Pass 1 Commit 4a `4895446` precedent.

Pre-locked contingent-split posture is durable cross-Pass discipline per JOURNAL J-138 Sub-section 2; criterion (~50 errors at checkpoint #3) is empirically grounded at three prior milestone closes.

### §10.2 Scope if fires

Mechanical projection-only edit across `xgen-client/tests/` integration test fixtures + any cross-surface test-fixture errors not absorbed at per-surface Commits 1-7. Pattern at Pass 3 (sibling-shape):

```rust
// BEFORE: untyped String construction
let space_id = "test_space_xgid".to_string();

// AFTER: typed construction
let space_id = SpaceXgid::from_xgid(Xgid::new("test_space_xgid".to_string()));
// OR: helper function that hides projection (sibling-shape to Pass 3's sdx/ndx/idx/edx/rdx test helpers)
let space_id = sdx("test_space_xgid");
```

Pass 3 introduced typed-XGID test helpers (`sdx` / `ndx` / `idx` / `edx` / `rdx`) at xgen-node test modules. Pass 4 may inherit these helpers via re-export or add sibling xgen-client-side helpers per Clair's judgment at Commit 7a implementation time.

**Parallel-subagent delegation candidate per Pass 3 §9.7 discipline data point**: if Pass 4 Commit 7a test-fixture error count exceeds ~500 at checkpoint #3 report, parallel-subagent delegation under per-crate guard-rails is a viable shape; discipline cost is explicit honest-deviation reporting at integration time (Rule 1) + per-crate independence verification.

### §10.3 Verification at Commit 7a boundary

Re-run 8 GREEN protocol per §11.3 after sweep lands. Verify total test count matches per-surface test additions (+15 target if all land + any absorbed sweep tests).

### §10.4 Files in this commit if fires

Per-test-module sweep target 4-15+ files at `xgen-client/tests/` + any cross-surface fixture modules. Sibling-shape to Pass 3 Commit 2a thirty-file atomic.

Additional D-074 atomic files:
- This runbook header chain entry recording Commit 7a landed.
- `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

---

## §11 Commit 8 — Milestone close

### §11.1 Scope

Pass 4 milestone close per D-074 atomic + J-108 codification + design doc §4.4.b cross-surface fragments. Five-to-six file atomic commit. Sibling-shape to Pass 3 Commit 3 milestone-close `8146ef0` (J-138 five-file atomic).

### §11.2 Files in this commit (target 5-6 atomic per D-074 + §4.4.b)

1. `tasks/XGID_RETROFIT_PASS_4_IMPL.md` — Status ACTIVE → COMPLETED + version bump v1.0 → v1.1 + Last-updated milestone-close note + DoD checklist verified.
2. `tasks/XGID_RETROFIT_PASS_4_DESIGN.md` — header chain entry only + §6.1 J-NNN placeholder freeze (per J-108 codification + Pass 3 §6.1 + Pass 2 §6.7 freeze pattern).
3. `JOURNAL.md` — J-NNN body entry with full milestone-close pattern per HANDOFF/precedent spec (target eight to ten sub-sections sibling-shape to J-122 + J-126 + J-138).
4. `CLAUDE.md` — header chain entry; PLAY block flip "XGID Retrofit Pass 4 implementation ACTIVE — Clair pickup at runbook §3 Commit 1" → "XGID Retrofit Pass 4 milestone CLOSED at J-NNN; standby for next-milestone selection (Pass 5 + M6 (new) both ready)".
5. `docs/ROADMAP.md` — version bump + visual tree Pass 4 row 🟢 → ✅ with full sub-bullet detail + Past entry + Present updated + Near future Pass 4 line removed + header chain.

Possibly sixth file: any code-side J-NNN code-comment freezes per J-108 codification grep guardrail (`grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO post-staging).

### §11.3 Verification at milestone close (8 GREEN minimum per Pass 3 §4.9 + §5.3)

- 5 isolated runs (`cargo clean -p xgen-common -p xgen-client` between each + `cargo test -p xgen-common -p xgen-core -p xgen-node -p xgen-client --lib`) — ALL GREEN.
- 3 consecutive workspace runs (`cargo test -p xgen-common -p xgen-core -p xgen-node -p xgen-client --lib` without intervening clean) — ALL GREEN.
- 8/8 GREEN minimum threshold met.
- `cargo clippy -p xgen-common -p xgen-core -p xgen-node -p xgen-client --lib --all-features -- -D warnings` clean.
- `cargo clippy -p xgen-common -p xgen-core -p xgen-node -p xgen-client --tests --all-features -- -D warnings` clean.
- `cargo build --workspace` deliberately broken at Pass 5 downstream sites only per Path A inherited (Pass 5 closes both Pass 4 + Pass 5 test-fixture sweep + workspace build restoration per design doc §2.9).
- `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches post-staging per J-108 codification.

Pre-existing flakes (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`) — document if fire but do not block per J-101 framing + Pass 3 J-137 inheritance.

### §11.4 What unblocks

- **XGID Retrofit Pass 5** — test-fixture sweep at `xgen-client/tests/` + trace-field formatter audit + Debug + Display impl audit on xgen-client public types + `cargo build --workspace` restoration. Per design doc §2.9. Runbook authoring is the next Chat Claude work-shape on the XGID retrofit track after Pass 4 close.
- **M6 (new) Node admin write path** — stays unblocked-but-not-selected per J-138 + J-140 inheritance; opens after Joe selects the next-active milestone at session open. Pass 5 + M6 (new) are both ready for selection; sequencing is Joe's call.

### §11.5 Definition of Done

DoD checklist for milestone close — Clair verifies each before staging:

- [ ] All seven xgen-client surfaces + Surface #8 doc-tree sweep from design doc §2 + §4 retyped per locked decisions at §4.1 + §4.2 + §4.3 + §4.5.
- [ ] Per-surface tests landed (target +15 = T1-T15 unless Joe locked different count at checkpoint #1).
- [ ] `cargo test -p xgen-common -p xgen-core -p xgen-node -p xgen-client --lib` GREEN (8/8 minimum at milestone-bearing boundary; re-verified at Commit 8).
- [ ] Both clippy gates clean (`--lib` + `--tests`, `-D warnings`).
- [ ] `cargo build --workspace` deliberately broken at Pass 5 downstream sites only (no regression at xgen-common + xgen-core + xgen-node + xgen-client lib retypes).
- [ ] Layered-B3 audit answer recorded in JOURNAL J-NNN body (expected null per design doc §5 + four-instance Pass-arc no-finding chain durability; flag at JOURNAL if surface unexpectedly emerges).
- [ ] Design doc §6.1 J-NNN placeholder frozen to milestone-close J-NNN per J-108 codification.
- [ ] `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches post-staging.
- [ ] D-NNN-format-boundary promotion-watch status recorded in JOURNAL J-NNN body — STAYS OPEN at three structurally-distinct instances across two Pass-arc per design doc §4.2.3; fourth structurally-distinct instance at Pass 5 OR cross-milestone closes durability gap.
- [ ] D-NNN-ε CLOSURE inherited per design doc §4.5.3 — recorded in JOURNAL J-NNN body as Pass-internal precedent for promotion-watch close-by-honest-framing.
- [ ] "Honest longer work over fast shortcuts" Pass 4 final count recorded in JOURNAL J-NNN body (target zero per Pass 2 milestone-close precedent + Pass 3 close-event-not-recurrence-event inheritance).
- [ ] D-074 application count incremented (thirty-eighth+ instance at Commit 1 + per-commit increments through milestone close; milestone-close tally fifteenth at this Commit 8).
- [ ] Cross-Pass discipline carry-overs verified intact per JOURNAL J-138 Sub-section 2 enumeration (Path A inheritance; Borrow<str> additive API; Layered-B3 expected-null; Pass-internal-consistency framing; Pre-locked contingent-split posture).

### §11.6 What this commit does NOT do

- Does NOT amend DECISIONS.md. D-NNN-format-boundary stays OPEN per §4.2.3; D-NNN-ε CLOSED at design phase per §4.5.3 (no DECISIONS.md amendment needed for closure since promotion-watch closure stays at canonical-design-doc rule table per §4.5.b).
- Does NOT touch xgen-client/tests/ test-fixture sweep beyond Commit 7a contingent scope. Pass 5 closes deferred items per §11.4.
- Does NOT close the D-071 future-removal arc for `validate_steps_8_13` + `accept_event` (Pass 2 §4.2 Q5.b deprecation attributes). That removal arc stays pending; surface-driven per D-071.
- Does NOT add `FromStr` to flavour wrappers. Deferred per design doc §4.3.c + D-071.
- Does NOT touch M6 (new) or M7 scope.

---

## §12 Discipline notes (six sub-sections)

Pass-4-specific discipline notes. Lighter than Pass 3's nine sub-sections because Pass 4 inherits more cross-Pass discipline carry-overs from JOURNAL J-138 Sub-section 2 without re-derivation; sub-section count reduction is honest framing per D-065 not corner-cutting.

### §12.1 Precedent-departure self-defense at runbook layer

Pass 4 runbook diverges from Pass 3 in commit-sequence shape per design doc §4.4 Option γ hybrid-split. Three drivers for the departure:

1. **Eight surfaces vs Pass 3's seven** — one additional surface (Surface #8 doc-tree sweep distributed atomic with code per §4.4.a).
2. **Per-surface commits vs Pass 3's seven-surface atomic** — design doc §4.4 explicitly locks Option γ hybrid-split at design phase per D-069 + D-071 audit-precedes-dependent-design framing. Runbook commit-sequence is downstream consequence of doc-tree coupling shape.
3. **No Commit 1 doc-pass** — per design doc §4.4.4: per-surface doc fragments ship atomic with code; ROADMAP + CLAUDE PLAY + JOURNAL bumps consolidate at milestone close per §4.4.b. The J-141 runbook-shipping commit IS the kickoff atomic.

Pass-internal-consistency framing per design doc §7.7 + JOURNAL J-138 Sub-section 2 cross-Pass discipline carry-overs: when Pass 4's structural novelty conflicts with Pass 3's lighter framing, Pass-internal consistency wins.

### §12.2 Option B commit-sequence Joe-locked-by-recommendation at runbook authoring

At runbook-authoring J-141, two Options walked for the §2.1 commit-sequence shape:

- **Option A — Pass-3-shape-mechanical**: retain "Commit 1 doc-pass minimal" + Commits 2-8 per-surface + Commit 8a contingent + Commit 9 close (9-10 commits).
- **Option B — honest §4.4.4 application**: no Commit 1 doc-pass; Commits 1-7 per-surface atomic + Commit 7a contingent + Commit 8 close (8-9 commits).

**Option B locked** per Joe-lock-by-recommendation (sibling-shape Pass-2 §7.2 inline-lock pattern fifth recurrence; per design doc §7.6 "fifth recurrence skipped per minimal-broadening discipline" — Pass 2 §7.2 already codifies the inline-lock pattern; recording at this §12.2 honestly per D-065 without elevating to canonical-discipline). Honest §4.4.4 application: zero-content Commit 1 doc-pass under Option A would be sibling-shape to J-131 "honest two-file vs three-file" collision but worse (zero-file commit per post-strip discipline); Option B closes the collision pre-emptively at runbook-authoring layer.

Discipline data point recorded for Pass 5 + future Pass-arc runbook authors: when design phase locks per-surface doc fragment atomic shape (Option γ hybrid split family), runbook commit-sequence default is per-surface code+doc atomic without Commit 1 doc-pass; runbook-authoring-time honest framing collapses the doc-pass commit into the runbook-shipping kickoff atomic per §4.4.b cross-surface fragments framing.

### §12.3 Pass 1 additive-API extension at xgen-common flavour wrappers (second instance load-bearing carry-over)

§4.1.b Option β locks inherent `.is_empty()` + Option `.as_deref()` additive-API extension at six xgen-common flavour wrappers per Pass 1 Commit 4 `Borrow<str>` additive-API precedent. **Second instance** of Pass-arc additive-API extension as load-bearing cross-Pass discipline:

- **Pass 1 Commit 4 (J-122)** — `Borrow<str>` on Xgid + six flavour wrappers; load-bearing for Pass 2 + Pass 3 + Pass 4 + Pass 5 HashMap lookup sites without per-query wrapper allocation.
- **Pass 4 Commit 1 (this Commit 1)** — inherent `.is_empty()` + Option `.as_deref()` on six flavour wrappers; closes recon §2.10 data point 1 category (b) without per-site rewrite churn at the ~hundreds of method-availability call sites at Surface #1 + Surfaces #2-#7 consumer sites.

Discipline data point recorded for sibling milestone authors: Pass 1's additive API was the load-bearing enabling decision for the entire five-Pass arc; Pass 4 Commit 1's additive-API extension is the second instance proving the pattern's durability. Future Pass-arc authors encountering call-site method-availability ergonomics should evaluate additive-API extension as the default option over per-site rewrite per D-077/D-078 surface-diversity framing now meeting two-instance threshold.

### §12.4 Format-boundary preservation Option γ split (D-NNN-format-boundary stays OPEN)

§4.2 locked Option γ split at design phase: A (Surface #1 stdout) + B (Surface #3 pipe JSON) consolidate under Pass 3 wire-shape boundary class (no new instance count); C (Surface #4 Tauri IPC) recognised as fresh boundary class at Pass 4. Total at this lock: 3 structurally-distinct instances across 2 Pass-arc.

**D-NNN-format-boundary promotion-watch STAYS OPEN at milestone close** per design doc §4.2.3 promotion-watch state table — D-077 multi-Pass-arc durability NOT yet met (Pass 4 boundary class is fresh-at-Pass-4). Promotion trigger: fourth structurally-distinct instance at Pass 5 OR cross-milestone (M6/M7 admin write path + possible future gRPC / WebRTC / HTTP API surfaces) closes durability gap and promotes to D-080.

T2 (Surface #1 stdout) + T7 (Surface #3 pipe JSON) + T8 (Surface #4 Tauri IPC) per-surface tests act as wire-format invariance witnesses at each instance — each test fails if the boundary class regresses, providing per-instance test-fixture coverage of the promotion-watch state for Pass 5 + cross-milestone audit.

### §12.5 D-NNN-ε CLOSED by honest framing (sibling-shape Pass-internal precedent)

§4.5 locked Option γ honest framing closure at design phase: D-NNN-ε promotion-watch CLOSED by honest framing per D-065 + D-079 — rule is Rust language idiom (`'static` bound on `tokio::spawn`), not XGen-specific decision. Pass 3 §4.2 v1.2 third row sibling-shape rule table extended at canonical-design-doc layer to record Pass 4 instances (7 sites at Surface #4 + #6).

**Pass-arc two-instance pattern of honest-framing-resolution of promotion-watches at Pass-arc design close** per design doc §7.4:
- Pass 3 §4.5 J-127 — D-NNN-γ — held open with two instances per D-069.
- Pass 4 §4.5 J-140 — D-NNN-ε — closed by honest framing per D-065 + D-079.

Plus the promotion-by-honest-framing precedent from Pass 3 J-134 (D-079 promotion atom). Three resolution shapes recorded: promote / close-by-honest-framing / hold-open-by-surface-diversity-threshold. All three are valid; selection requires honest assessment of instance count + structural diversity + Pass-arc durability against D-077/D-078 framing.

Discipline data point recorded for Pass 5 + future Pass-arc design phase walks: promotion-watch boundary admits three shapes; D-NNN slot preserved for actual XGen-specific decisions; ubiquity of a Rust language idiom confirms language-idiom framing rather than promoting to project decision.

### §12.6 Layered-B3 expected null per four-instance Pass-arc no-finding chain durability

Per design doc §5 + §5.4: layered-B3 confirmed expected null at full eight-surface scope. Pass-arc pattern's durability at four instances (Pass 1 J-122 + Pass 2 J-126 + Pass 3 J-138 + Pass 4 J-140 design-close expected-null) makes the expected-null finding evidence-grounded.

However per-surface §3.6 + §4.5 + §5.5 + §6.5 + §7.5 + §8.5 + §9.6 audits still require Clair to perform the audit at each Commit boundary per Rule 5 + D-065 honest-audit-not-honest-assumption discipline. If a layered-B3 surface unexpectedly emerges at implementation time, STOP per Rule 3 and surface for Joe-lock; flag at JOURNAL J-NNN body as discipline data point breaking the Pass-arc pattern.

The mechanism: identifier-slot retype scopes do not surface layered-B3 because the projection mechanism (`Borrow<str>`) handles type-projection at boundaries uniformly without forcing secondary encodings of the same invariant across all retyped functions. This is the structural reason Pass-arc expects null; the empirical confirmation across four Pass-arc instances grounds the expectation.

Pass 5 inheritance per design doc §5.4: Pass 5 design phase opens with expected-null at Pass 5 scope per four-instance chain durability; Pass 5 runbook authoring inherits the per-Commit audit-at-implementation-boundary discipline without re-derivation.

---

## §13 Cross-references

### §13.1 Design doc anchors

- `tasks/XGID_RETROFIT_PASS_4_DESIGN.md` COMPLETED v1.2 at J-140:
  - §1 Framing + §1.2 precedent-positioning + §1.3 NOT scope
  - §2 Surface enumeration (eight surfaces including Surface #8 doc-tree sweep)
  - §3 Governing principle (inherited from Pass 2 + Pass 3 unchanged — four-instance Pass-arc inheritance)
  - §4.1 Surface #1 M5 Ops Layer composite (§4.1.0 honest recon corrections + §4.1.a 46-slot classification + §4.1.b Pass 1 additive-API Option β + §4.1.c serde-transparent wire-neutrality) — **LOAD-BEARING for §3 + checkpoint #1**
  - §4.2 Format-boundary preservation Option γ split (D-NNN-format-boundary STAYS OPEN)
  - §4.3 CLI arg parsing Option α (clap stays String; 16 identifier-shaped Args slots) — **LOAD-BEARING for §4 + checkpoint #1**
  - §4.4 Doc-vs-code commit-shape Option γ hybrid split (runbook commit-sequence pre-frame 8-9 commits)
  - §4.5 Async-spawn captures Option γ honest framing closure (D-NNN-ε CLOSED; 7 sites at Surface #4 + #6) — **LOAD-BEARING for checkpoint #1**
  - §5 Layered-B3 expected null at full eight-surface scope (four-instance Pass-arc no-finding chain)
  - §6.1 Historical-pointer (Shape α, pointer-style; J-NNN placeholder freeze at runbook close)
  - §7 Discipline notes five sub-sections

### §13.2 Pass-arc predecessor runbooks

- `tasks/XGID_RETROFIT_PASS_1_IMPL.md` COMPLETED v2.1 at J-122 (six-commit base; Pass 1 closed with one recurrence at J-121 hygiene atom).
- `tasks/XGID_RETROFIT_PASS_2_IMPL.md` COMPLETED v1.1 at J-126 (three-commit base; Pass 2 closed with zero recurrences — first project milestone since the framework was named).
- `tasks/XGID_RETROFIT_PASS_3_IMPL.md` COMPLETED v1.6 at J-138 (four-commit base including Commit 2a split; Pass 3 closed with two recurrences at J-129 + J-134 both prospective catches at canonical-record-amendment layer).

### §13.3 Sibling-shape trilogy precedent

- `tasks/FEDERATION_TOPOSORT_IMPL.md` COMPLETED v1.2 at J-101 (trilogy precedent at ~93 KB).
- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` COMPLETED v1.2 at J-108 (trilogy precedent at ~95 KB).
- `tasks/FEDERATION_PROPAGATION_PHASE_9_COMMIT_3B_4_IMPL.md` COMPLETED v1.2 at J-119 (trilogy precedent at ~57.5 KB).

### §13.4 Cross-cutting principles applied at Pass 4

- **Rule 0** (CLAUDE.md) — mandatory session-open reading sequence; Clair reads CLAUDE PLAY block + JOURNAL latest entry + ACTIVE HANDOFF notes before runbook §3 Commit 1.
- **Rule 5** (CLAUDE.md) — never invent numbers; recon estimates are pre-walk shape-grounding, not verbatim authoritative; expect single-digit drift at walk-time per design doc §7.1 honest recon corrections precedent.
- **D-065** honest-behaviour-over-polite-behaviour at all framing decisions.
- **D-067** no-drift-surface code-organisation at Surface #1 atomic three-layer retype (field types + helper signatures + public-API parameters).
- **D-069** audit-vs-design boundary for D-NNN-format-boundary promotion-watch state.
- **D-071** audit-precedes-dependent-design for future-removal arcs (`FromStr` flavour wrapper validation at design walk; `validate_steps_8_13` + `accept_event` Pass 2 §4.2 Q5.b deprecation deferred arcs).
- **D-074** atomic-commit discipline at all commits in this runbook (thirty-eighth+ instance at J-141 runbook-shipping + per-commit increments through milestone close; milestone-close tally fifteenth at Commit 8).
- **D-076 v1.1** one-principle-two-properties amend-in-place pattern (sibling-shape to §4.2 Option γ split inheritance).
- **D-077** backward-coherence cross-milestone amendment dependency + three-instance durability framing.
- **D-078** production-grounded test enumeration at Joe-lock checkpoint #1 (LOAD-BEARING D-078 application surface for Pass 4 — verbatim classification-table approval pre-Commit-1).
- **D-079** honest-framing-resolution of promotion-watches admits three shapes (promote / close-by-honest-framing / hold-open).
- **Grep guardrail scope discipline** (J-108 codification) — `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO post-staging at Commit 8.

### §13.5 Cross-Pass discipline carry-overs (load-bearing inherited at Pass 4)

Per JOURNAL J-138 Sub-section 2 enumeration (three-instance durability established at Pass 3 close):

- **Path A** — Three-instance durability across Pass 1 + Pass 2 + Pass 3 established; permanent cross-Pass discipline; Pass 4 inherits without re-lock. `cargo build --workspace` deliberately broken at Pass 5 downstream sites throughout Pass 4; restored at Pass 5 close.
- **Borrow<str> additive API** — Pass 1 Commit 4 introduced; Pass 2 + Pass 3 consumed mechanically; Pass 4 inherits at all HashMap lookup sites + Surface #1 §4.1.b additive-API extension as second-instance Pass-arc carry-over.
- **Layered-B3 expected-null** — Three-instance no-finding chain at Pass 3 close; Pass 4 design-phase expectation null at fourth instance; runbook-phase audit verifies per Rule 5.
- **Pass-internal-consistency framing over trilogy-internal-consistency** — Pass 2 §7.7 + Pass 3 §7.2 establish precedent; Pass 4 inherits at §12.1 + §1.2 precedent-departure self-defense layer.
- **Pre-locked contingent-split posture** — Pass 2 §7.3 + Pass 3 §5.1 establish criterion + runbook-authoring shape; Pass 4 §10 + checkpoint #3 inherit as default per JOURNAL J-138 Sub-section 2.

---

## §14 Footer — Authoring provenance

### §14.1 J-141 v1.0 authoring provenance (original)

Runbook authored at J-141 (2026-05-28) by Chat Claude with Joe at design-close-plus-one session per Pass 2 J-124 + Pass 3 J-128 design-then-runbook precedent. Sibling-in-shape to `tasks/XGID_RETROFIT_PASS_3_IMPL.md` COMPLETED v1.6 with structural extensions for Pass 4's per-surface-commit Option γ hybrid-split commit-sequence:

1. **Per-Commit sections §3-§9** (one per xgen-client surface) vs Pass 3's single §4 Commit 2 seven-surface atomic — design doc §4.4 Option γ hybrid-split locks per-surface code+doc atomic shape; runbook §3-§9 inherit the per-surface enumeration with shared shape (scope + files + per-surface tests + verification + layered-B3 audit).
2. **§12 discipline notes six sub-sections vs Pass 3's nine** — lighter because Pass 4 inherits more cross-Pass discipline carry-overs from JOURNAL J-138 Sub-section 2 without re-derivation. Sub-section count reduction is honest framing per D-065 not corner-cutting.
3. **§2.1 commit sequence 8-9 commits vs Pass 3's 4** — per-surface code+doc atomic × 7 + Commit 7a CONTINGENT + Commit 8 close. Per design doc §4.4.c pre-frame.

Joe-locks at runbook-authoring session (J-141):
- **Option B locked-by-recommendation** for §2.1 commit-sequence shape (honest §4.4.4 application; no Commit 1 doc-pass; J-141 runbook-shipping commit IS the kickoff atomic per §4.4.b cross-surface fragments).
- **Per-surface test target +15** (verbal lock at this authoring; Joe may adjust at checkpoint #1 approval — T1-T3 at Surface #1 + T4-T5 at Surface #2 + T6-T7 at Surface #3 + T8-T9 at Surface #4 + T10-T11 at Surface #5 + T12-T13 at Surface #6 + T14-T15 at Surface #7).
- **Three Joe-lock checkpoints at §2.3** — #1 pre-Commit-1 verbatim classification-table approval (LOAD-BEARING D-078 surface; moved from Pass 3's pre-Commit-2 location to pre-Commit-1 because no Pass 4 Commit 1 doc-pass exists); #2 post-Commit-1 first-surface drift check + wire-format invariance witness verification; #3 post-Commit-7 split-trigger decision per ~50-error threshold.

D-074 application count at v1.0 runbook ship: **thirty-eighth instance** (J-127 24th → J-128 25th → J-129 26th → J-130 27th → J-131 28th → J-132 29th → J-133 30th → J-134 31st → J-135 32nd → J-136 33rd → J-137 34th → J-138 35th → J-139 36th → J-140 37th → this J-141 38th). Lock #3 per-commit cadence; not a milestone-close so milestone-close tally — fourteenth at J-138 — does NOT increment.

**Four-file atomic at this v1.0 runbook ship**:
1. This runbook NEW v1.0 ACTIVE.
2. `docs/ROADMAP.md` v1.45 → v1.46 + Past entry + Present updated for "Pass 4 implementation ACTIVE; Clair pickup at runbook §3 Commit 1".
3. `CLAUDE.md` PLAY block flip + header chain entry.
4. `JOURNAL.md` J-141 body entry per D-074 + Lock #3 per-commit cadence.

DECISIONS.md NOT amended (no new principles locked at runbook authoring; D-NNN-format-boundary stays OPEN per design doc §4.2.3; D-NNN-ε closure inherited per design doc §4.5.3).

**"Honest longer work over fast shortcuts" Pass 4 count stays at zero at runbook authoring** — sibling-shape to close-event-not-recurrence-event framing at J-128 Pass 3 runbook authoring + J-124 Pass 2 runbook authoring + J-101 / J-108 / J-122 / J-126 milestone-close framing. Runbook authoring is a within-milestone substantive event, not a recurrence shape.

### §14.2 Next-active (post-J-141)

**Next-active for Clair**: pickup at runbook §3 Commit 1 (Surface #1 M5 Ops Layer code+doc atomic). Read CLAUDE.md PLAY block + JOURNAL J-141 entry first per Rule 0, then this runbook §1-§3 in order, then design doc `tasks/XGID_RETROFIT_PASS_4_DESIGN.md` v1.2 §4.1.a + §4.3.0 + §4.5.0 classification tables verbatim (Joe-lock checkpoint #1 requires verbatim classification-table approval before any production code touches).

**Next-active for Chat Claude**: standby until Clair's Commit 1 closes affirmatively at Joe-lock checkpoint #2; parallel-eligible items include M6 (new) Block 4 verb-by-verb walks at `docs/xgen_node_admin_ops_design.md` §6 (~35 verbs across 7 categories) if Joe selects parallel-track work.

Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079.
