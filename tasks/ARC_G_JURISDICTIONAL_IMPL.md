# XGen Protocol — Arc G (Jurisdictional Namespacing, PG-04) Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Frame

Runbook for PG-04 (arc G), gated by `ARC_G_JURISDICTIONAL_AUDIT.md` v1.0 + `ARC_G_JURISDICTIONAL_DESIGN.md` v1.0 (AG-D1–D8 Joe-locked 2026-06-04, strict undeclared-denied semantic confirmed). Two commits + a doc-only close. AG-D# arc-local (D-069); promotion eval at close.

**Honesty (D-065):** dormant-but-correct. The field is real + convergence-safe from C1; the federation hook is **live but no-op** under default policy, teeth only when an operator sets `allowed_jurisdictions` (PG-13 family). Active data residency is NOT delivered (operator/Tier-2+ infra, fenced out AG-D8).

**One writer per file per commit.** Clair owns the Rust; Chat Claude holds edits to shared canonical files while Clair works. Joe pushes; Claude never pushes.

## §2 — C1: protocol half (declaration)

Goal: a Space can declare a set-once `jurisdiction`; it reads back, defaults `None`, and survives a permuted M8 rebuild. No enforcement yet.

**Steps.**
1. **`xgen-core/src/space/state.rs`** — add `pub jurisdiction: Option<String>` to `SpaceState` (doc-comment: set-once at create, AG-D1; open string AG-D2; rides M8 via `PartialEq`, AG-D3).
2. Read it in **all three constructors**: `from_space_create` (`content["jurisdiction"].as_str().map(str::to_string)`), `from_dm_space_create` and `from_dm_space_create_node` (both hard-set `None` — DM declares none, AG-D4). It is purely create-carried — **no `apply_event` arm, no `state_key_for_event` arm** (AG-D1/D3).
3. **`build_space_create_event`** — add `jurisdiction: Option<&str>` (last param); write `content["jurisdiction"]` only when `Some` (mirror the `topic` pattern). Sweep every call site to pass `None` (mechanical — tests + any prod caller; grep `build_space_create_event(`).
4. **ch3** (`docs/xgen_ch3_specification.md`) — add the optional `jurisdiction` field to the `state.space_create` content schema (CP-3: ground the exact §/anchor at the edit). Header refresh.
5. **AppC** (`docs/xgen_appendix_c_en.md`) — Space class gains `+jurisdiction: string (optional)`; update the Arc-E note region only as needed (jurisdiction stays on `AuthModule` too — AE-D5 intact). Header refresh.

**Tests (C1).** `from_space_create` with a declared jurisdiction reads it back · absent ⇒ `None` · DM (`from_dm_space_create` + `_node`) ⇒ `None` · **M8 convergence pin**: a permuted `derive_resolved` rebuild of a Space carrying `jurisdiction` yields an identical `SpaceState` (field survives, AG-D3).

**Gate (C1).** `cargo test --workspace` green (+N over 1107) · `cargo build --workspace --all-targets` 0 · `cargo clippy --workspace --lib --tests -- -D warnings` clean (default **and** `--all-features`). No DECISIONS/ROADMAP arc-state change beyond the next-active flip.

## §3 — C2: implementation half (containment hook)

Resolve **CP-1** + **CP-2** at pickup (below) before coding the compose.

**CP-1 — jurisdiction at the two `policy_permits` sites.** Inbound `xgen-node/.../app.rs:2421` (`policy_permits(policy.as_ref(), &sid)`) and outbound `.../federation_session.rs:315` (`policy_permits(store.get(peer), &space_id)`). Confirm the Space's declared `jurisdiction` is reachable at each (the node holds the derived `SpaceState` for `sid`/`space_id`). If not in scope, read it from the rehydrated/derived state and thread it. Lock the exact `policy_permits(...) && jurisdiction_permits(...)` placement.

**CP-2 — operator plumbing.** Confirm whether `admin_ops::federation_set_policy` (+ the `--aicontrol`/AI write path) deserialises `allowed_jurisdictions` for free (it is part of the `FederationPolicy` payload) or needs an explicit arg/verb. If heavy, ship field + enforcement only and defer the CLI authoring surface to the ops/UI pass (record the deferral).

