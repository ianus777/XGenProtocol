# XGID Retrofit Pass 4 — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.3  
> Date: May 2026  
> **Last updated**: 2026-05-29  
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

1. **Surface #1** — M5 Ops Layer Result structs at `xgen-client/src/ops.rs` (design doc §2.1 + §4.1 — 49-slot classification + Pass 1 additive-API extension Option β + serde-transparent wire-neutrality)
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

- **Eight surfaces vs Pass 3's seven** — one additional surface (Surface #8 doc-tree sweep, landing atomic with the code in Commit 1).
- **~~Per-surface commits~~ → consolidated retype atomic (RE-LOCKED at J-143).** The J-141 per-surface commit sequence was withdrawn at Commit-1 prep: the single-crate + Path A reality means a per-surface commit cannot leave the lib compiling (see §2.1 re-lock note). Pass 4 now **matches Pass 3's proven seven-surface atomic shape** — all surfaces retype together in Commit 1. The per-surface-commit departure from Pass 3 is therefore withdrawn; the remaining departure is only the heaviest-doc-work-pass dimension (Surface #8).
- **No Commit 1 doc-pass** per design doc §4.4.4 — doc fragments ship atomic with their code _inside Commit 1_; ROADMAP + CLAUDE PLAY + JOURNAL bumps consolidate at milestone close (Commit 2). The J-141 runbook-shipping commit + J-142 + J-143 amendments are the doc-side atomics preceding Commit 1.

Pass-internal-consistency framing per design doc §7.7 + JOURNAL J-138 Sub-section 2 cross-Pass discipline carry-overs: after the J-143 re-lock, Pass 4's commit shape **converges on Pass 3's** (consolidated retype atomic + contingent test-fixture sweep + milestone close) rather than diverging. The trilogy-internal ~80-100 KB target band is respected; Pass 4 lands lighter than the trilogy precedent on grounds of the design doc's exhaustive §4.1 + §4.3 + §4.5 walks doing the architectural work upstream.

### §1.3 What this runbook does NOT do

- Does NOT touch xgen-node at Pass 4. Pass 3 closed it (J-138).
- Does NOT promote D-NNN-format-boundary at Pass 4 close. Per design doc §4.2.3 Option γ split: promotion-watch stays OPEN at three structurally-distinct instances across two Pass-arc (D-077 multi-Pass-arc durability not yet met). Fourth structurally-distinct instance at Pass 5 OR cross-milestone closes the gap.
- Does NOT undo D-NNN-ε closure. Per design doc §4.5.3 Option γ honest framing: D-NNN-ε CLOSED by honest framing per D-065 + D-079 (Rust language idiom not XGen-specific decision); D-NNN slot preserved for actual XGen-specific decisions.
- Does NOT add `FromStr` to flavour wrappers. Per design doc §4.3.c: deferred to future audit-design-impl arc per D-071 if protocol-level "valid XGID string format at parse-time" lock surfaces as substantive Pass-arc design decision.
- Does NOT close Pass 5 deferred items: test-fixture sweep at `xgen-client/tests/` integration tests; trace-field formatter audit; Debug + Display impl audit on xgen-client public types; `cargo build --workspace` restoration. Per design doc §2.9.
- Does NOT touch M7 (--aicontrol v1 covering both binaries) scope. Per design doc §2.9.5 + §1.3.
- Does NOT touch M6 (new) Node admin write path scope. Per design doc §2.9.5 + §1.3.
- Does NOT modify the design doc §4.1.a (49-slot classification) + §4.3.0 (16 clap Args) + §4.5.0 (7 async-spawn sites) classification tables. If Clair surfaces a structural gap mid-implementation, STOP per Rule 3 + Lock 1 Trigger (a) and surface for Joe-lock before continuing. Any deviation from the verbatim classification tables requires Joe-lock checkpoint #1 re-approval.
- Does NOT amend DECISIONS.md at Pass 4 milestone close. Two candidate D-NNN promotion-watch states (format-boundary OPEN + ε CLOSED) stay as locked at design close per D-069.

