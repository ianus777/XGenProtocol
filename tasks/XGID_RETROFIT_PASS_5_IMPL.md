# XGID Retrofit Pass 5 — Audit Findings + Implementation Runbook (lean)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Framing

Pass 5 is the **fifth and final** XGID Retrofit pass. Scope was reduced from 4 deferred items to **2** at Pass 4 close (J-146) and amended canonically at J-147 (design doc §2.9 + this runbook's predecessors): the workspace-build restoration + xgen-client/tests fixture sweep were satisfied by Pass 4 itself. Pass 5's two genuine items are both **projection-discipline audits, independent of compilation**:

1. **Trace-field formatter audit** — every `tracing::` call site in xgen-client: are typed-XGID fields formatted via their `Display` projection (`%`) rather than the `Debug` sigil (`?`), which would leak the flavour wrapper into log output?
2. **Debug/Display impl audit on xgen-client public types** — lock projection discipline on every explicit `Display`/`Debug` impl and every `derive(Debug)` on a public type.

This is a **lean, audit-driven pass**: no separate design doc, no separate runbook. This single document folds the audit findings (the design substance) and the Clair-facing fix (the implementation) into one. Per the Pass-arc role-split locked at J-140, the audit recon was performed in-seat (Chat Claude + Joe); the one code fix is Clair's; the milestone close is the Chat-Claude-cross-file-consistency atomic.

**Headline finding: Pass 5 is a confirm-clean pass.** The audits found exactly **one** trivial trace-formatting fix and otherwise confirmed the discipline already holds end-to-end after Pass 4. Two commits close the pass and the whole arc.

---

## §2 Audit 1 — Trace-field formatter (xgen-client)

Scope: ~60 `tracing::{trace,debug,info,warn,error}!` call sites across `ops.rs` (3), `app.rs` (20), `batch.rs` (9), `desktop.rs` (1), `session.rs` (4), `ai_service.rs` (18), `service.rs` (9), `main.rs` (2). (`ai_behavior.rs`, `pacing.rs`, `temperature.rs`, `identity.rs`, `lifecycle.rs`, `lib.rs` emit none.)

### §2.1 Finding F-1 (the only finding)

**`xgen-client/src/app.rs:2288`** — in the `ai_status` debug line:

```rust
ai_invited_by = ?inviter,
```

`inviter` is `r.ai_invited_by.as_ref()`, i.e. `Option<&IdentityXgid>` (`ops.rs:1000 pub ai_invited_by: Option<IdentityXgid>`). The `?` (Debug) sigil emits flavour-wrapper noise into the trace line — `Some(IdentityXgid(Xgid("xgen://…")))` — instead of the clean canonical string. This is the exact discipline the trace-field audit exists to catch.

**Fix (Commit A):** project to the plain string before formatting, so the field reads `Some("xgen://…")`:

```rust
ai_invited_by = ?inviter.map(IdentityXgid::as_str),
```

(Equivalent acceptable forms: `?inviter.map(|x| x.as_str())`. `%` cannot apply directly because the value is an `Option`.)

### §2.2 Everything else is clean

All other XGID-typed trace fields use the `%` (Display) sigil and project cleanly: `space_id = %space_id`, `identity_id = %auth_id`, `home_node = %node`, `event_id = %req_id`, `owner = %r.owner_id`, etc. The only `?` / `{:#}` / `{:?}` sigils elsewhere are on **non-XGID** values — `anyhow` errors (`{e:#}`), plugin names, `mention_token`, the descriptive `ai_member_role: String` stay, and a wire-response enum in an error `bail!`. No other leaks.

---

## §3 Audit 2 — Debug/Display impls on xgen-client public types

**Result: CLEAN. No code changes.**

- **Explicit impls:** exactly one in the crate — `lifecycle.rs:49 impl fmt::Display for ClientLifecycleState`. It maps each state variant to a canonical label (`"Connecting"`, `"Ready"`, …); no XGID involvement. Clean.
- **`derive(Debug)`:** ~28 across public types (`ops.rs` Result structs, `app.rs` config sections, `pacing.rs`, `temperature.rs`, `batch.rs`, `session.rs`, `lifecycle.rs`). Several carry typed-XGID fields, so their derived `Debug` reveals the flavour wrapper. This is **idiomatic and diagnostics-only**, and is consistent with §5.6 ("field name carries the role, type carries the contract") — `Debug` legitimately revealing the type is not a projection violation.
- **User-facing formatting** (all `println!`/`format!` of XGID values) uniformly uses `Display` (`{}`) or `.as_str()`/`.as_ref()` projection — never `Debug`. The existing test `format!("{}", r.space_id) == "xgen://hash/sha256:S"` (`app.rs`) locks that `Display` projects to the bare canonical string with no wrapper noise.

No discipline gap. No fix required under Audit 2.

---

## §4 Commit A — trace fix (Clair)

**Scope:** the single one-line change at `xgen-client/src/app.rs:2288` per §2.1.

**Files:** `xgen-client/src/app.rs` (1 line).

**Verification:**
- `cargo build -p xgen-client --lib` clean.
- `cargo build --workspace --all-targets` — 0 errors (Path A already closed at Pass 4; Pass 5 keeps it green).
- `cargo clippy -p xgen-client --lib --all-features -- -D warnings` clean.
- Full lib suite GREEN (expected **637**, unchanged — the fix is a formatting change inside a `tracing::debug!` field; no new test, since asserting `tracing` field rendering is not unit-testable without a subscriber harness, which is out of scope for a one-line projection fix; the Audit-2 `format!("{}", r.space_id)` test already locks Display projection at the type level).

**No Commit Aa.** No fixture sweep (audit found nothing else); no doc fragments beyond this file.

**VERIFIED 2026-05-29 (Commit A):** fix applied in the closure form `ai_invited_by = ?inviter.map(|x| x.as_str())`; `cargo build --workspace --all-targets` 0 errors; xgen-client 61 lib tests GREEN (637 workspace); `cargo clippy -p xgen-client --lib --all-features -- -D warnings` clean.

---

## §5 Commit B — Pass 5 milestone close = XGID Retrofit arc close (Chat Claude + Joe)

Closes Pass 5 **and** the entire five-pass XGID Retrofit arc (Pass 1 J-122 → Pass 2 J-126 → Pass 3 J-138 → Pass 4 J-146 → Pass 5).

**Files (milestone-close atomic per D-074):**

1. `DECISIONS.md` — **D-081 promoted** (§6 below). The wire-format-invariance principle promised at Pass 5 close per the ROADMAP Near-future entry. (Numbered **D-081**, not D-080 as first discussed at the J-147 turn: D-080 was already taken by the Node-storage EventStore decision dated 2026-05-29; the collision was caught at Commit B authoring before insertion.)
2. `tasks/XGID_RETROFIT_PASS_5_IMPL.md` — this file, Status ACTIVE → COMPLETED + §7 close-record + close-J-entry frozen.
3. `JOURNAL.md` — the Pass-5-close / arc-close body entry.
4. `CLAUDE.md` — PLAY flip: Pass 5 ACTIVE → **XGID Retrofit arc COMPLETE; standby for next-milestone selection (M6 (new) Node admin write path ready)**.
5. `docs/ROADMAP.md` — version bump + visual tree Pass 5 row 🟡 → ✅ + Near-future Pass 5 line removed + cross-cutting section gains the D-081 named principle + Past entry.

**§7.10 discipline-doc consolidation:** **SKIPPED** at Pass 5 close per Joe-lock (J-147 turn) — optimization, not correctness; available as its own future atomic if the XGID retrofit family ever extends.

**Layered-B3 at arc close:** expected null — fifth Pass-arc no-finding instance (J-122 + J-126 + J-138 + J-146 + Pass 5); the four-instance chain becomes five.

---

## §6 D-081 — statement to promote at Commit B

> **D-081 — XGID typing is wire-format and persistence-format invariant.** Retyping a `String` identifier slot to a typed XGID flavour (`EventXgid`, `SpaceXgid`, `RoomXgid`, `TrustAssertionXgid`, `NodeXgid`, `IdentityXgid`) is a pure in-memory type-discipline change. Because every flavour is `#[serde(transparent)]` over a base `Xgid(String)`, it serializes and deserializes byte-identically to the pre-retrofit `String` shape on every boundary — Node↔Node wire, Node↔Client wire, AI-control / batch JSONL, and on-disk persistence. No retrofit pass (1–5) changed a single serialized byte; the five wire-format invariance witnesses (Appendix J §J.5) plus the per-pass serde-transparent witness tests lock this. The canonical string form is the flavour's `Display` projection; `Debug` may reveal the wrapper for diagnostics. D-081 realises the XGID Adoption v1 Q4 invariance promise and is the sibling of D-076 in the wire-format discipline family (D-076 = byte-identical sender output across senders; D-081 = byte-identical across the typed/untyped boundary).

After D-081 lands, the §5.6 principle ("field name carries the role, type carries the contract") is fully realised in code across all four crates, and the Q3 "mixed discipline transitionally" clause no longer applies.

---

## §7 Discipline notes + cross-references

- **Honest finding per D-065:** Pass 5 came back confirm-clean with a single one-line fix — lighter than any prior pass — because Pass 4 retyped xgen-client cleanly and the serde-transparent flavours make `Display` projection the path of least resistance at every output site. Recorded honestly rather than manufacturing scope.
- **Audit-not-assumption per Rule 5:** both audits were run against production source (every `tracing::` site enumerated; every `Display`/`Debug` impl + `derive(Debug)` enumerated), not assumed clean from the Pass-4 close.
- **Two-commit lean shape** (Joe-lock, J-147 turn): no separate design doc or runbook; this file is both. Departs from the Pass 1–4 design→runbook→impl shape because the audit scope is two confirm-clean items, not a slot-retype surface set.
- **Cross-references:** design doc `tasks/XGID_RETROFIT_PASS_4_DESIGN.md` v1.7 §2.9 (Pass 5 scope as amended at J-147); predecessor runbooks Pass 1–4; D-076 + D-081 wire-format discipline family; Appendix J §J.5 invariance witnesses; ROADMAP Near-future Pass 5 line + cross-cutting section.

---

## §8 Footer — authoring provenance

- v1.0 (J-148-candidate, 2026-05-29) — NEW. Audit findings (Audit 1 = 1 finding F-1; Audit 2 = clean) + Commit A (Clair trace fix) + Commit B (close + D-080) + D-080 statement. Authored in-seat (Chat Claude + Joe) per the lean shape locked at the J-147 turn. Status flips ACTIVE → COMPLETED at Commit B; the close-J-entry freezes at that commit per J-108 codification.
- v1.1 (J-148, 2026-05-29) — Status ACTIVE → COMPLETED at **Pass 5 milestone close = XGID Retrofit arc close**. Commit A applied F-1 (closure form), verified green (637 / 0 build errors / clippy clean). Commit B (this commit) promoted **D-081**, flipped this doc, and closed the five-pass arc (Pass 1 J-122 → Pass 5 J-148). Close-J frozen to J-148. §7.10 discipline-doc consolidation SKIPPED per Joe-lock. Lean shape: doc landed at close (Commit B) rather than at Commit A — single COMPLETED state, no ACTIVE→COMPLETED churn, honest for a same-session confirm-clean pass. (D-080 collision caught + corrected to D-081 at Commit B authoring.)