**Steps.**
1. **`xgen-core/src/federation/federation_policy.rs`** — add `pub allowed_jurisdictions: Option<Vec<String>>` to `FederationPolicy` with `#[serde(skip_serializing_if = "Option::is_none", default)]` (additive, mirrors `allowed_spaces`; the `Default` stays permit-all). Add pure fn `jurisdiction_permits(policy: Option<&FederationPolicy>, space_jurisdiction: Option<&str>) -> bool` per AG-D5 (None set ⇒ true; `Some(set)` ⇒ space declares ∈ set; **undeclared ⇒ false under a restrictive set** — strict, locked). Doc-comment + truth-table tests sibling to `policy_permits`.
2. **Both enforcement sites (CP-1)** — AND-compose: `policy_permits(...) && jurisdiction_permits(policy, space_jurisdiction)`. Outbound `federation_session.rs:315`, inbound `app.rs:2421`.
3. **Operator plumbing (CP-2)** — per the pickup decision.
4. **ch3** — add the **MAY** clause authorising the hook (a Node MAY refuse to host/relay a Space outside its jurisdiction policy). Same commit as the behaviour (D-074). Header refresh.

**Tests (C2).** Unit (`jurisdiction_permits` truth table): no `allowed_jurisdictions` ⇒ permit-all no-op · `Some(["SK"])` permits an `"SK"` Space · denies a `"RU"` Space · **denies an undeclared (`None`) Space** (strict) · `Deny` mode blocks regardless (via `policy_permits`). Integration: the AND-compose denies/permits correctly at **both** inbound and outbound sites (mirror the existing `federation_policy_enforcement` two-site test). Default-policy regression: a peer with no policy federates byte-for-byte as today (prime invariant intact).

**Gate (C2).** Same green bar as C1; suite up by the C1+C2 test count.

## §4 — Close (D-074 doc-only)

1. **ch3** — the §2.2 central-aggregation **MUST-NOT** clause (promote ch1 L858's implication to normative; CP-3 ground the §/wording). Header refresh.
2. **AppC** — final reconcile if C1 left anything (jurisdiction shipped on Space; remains on AuthModule).
3. **`tasks/PROTOCOL_GAP_AUDIT.md`** — §5 tracker **PG-04 ✅ DONE** (Arc G); rollup **Open 3 / 13 · Done 9 · NO-GAP 1** (open = PG-02/05/11); §4 Wave-3 G marked DONE; Arc-G close note.
4. **`docs/ROADMAP.md`** — Present Arc-G block ⚫ CLOSED; live frontier register 9/13. Paired with CLAUDE.md PLAY (same commit).
5. **`JOURNAL.md`** — J-NNN close entry.
6. **`tasks/ARC_G_JURISDICTIONAL_{AUDIT,DESIGN,IMPL}.md`** → COMPLETED v1.1.
7. **AG-D# promotion eval** — AG-D1–D8 are "how arc G implements PG-04"; expected all arc-local (D-069). Record dormant-but-correct posture (D-065).
8. **DECISIONS.md** — no change expected (confirm at eval).

## §5 — Definition of Done

**C1.** SpaceState field + 3 constructors (DM⇒None) + builder param (+ call-site sweep) + ch3 schema + AppC field · C1 tests green · build/clippy green.

**C2.** CP-1 + CP-2 resolved + recorded · `allowed_jurisdictions` + `jurisdiction_permits` + AND-compose at both sites · ch3 MAY clause · C2 tests green · build/clippy green.

**Close.** ch3 MUST-NOT · gap-audit §5 PG-04 ✅ (3/13 open) · ROADMAP+CLAUDE · JOURNAL · task docs COMPLETED · AG-D# eval · DECISIONS confirmed.

(Per task convention, "commit pushed" is **not** a DoD item — the `Status: COMPLETED` header + Joe's push are the shipped signal.)