---

## §2 Sequence overview

### §2.1 Commit sequence — consolidated xgen-client retype atomic (RE-LOCKED at J-143; supersedes J-141 per-surface Option B)

> **Re-lock at J-143 (2026-05-29) — Commit-1-prep finding, before any production code.** The J-141 Option B per-surface commit sequence (Commit 1 = Surface #1 alone, …, Commit 7 = Surface #7) is **infeasible under the single-crate + Path A reality**: all seven surfaces live in one crate (`xgen_client_lib`), which compiles as a unit, and xgen-client is already broken (191 errors — grep-confirmed at J-143) because every surface consumes retyped Pass 1–3 upstream types. Surfaces #2/#4/#6/#7 errors are **independent of ops.rs**, so no per-surface commit can leave `cargo build -p xgen-client --lib` clean, and T1–T15 cannot run until the whole lib compiles. Pass 3 avoided this by shipping all seven xgen-node surfaces in **one atomic** (Commit 2, ten-file) — Pass 4 now adopts the same proven shape. Joe re-locked **"one xgen-client retype atomic (Pass-3 shape)"** at J-143. This is the **second** "honest longer work" prospective catch in Pass 4 (after J-142's §4.1.a count drift). Per-surface sections §3–§9 below are **retained as the per-surface work-guides _within_ Commit 1**, NOT as separate commits.

| Commit | Scope | Files | Atomic posture | Joe-lock checkpoint |
|--------|-------|-------|----------------|---------------------|
| 1 | **xgen-client retype atomic** — all seven surfaces (§3–§9) together in one commit: xgen-common §4.1.b additive-API + ops.rs 49-slot retype (Surface #1) + app.rs 16 clap-Args projection + format paths (Surface #2) + batch.rs (Surface #3) + desktop.rs (Surface #4) + session.rs + lifecycle.rs (Surface #5) + ai_service.rs + ai_behavior.rs + service.rs (Surface #6) + pacing.rs + temperature.rs (Surface #7) + all Surface #8 doc fragments (Appendix F + xgen_aicontrol_implementation.md + Ch6 §6.15) + per-surface tests T1–T15 in-tree. **lib-clean + 8-GREEN + T1–T15 verified HERE** (per §11.3). | many (lib + doc fragments + tests) | D-074 atomic | #1 pre-ship (closed J-142); #2 post-ship drift + T2 |
| 1a | **[CONTINGENT]** test-fixture projection sweep at `xgen-client/tests/` if checkpoint #3 fires split (> ~50 fixture errors). Was "Commit 7a" pre-J-143. | varies | D-074 atomic | #3 |
| 2 | **Milestone close** (runbook + design doc J-NNN freeze + JOURNAL J-NNN body + CLAUDE PLAY flip + ROADMAP visual tree row ✅ + Past entry). Was "Commit 8" pre-J-143. | 5-6 | D-074 atomic | — |

**Total: ~3 commits** (matches Pass 3's 4-commit shape), down from the J-141 pre-frame of 8–9. The J-141 runbook-shipping commit + the J-142 §4.1.a count-correction amendment + this J-143 commit-shape re-lock are the doc-side atomics that precede the Commit 1 code.

**Governing note for §3–§11 (J-143):** the per-section "Files in this commit" + "Verification at Commit N boundary" subsections in §3–§9 now describe **per-surface scope + checks performed _within_ the single Commit 1 atomic** — they are NOT separate commits. `cargo build -p xgen-client --lib` clean, the 8-GREEN protocol, and the full T1–T15 run all verify **at the Commit 1 atomic boundary per §11.3**, not per-surface. References to "Commit 7a" → **Commit 1a**; "Commit 8" → **Commit 2**. The doc-coupling principle (design doc §4.4 Option γ — doc fragments ship atomic with their code) is preserved: all fragments land inside Commit 1 rather than across seven commits.

### §2.2 Two split triggers (Lock 1 enumeration)

Two triggers documented at this §2.2 mirror Pass 3's pre-locked contingent-split posture per design doc §4.4.c sibling-shape inheritance. Each trigger fires Joe-lock STOP per Rule 3 + Lock 1.

- **Trigger (a)** — non-existent production contract per design doc §4.1.a + §4.3.0 + §4.5.0 verbatim classification tables. If Clair grep at Commit 1 prep (or any subsequent Commit prep) finds a named field, type, method, or async-spawn site does not exist in production code (sibling-shape to J-129 Pass 3 runbook surface-ordering drift + J-133 Q5.14 v1.3 amendment), STOP and surface for Joe-lock canonical-record amendment. **D-078 applies** — production-grounded verification at Joe-lock checkpoint #1 BEFORE any code touches. Pass 3 §7.11 discipline data point ("design-doc-grounded surface enumeration at runbook authoring") instantiates here at table-grounded-verification layer.
- **Trigger (b)** — family-boundary size split if any individual Commit 1-7 exceeds ~600 lines diff (excluding test additions + doc fragments). Family-boundary not arbitrary line count; sibling-shape to Pass 3 §2.2 Trigger (c). Per-surface commits are pre-bounded by their surface's slot count + tests + doc fragment scope; if any surface unexpectedly exceeds boundary, candidate sub-commit-split surfaces at runbook re-walk layer.

### §2.3 Three Joe-lock checkpoints

- **Checkpoint #1 — pre-Commit-1 verbatim classification-table approval.** Clair extracts the design doc §4.1.a (49-slot classification: 33 identifier retypes + 11 descriptive stays + 5 borderline slots [4 `NodeXgid` + 1 `String`]) + §4.3.0 (16 identifier-shaped clap Args slots + 5 descriptive stays + 4 transport/config stays) + §4.5.0 (7 async-spawn sites across Surface #4 + #6) verbatim and surfaces them to Joe by name. Joe approves the full table content before any production code lands. This is the LOAD-BEARING D-078 application surface for Pass 4; Trigger (a) fires here if any named field or method does not exist in production. Sibling-shape to Pass 3 checkpoint #2 (pre-Commit-2 verbatim seven-surface Q-tables) but moved to pre-Commit-1 because Pass 4 has no Commit 1 doc-pass per §1.2. **Closed affirmatively at J-142** (count drift 46→49 corrected) for §4.1.a/§4.3.0/§4.5.0. **Checkpoint-#1-equivalent for Surfaces #3/#5/#6/#7 closed affirmatively at J-144** — those surfaces had only §2.x Initial Q-anchors (no verbatim table at design close); Clair grep-enumerated them, surfaced three drift findings + three classification calls, and Joe locked the production-grounded classification now at design doc §4.6 (Trigger (a) fired three times, all corrected pre-retype).
- **Checkpoint #2 — post-Commit-1 (retype atomic) drift check + wire-format invariance witness verification.** Three drift-detection points at the consolidated atomic boundary: (1) ops.rs Result struct retypes (and the other six surfaces) landed atomically with their Appendix F / aicontrol / Ch6 doc fragments (no doc-vs-code drift surface); (2) Pass 1 additive-API extension shipped at xgen-common flavour wrappers per §4.1.b Option β (`.is_empty()` inherent + `Option<XxxXgid>::as_deref()` via std/Deref); (3) serde-transparent wire-format invariance witness test (T2 at §3.4) passes — pre-Pass-4 batch consumer reads byte-identical JSON from post-Pass-4 Result types. Joe approves before milestone close (Commit 2) begins.
- **Checkpoint #3 — post-Commit-1 (retype atomic) split-trigger decision.** When the lib compiles clean, Clair runs `cargo test -p xgen-client --tests` and reports the test-fixture error count. Joe locks single-commit (absorb the sweep into Commit 1 itself) if errors ≤ ~50, or split (Commit 1 lib-clean + **Commit 1a** sweep atomic) if errors > ~50. Sibling-shape to Pass 2 checkpoint #3 (fired at 93) + Pass 3 checkpoint #3 (fired at 638). Pre-locked contingent-split posture is durable cross-Pass discipline per JOURNAL J-138 Sub-section 2.

---

## §3 Commit 1 — Surface #1 M5 Ops Layer

### §3.1 Scope

Commit 1 ships the foundational xgen-client surface per design doc §2.1 + §4.1 dependency-order anchor. All three sub-locks atomic per drift surface uniformity (D-067):

- **§4.1.a Field retype scope** — 49 String slots across 13 Result structs + `HistoryMessage` (`OpContext` carries no `String` slots — `node_override: Option<&str>`): 33 identifier-shaped mechanical retypes per §3 governing principle (e.g. `identity_id` ×3 → `IdentityXgid`, `space_id` ×9 → `SpaceXgid`, `event_id` ×7 → `EventXgid`); 11 descriptive-string mechanical stays (e.g. `display_name`, `version`, `name`); 5 borderline slots (`home_node` ×3 + `node` ×1 → `NodeXgid`; `source: Option<String>` ×1 operator-source enum-tag stays String).
- **§4.1.b Pass 1 additive-API extension** — inherent `.is_empty()` on six flavour wrappers (`IdentityXgid`, `SpaceXgid`, `EventXgid`, `RoomXgid`, `NodeXgid`, `TrustAssertionXgid`) at xgen-common per Option β; analogous Option `.as_deref()` for `Option<XxxXgid>::as_deref()` returning `Option<&str>`-equivalent. Sibling-shape to Pass 1 Commit 4 `Borrow<str>` additive-API lock — second instance of Pass-arc additive-API extension as load-bearing carry-over.
- **§4.1.c serde-transparent wire-neutrality** — confirmed via wire-format invariance witness test (T2 below). All flavour wrappers are `#[serde(transparent)]` per Pass 1 design; Result struct retypes do not change JSON wire shape.

### §3.2 Narrow scope clarifications

**What §4.1.a retype atomic means.** All three layers retype in same commit per drift surface uniformity (D-067):
- Field types on 13 Result struct declarations + `HistoryMessage` at `xgen-client/src/ops.rs` (49 slots).
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

1. `xgen-client/src/ops.rs` — Surface #1 §4.1.a 49-slot retype atomic across 13 Result structs + HistoryMessage.
2. `xgen-common/src/xgid/flavours.rs` (or wherever flavour wrappers live) — Surface #1 §4.1.b Pass 1 additive-API extension: inherent `.is_empty()` on six flavour wrappers + Option `.as_deref()` extension. **Cross-crate atomic** — xgen-common edit ships in same commit as xgen-client edit per drift surface uniformity (D-067).
3. `xgen-client/src/ops.rs` (cont) — per-surface tests T1-T3 in-tree `#[cfg(test)] mod pass_4_commit_1_tests` block.
4. `docs/xgen_appendix_f_en.md` — Surface #8 fragment: §F.0.6 M5 ops layer Result-struct field-classification annotation per design doc §4.4.a + §4.1.a. Mechanical edit; annotate per-field typed-XGID-in-memory + String-on-wire per §4.1.c serde-transparent confirmation.
5. This runbook header chain entry recording Commit 1 landed (Status stays ACTIVE v1.0).
6. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

Five-to-six file atomic. Sibling-shape to Pass 3 Commit 2 ten-file atomic but narrower because Pass 4 Commit 1 is one-surface scope vs Pass 3's seven-surface atomic.

### §3.4 Per-surface tests (T1-T3, 3 tests target)

**Joe-lock checkpoint #1 includes per-surface test list approval by name** alongside the design doc §4.1.a + §4.3.0 + §4.5.0 verbatim classification tables. Test naming follows Pass 3 §4.7 precedent (`<surface>_<flavour>_<scenario>`).

- **T1**: `ops_result_struct_field_retype_49_slots_compile` — compile-time witness that all 49 slots per §4.1.a are classified correctly (37 retypes = 33 identifier + 4 `NodeXgid`; 12 stays = 11 descriptive + 1 `source`). Constructs each of the 13 Result struct variants + HistoryMessage with typed-XGID values at every identifier slot + String at every descriptive slot; verifies type checker accepts the construction. (Renamed from `..._46_slots_compile` at J-142 checkpoint #1 grep correction.)
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

- **Q3.1 — CORRECTED at design doc §4.6.a (J-144):** `get_dag_tips(space_id: &str)` parameter **stays `&str` + `Borrow<str>`** (Q3 Option A), NOT retyped to `SpaceXgid`. Wire-boundary dispatch-entry function reading events off the sync wire; pipe callers pass JSON-decoded `String` with no projection; sibling-shape to Surface #2 Option α. Return `Vec<String>` (event-id strings) stays `String` per §4.2 Instance B. No declaration retype at Surface #3.
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

- **T6** (renamed at J-144): `batch_get_dag_tips_space_id_stays_str_with_borrow_projection` — verifies §4.6.a lock. Constructs a pipe request with String `space_id`; verifies `get_dag_tips` accepts `&str` directly (no `SpaceXgid` projection needed at the call site) and Space-filters correctly against wire event data.
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

**Surface #5 classification CORRECTED at design doc §4.6.b (J-144).** Only ONE clean identifier retype; three drift corrections.

- **Q5.1 ClientIdentity struct — CORRECTED:** `identity_id` → `IdentityXgid` (clean). `home_node` **stays `String`** — it holds a `ws://` transport endpoint URL (passed to `connect_url`), NOT a Node XGID. (The Q5.1 prose `home_node → NodeXgid` was a drift; the typed `NodeXgid` lives only on the ops.rs Result side per §4.1.a.iii, fed by a String→typed projection at the ops construction boundary — see §4.6.e.)
- **Q5.2 SessionState struct — CORRECTED:** `home_node` stays `String` (above); `bindings`/`spaces` are empty M7-shape placeholders (`SpaceCache` is a unit struct) — **stay `String`**, M7 defines their semantics.
- **Q5.3 Lifecycle state event payloads — CORRECTED:** `lifecycle::ClientStateEvent { state, label, timestamp }` has **zero identifier slots** in production (`state` enum + descriptive `label`/`timestamp`). Nothing to retype (drift corrected).
- **Q5.4 On-disk persistence at xgen-client_state.json** — serde-transparent preserves wire format; on-disk JSON shape unchanged per §4.2 wire-shape boundary class. (`home_node` stays a String value on disk regardless.)
- **Q5.5 Idempotent ensure_* helpers — CORRECTED:** `ensure_identity(keypair_path: &Path)` has no XGID param; `ensure_connected(node_override: Option<&str>)` **stays `&str`** (transport URL, sibling to `home_node`).

### §7.2 Files in this commit (target 2-3 atomic per D-074)

1. `xgen-client/src/session.rs` — Surface #5 ClientIdentity + SessionState + ensure_* helpers retypes + per-surface tests T10-T11 in-tree.
2. `xgen-client/src/lifecycle.rs` — ClientStateEvent identifier slot retypes per Q5.3.
3. This runbook header chain entry.
4. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.

Three-to-four file atomic. **No per-surface doc fragment** — Session State internals not surfaced in doc-tree at Pass 4 scope.

### §7.3 Per-surface tests (T10-T11, 2 tests target)

- **T10** (reframed at J-144): `client_identity_identity_id_typed_home_node_stays_string` — verifies §4.6.b: `ClientIdentity.identity_id` is `IdentityXgid`; `SessionState.home_node` stays `String` (transport URL). Type-checker witness.
- **T11**: `session_state_on_disk_persistence_format_round_trip_string_at_boundary` — verifies Q5.4 xgen-client_state.json serde round-trip preserves wire-format. Serialises SessionState (with `identity_id` typed, `home_node` String) via serde_json; deserialises; verifies values reconstruct + on-disk JSON shape matches canonical pre-Pass-4 String shape (sibling-shape to T2).

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

**Surface #6 classification CORRECTED at design doc §4.6.c (J-144).** Two retypes; three drift corrections.

- **Q6.1 AiBehavior trait method signatures — CORRECTED:** `on_event(&mut self, ctx) -> Option<String>` returns reply *text* (descriptive) and takes `&EventContext` — **no identifier slots in the sig itself**. The identifier material is `EventContext.ai_identity_id: &str` → **`&IdentityXgid`** (Q4 lock; string-matching sites use `.as_str()`).
- **Q6.2 AiPacingTracker per-Space pacing key** — `AiPacingTracker.last_send_at_ms: HashMap<String, u64>` key → **`HashMap<SpaceXgid, u64>`** per D-060; `Borrow<str>` lookup. (Unchanged from prose.)
- **Q6.3 EchoPlugin reference impl — CORRECTED:** EchoPlugin has **no own identifier struct fields**; it reads `ctx.ai_identity_id` (retyped via Q6.1 above) + `ctx.event.sender` (already typed on `Event`). Nothing to retype on EchoPlugin itself.
- **Q6.4 AI mode `__HEALTH__` `operator_known` — CORRECTED:** `HealthState.operator_known: Option<(usize, usize)>` is a **numeric** (known, total) count — no string identifier to retype (drift corrected).
- **service.rs** 2 × `tokio::spawn` — no decl change (Rust `'static` idiom per §4.5).
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

- **T12** (renamed at J-144): `ai_behavior_event_context_ai_identity_id_typed` — verifies §4.6.c: `EventContext.ai_identity_id` is `&IdentityXgid`. Constructs an `EventContext` with a typed `ai_identity_id`; implements a test plugin `impl AiBehavior`; verifies `on_event` compiles and mention-detection reads via `.as_str()`.
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
- **Q7.2 temperature.rs event payload struct — RESOLVED at design doc §4.6.d (J-144):** `TemperatureUpdate.space_id → SpaceXgid`; `room_id → RoomXgid`. **`subject_id` stays `String`** (Q2 lock) — it is a union of a member `IdentityXgid` OR the non-XGID `SUBJECT_ROOM` sentinel (§6.12.3); a typed field would have to wrap a non-XGID, so D-061 spec treats it as `String`. `state` stays `String` (descriptive temperature-state label).
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
- **T15** (reframed at J-144): `temperature_update_space_room_typed_subject_id_stays_string` — verifies §4.6.d: `TemperatureUpdate.space_id`/`room_id` typed (`SpaceXgid`/`RoomXgid`); `subject_id` + `state` **stay `String`**. Constructs both a per-member update and a `SUBJECT_ROOM` update; type-checker witness pins `subject_id` as `String` (D-061 sentinel union).

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
  - §4.1 Surface #1 M5 Ops Layer composite (§4.1.0 honest recon corrections + §4.1.a 49-slot classification [grep-corrected from "46" at J-142 checkpoint #1] + §4.1.b Pass 1 additive-API Option β + §4.1.c serde-transparent wire-neutrality) — **LOAD-BEARING for §3 + checkpoint #1**
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

### §14.2 J-142 v1.0 → v1.1 amendment provenance (checkpoint #1 count correction)

Amended in place at J-142 (2026-05-29) by Clair at Commit 1 session-open, BEFORE any production code touched. Cause: Joe-lock checkpoint #1's LOAD-BEARING D-078 production-grounded verification surfaced a slot-count drift in design doc §4.1.a — `grep -cE '^\s+pub [a-z_]+: (String|Option<String>)' xgen-client/src/ops.rs` returned **49**, not the **46** recorded at design close v1.2. The classification *substance* (which field → which flavour) was 100% correct against production; only the slot-count arithmetic drifted (`identity_id` ×4 should be ×3; §4.1.a.i 31→33; §4.1.a.ii 12→11; §4.1.a.iii "3 slots" was field-name rows not slot-occurrences → 5 slots; grand total 46→49). §4.3.0 (16 clap Args) + §4.5.0 (7 async-spawn) verified clean — all named contracts exist exactly as tabled.

Joe locked: **amend design doc to 49, then code** (Track 1 canonical-record amendment as its own atomic, then Commit 1) — sibling-shape to J-129 Pass 3 runbook drift amendment + J-133/J-134 design-doc amendments. This is a prospective catch at the design-doc-grounded verification layer (checkpoint #1 = D-078 working as designed). **Pass 4 "Honest longer work over fast shortcuts" count incremented to ONE** at this catch (first recurrence; sibling-shape to Pass 3 J-129 first prospective catch).

Runbook v1.1 changes: T1 renamed `..._46_slots_compile` → `..._49_slots_compile`; all live "46-slot" / count references corrected to grep-verified figures across §1, §1.3, §2.3 checkpoint #1, §3.1, §3.2, §3.3, §3.4, §13. Five-file atomic at J-142: design doc v1.2 → v1.3 (§4.1.0 + §4.1.a corrected; chain stripped per strict `Last updated` discipline) + this runbook v1.0 → v1.1 + JOURNAL J-142 body entry + CLAUDE.md PLAY flip + ROADMAP Past entry. DECISIONS.md NOT amended (no new principle; the count fix is a Rule-5 arithmetic correction).

### §14.4 J-143 v1.1 → v1.2 amendment provenance (commit-shape re-lock)

Amended in place at J-143 (2026-05-29) by Clair at Commit 1 prep, BEFORE any production code. Cause: confirming the J-141 per-surface commit sequence against the build, `cargo build -p xgen-client --lib` reported **191 errors** (xgen-client is in the inherited Path A broken state — every surface consumes retyped Pass 1–3 upstream types). Because all seven surfaces share one crate (`xgen_client_lib`), and Surfaces #2/#4/#6/#7 errors are independent of ops.rs, **no per-surface commit can leave the lib compiling**, and T1–T15 cannot run until the whole lib compiles. The J-141 Option B per-surface commit sequence was therefore infeasible.

Joe re-locked **"one xgen-client retype atomic (Pass-3 shape)"** at J-143 — all seven surfaces retype together in Commit 1 (lib-clean + 8-GREEN + T1–T15 verified at that boundary per §11.3), then contingent test-fixture sweep (Commit 1a), then milestone close (Commit 2). Pass 4's commit shape now converges on Pass 3's proven seven-surface atomic (Commit 2 `67fb48d` ten-file) rather than diverging. Per-surface runbook sections §3–§9 are retained as per-surface work-guides within Commit 1.

Runbook v1.2 changes: §2.1 commit table replaced (per-surface → consolidated atomic) + re-lock note; §1.2 precedent-departure (per-surface departure withdrawn; converges on Pass 3); §2.3 checkpoints #2/#3 reworded to the atomic boundary; this §14.4 provenance. **Pass 4 "Honest longer work over fast shortcuts" count: ONE → TWO** — second prospective catch (after J-142's count drift), both caught before any production code. Five-file atomic at J-143: this runbook v1.1 → v1.2 + design doc v1.3 → v1.4 (§4.4 pre-frame superseded-note) + JOURNAL J-143 + CLAUDE PLAY flip + ROADMAP Past entry. DECISIONS.md NOT amended.

### §14.5 J-144 v1.2 → v1.3 amendment provenance (Surfaces #3/#5/#6/#7 classification)

Amended in place at J-144 (2026-05-29) by Clair at Commit 1 resume, BEFORE any Surfaces #3/#5/#6/#7 production retype. Cause: at design close only Surface #1 (§4.1.a) + Surface #2 (§4.3.0) + async (§4.5.0) had locked, checkpoint-#1-verbatim-approved classification tables; Surfaces #3/#5/#6/#7 carried only the §2.x **Initial Q-anchors**. Per Lock 1 Trigger (a) + D-078 (do NOT retype from prose alone), Clair grepped production to enumerate the actual slots and surfaced the enumeration to Joe for a checkpoint-#1-equivalent approval. The grep surfaced **three drift findings** (`SessionState.home_node` is a `ws://` transport URL not a Node XGID; `lifecycle::ClientStateEvent` has zero identifier slots; `HealthState.operator_known` is a numeric count) plus three genuine classification calls.

Joe locks at J-144:
- **Q1 → Option B** (this Track-1 amendment atomic first, then resume Commit 1).
- **Q2 → `subject_id` stays `String`** (D-061 sentinel union `IdentityXgid` | `SUBJECT_ROOM`).
- **Q3 → `get_dag_tips(space_id)` keeps `&str` + `Borrow<str>`** (Surface #2 Option α sibling).
- **Q4 → `EventContext.ai_identity_id` → `&IdentityXgid`** (§3 governing principle).
- **ops.rs `home_node` → keep `NodeXgid`** (no Surface #1 rework; §4.1.a.iii reasoning text corrected; flagged Pass-5 future-hygiene; the String→`NodeXgid` projection at the ops construction boundary is the §3 discipline).

This is the **third** "Honest longer work over fast shortcuts" prospective catch in Pass 4 (after J-142 §4.1.a count drift + J-143 commit-shape infeasibility), all before any of the affected production retypes. Count: TWO → **THREE**.

Runbook v1.3 changes: §5.1 Q3.1 + §7.1 Q5.1/Q5.2/Q5.3/Q5.5 + §8.1 Q6.1/Q6.3/Q6.4 + §9.1 Q7.2 corrected to the §4.6 locks; tests T6/T10/T12/T15 renamed/reframed; §2.3 checkpoint #1 gains the J-144 checkpoint-#1-equivalent closure note; this §14.5 provenance. **Six-file atomic at J-144**: design doc v1.4 → v1.5 (new §4.6 production-grounded classification + §4.1.a.iii reasoning correction + §2.x resolution pointers) + this runbook v1.2 → v1.3 + JOURNAL J-144 body entry + CLAUDE.md PLAY flip + ROADMAP Past entry + `tasks/HANDOFF_PASS_4_COMMIT_1_IN_FLIGHT.md` §3.1 updated to the locked classifications. DECISIONS.md NOT amended (production-grounded classification of milestone-internal surfaces; no new principle).

### §14.6 Next-active (post-J-144)

**Next-active for Clair**: resume **Commit 1 — the consolidated xgen-client retype atomic**. Surface #1 (ops.rs 49-slot) + the mechanical consumption projections + xgen-common §4.1.b additive-API (`is_empty` + T3 GREEN) are already in the working tree (lib-clean). Remaining per §4.6 locks: Surface #5 `ClientIdentity.identity_id → IdentityXgid`; Surface #6 `AiPacingTracker` key → `SpaceXgid` + `EventContext.ai_identity_id → &IdentityXgid`; Surface #7 pacing.rs + temperature.rs retypes (subject_id/state stay String); Surface #3 no decl retype. Then fix in-tree test modules + author T1–T15 + Surface #8 doc fragments; verify lib-clean + 8-GREEN + T1–T15 per §11.3; then checkpoint #2 (drift + T2) and checkpoint #3 (split-trigger). Design doc anchor: `tasks/XGID_RETROFIT_PASS_4_DESIGN.md` v1.5 §4.1.a + §4.3.0 + §4.5.0 + **§4.6**.

**Next-active for Chat Claude**: standby until Clair's Commit 1 closes affirmatively at checkpoint #2; parallel-eligible items include M6 (new) Block 4 verb-by-verb walks at `docs/xgen_node_admin_ops_design.md` §6 (~35 verbs across 7 categories).

Per Rule 0 + Rule 3 + Rule 6 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079.
