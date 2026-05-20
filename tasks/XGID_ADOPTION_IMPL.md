# XGID Adoption v1 — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-20  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

This runbook is Clair's build instructions for **XGID Adoption v1**. It tells Clair what to ship in `xgen-common`, what production-code retype to perform in Phase 7.5's federation introducer field, and how to test that the wire-format invariance promise (Ch3 §3.0.3, Appendix J §J.5) holds.

The authoritative architectural sources are:

- `DECISIONS.md` D-072 — XGID Adoption v1 (the architectural commitment)
- `DECISIONS.md` D-073 — Field-name-vs-type discipline (the composition rule)
- `docs/xgen_appendix_j_en.md` — Canonical expository document (taxonomy, construction, wire-invariance, immutability, type representation, worked rejection examples)
- `docs/xgen_ch3_specification.md` §3.0 — Terse normative section
- `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5.6 — Originating precedent for `introducer_node_id`

This runbook does not re-specify those; it points to them. Where this runbook makes implementation choices that go beyond what those documents say, the choice is recorded here and stays here.

---

## Scope and non-scope

### In scope at v1

- All six XGID flavour wrappers in `xgen-common`: `EventXgid`, `SpaceXgid`, `RoomXgid`, `TrustAssertionXgid`, `NodeXgid`, `IdentityXgid`.
- The base `Xgid` newtype in `xgen-common`.
- The `XgidLike` trait in `xgen-common`.
- Flavour-specific constructors and methods (J.6).
- Serde-transparent serialisation matching wire-format invariance 2.
- Phase 7.5's `SpaceLocalMetadata.introducer_node_id` retype from `Option<String>` to `Option<NodeXgid>` — the only production-code retype in v1.
- A wire-format invariance test suite covering serialisation, deserialisation, transparency, and roundtrip-through-protocol-payload behaviour.

### Out of scope at v1

- Retyping any other XGID-carrying field in the codebase. **Every other `String`-typed XGID field stays as `String` at v1.** The retrofit Passes 1–5 (ROADMAP.md Near future) handle them in sequence after this milestone closes.
- Changes to the federation wire format. The wire stays exactly as it is; XGID's serde-transparent design guarantees this.
- Changes to the AI control / batch JSONL wire format or to `docs/xgen_aicontrol_implementation.md` beyond the one-line pointer (a documentation task handled separately, not by Clair).
- Adding flavour-specific construction validators that parse on construction (see Commit 1 detail on `pubkey()` signature).
- Changes to any test that does not exist yet for invariance verification, beyond what this runbook prescribes.

### Honest-broadening warning

Clair will likely encounter places during Commit 2 where retyping a nearby `String` field to a typed XGID flavour seems trivial and obviously correct. **Resist the urge.** XGID Adoption v1's scope discipline is deliberately narrow: types ship in `xgen-common`, and exactly one production-code field is retyped (`introducer_node_id`). If another field obviously needs the same treatment, it belongs to a Retrofit Pass, not to v1. The reasoning is in D-072's "Staged retrofit is honest about the cost of perfection" section. Broadening v1 scope risks v1 not shipping; staged retrofit honesty is the point.

---

## Commit plan

XGID Adoption v1 ships in **two commits**:

- **Commit 1 — `xgen-common` XGID types.** Ship all six flavour wrappers, the base `Xgid` type, the `XgidLike` trait, all constructors and methods, serde-transparent impls, and the wire-format invariance test suite. Production code outside `xgen-common` is not modified.
- **Commit 2 — Phase 7.5 `introducer_node_id` retype.** Retype `SpaceLocalMetadata.introducer_node_id` from `Option<String>` to `Option<NodeXgid>`. Update call sites, persistence (de)serialisation, and the Phase 7.5 federation introducer test. No other field is touched.

The two commits are sequential, not parallel. Commit 1 lands first, gets reviewed, gets merged. Commit 2 lands after.

Phase 9 integration tests (when they resume) will use XGID types from start automatically once Commit 1 ships — no separate "Commit 3" prep is needed.

---

## Commit 1 — `xgen-common` XGID types

### What ships

**Module location.** New module `xgen-common/src/xgid/` (directory module). Clair chooses internal sub-module layout (`mod.rs`, `base.rs`, `flavours.rs`, `trait_xgid_like.rs`, etc. — pick what's clean).

**Re-exports from `xgen-common/src/lib.rs`.** All six flavour wrappers, the base `Xgid` type, the `XgidLike` trait, and any flavour-specific error types are re-exported at the crate root so downstream crates write `use xgen_common::{Xgid, NodeXgid, EventXgid, ...}`.

### Type specifications

**Base type.**

```rust
pub struct Xgid(String);
```

- Single-field tuple newtype.
- Derives: `Clone`, `Debug`, `Display` (writes the inner string), `Eq`, `PartialEq`, `Hash`, `Ord`, `PartialOrd`.
- Serde-transparent: serialises to a JSON string (the inner bytes), deserialises from a JSON string. Use `#[serde(transparent)]`.
- Constructor: `Xgid::new(s: String) -> Self` — accepts any string. No URI-grammar validation at v1 (see "What does not ship at v1" below).
- Accessor: `Xgid::as_str(&self) -> &str` for read-only access.

**Flavour wrappers (six of them).**

```rust
pub struct EventXgid(Xgid);
pub struct SpaceXgid(Xgid);
pub struct RoomXgid(Xgid);
pub struct TrustAssertionXgid(Xgid);
pub struct NodeXgid(Xgid);
pub struct IdentityXgid(Xgid);
```

Each wrapper:

- Single-field tuple newtype wrapping `Xgid`.
- Derives: `Clone`, `Debug`, `Display`, `Eq`, `PartialEq`, `Hash`, `Ord`, `PartialOrd`.
- Implements `Deref<Target = Xgid>` (read-only access to the underlying `Xgid`).
- Serde-transparent: serialises as a JSON string (delegating through `Xgid`), deserialises from a JSON string. Use `#[serde(transparent)]` on the wrapper.

**Flavour-specific constructors and methods.**

Hash-anchored flavours (Event, Space, Room, TrustAssertion):

- Each provides `from_canonical_bytes(bytes: &[u8]) -> Self` — computes SHA-256 over the bytes, encodes per URI grammar, returns the flavour-typed XGID.
- Each provides a higher-level constructor matching its construction source where it is clean to do so:
  - `EventXgid::from_event(event: &Event) -> Self` — wraps `from_canonical_bytes` over the event's canonical form.
  - `SpaceXgid::from_space_create(event: &Event) -> Self` — same, asserts the event is a `state.space_create` in debug builds.
  - `RoomXgid::from_room_create(event: &Event) -> Self` — same for `state.room_create`.
  - `TrustAssertionXgid::from_assertion(assertion: &TrustAssertion) -> Self` — same over the assertion's canonical bytes.

Principal flavours (Node, Identity):

- `NodeXgid::from_pubkey(pk: &VerifyingKey) -> Self` — encodes the public key per URI grammar, returns the typed XGID. Infallible.
- `IdentityXgid::from_pubkey(pk: &VerifyingKey) -> Self` — same.
- `NodeXgid::pubkey(&self) -> Result<VerifyingKey, XgidDecodeError>` — **parse-fallible** at v1. Returns Err if the inner string is not a valid encoding of an Ed25519 public key. Decision rationale: the base `Xgid(String)` accepts any string at v1; principal flavours cannot promise more than what the construction-source data supports. A future walkthrough may tighten this to infallible if parse-on-construction is adopted; that tightening is **not** in scope at v1.
- `IdentityXgid::pubkey(&self) -> Result<VerifyingKey, XgidDecodeError>` — same.

**XgidLike trait.**

```rust
pub trait XgidLike {
    fn as_xgid(&self) -> &Xgid;
    // Other methods as needed: as_str, display formatting, etc.
}
```

- Implemented by `Xgid` itself (returns `self`) and by all six flavour wrappers (returns the inner `Xgid` via `Deref`).
- Sparingly used: code that genuinely operates over "any XGID" without caring about flavour. Trace logging is the canonical use case. Reach for `XgidLike` only when no specific flavour would be honest at the use site.

**Error types.**

- `XgidDecodeError` — emitted by `NodeXgid::pubkey()` and `IdentityXgid::pubkey()` when the inner string cannot be parsed as an Ed25519 public key.
- Variants Clair chooses to capture distinct decode failures (wrong prefix, wrong length, malformed encoding, etc.). Keep the variant set small and orthogonal.

### What does not ship at v1

- **URI-grammar validation on construction.** `Xgid::new(s: String)` accepts any string. Validation of URI prefix, length, character class happens elsewhere or via the flavour-specific decode methods (`pubkey()` for principal flavours). Construction-time validation is a useful future tightening but its design needs its own walkthrough (e.g. what does an invalid XGID *do* — refuse to construct? construct but fail at use? construct but log?). Out of scope at v1.
- **Cross-flavour conversion.** Not provided (per J.6). A code that needs to construct, say, an `EventXgid` from a `NodeXgid`'s underlying string must do so explicitly through `Xgid` extraction — this is intentional friction.
- **Normalisation hooks, case-folding, whitespace tolerance.** Wire-format invariance 5 is strict equality at the byte level. No normalisation API.
- **Flavour-tagging on the wire.** Wire format stays minimal `string` per invariance 2. Flavour information lives in the type system and in surrounding field names, not on the wire.

### Test suite

A wire-format invariance test suite ships in Commit 1 alongside the types. Tests live in `xgen-common/tests/xgid_invariance.rs` (or equivalent). The following test names are required; others MAY be added as Clair sees fit:

- **`xgid_serializes_as_plain_string`** — constructs a base `Xgid` and a representative flavour wrapper (e.g. `NodeXgid`), serialises each via serde_json, asserts the output is a plain JSON string (`"xgen:node:..."`) with no object wrapping, no extra fields, no flavour tag.
- **`xgid_deserializes_from_plain_string`** — deserialises a plain JSON string into base `Xgid` and into each flavour wrapper, asserts the inner bytes match exactly. No leading/trailing whitespace tolerance, no quote-mark normalisation.
- **`flavour_wrapper_is_serde_transparent`** — serialises a `NodeXgid` and a `String` containing the same bytes; asserts the two JSON outputs are byte-equal. Repeat for one hash-anchored flavour (e.g. `EventXgid`).
- **`event_xgid_roundtrip_through_event_canonical_form`** — constructs a representative Event, computes the EventXgid via `from_event`, serialises the Event with the EventXgid field, deserialises, recomputes the EventXgid from the deserialised event's canonical form, asserts equality with the original. This is the canonical-form invariance (3) end-to-end.
- **`node_xgid_roundtrip_through_handshake_message`** — constructs a representative federation handshake message that carries a `NodeXgid` field, serialises, deserialises, asserts the recovered NodeXgid bytes equal the original. Uses real Phase 7.5 federation message structure or a faithful test stand-in.

Additional tests Clair MAY add:

- Equality semantics (invariance 5) — two XGIDs with byte-equal inner strings compare equal; two XGIDs with one-character difference compare unequal.
- Display vs Debug format consistency.
- `Deref` chain — accessing `.as_str()` through `NodeXgid` works without explicit unwrap.
- `from_pubkey` → `pubkey()` roundtrip on principal flavours for a representative keypair.

### Definition of Done — Commit 1

- [ ] All six flavour wrappers and base `Xgid` exist in `xgen-common` and re-export from the crate root.
- [ ] `XgidLike` trait exists and is implemented by all seven types (six flavours + base).
- [ ] Constructors specified above are implemented and tested.
- [ ] `NodeXgid::pubkey()` and `IdentityXgid::pubkey()` are parse-fallible, returning `Result<VerifyingKey, XgidDecodeError>`.
- [ ] All five required invariance tests pass.
- [ ] `cargo test -p xgen-common` is clean.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean.
- [ ] No production code outside `xgen-common` is modified.
- [ ] Commit message names the milestone: `XGID Adoption v1 — Commit 1: xgen-common types + invariance tests`.

---

## Commit 2 — Phase 7.5 `introducer_node_id` retype

### What ships

The single production-code retype at v1. `SpaceLocalMetadata.introducer_node_id` flips from `Option<String>` to `Option<NodeXgid>`. Surfaces updated:

- The `SpaceLocalMetadata` struct definition (likely in `xgen-node` or wherever the struct currently lives — Clair locates).
- Persistence: the field's (de)serialisation path. Serde-transparency on `NodeXgid` means the on-disk format is unchanged (still a string in JSON), but the type signature in code changes. Validate with a roundtrip test.
- Call sites that construct `SpaceLocalMetadata` with this field — they now wrap the string in `NodeXgid::new(Xgid::new(s))` or call a more direct constructor (Clair's preference) at the boundary where the value enters the type system.
- Call sites that read the field — they now hold a `NodeXgid` value. Where they previously passed the field to functions expecting `String`, those call sites are updated to either pass the `Xgid` via `Deref` (`&*introducer.as_xgid()`) or to update the receiving function signature if `NodeXgid` is the honest type for the receiver.
- The Phase 7.5 federation introducer test — updates to use `NodeXgid::from_pubkey(...)` or equivalent at construction sites in the test.

### Scope discipline (read this twice)

**Commit 2's `introducer_node_id` retype is the only production-code retype in v1. No other field qualifies as new-code at v1; retrofit Passes 1–5 handle the rest.**

If Clair encounters a nearby field that obviously *could* be retyped (e.g. `peer_node_id` in a federation session struct, `space_id` somewhere adjacent), **do not retype it in Commit 2**. That field belongs to Pass 3 or wherever the relevant subsystem retrofit lands. Touching it now:

- Inflates Commit 2's scope and review surface.
- Forces decisions about retype consequences that Pass 3 has the right context to make.
- Risks discovering that the "obviously simple" retype has cascading call-site fixes that don't belong in this milestone.

If a fix to `introducer_node_id` *forces* an adjacent retype because a shared function signature changes and the cascading types must be consistent, Clair flags this and pauses — that's a design question for Joe, not a unilateral broadening decision.

### Definition of Done — Commit 2

- [ ] `SpaceLocalMetadata.introducer_node_id` is `Option<NodeXgid>` everywhere it appears (struct definition, call sites, persistence path, test fixtures).
- [ ] On-disk format roundtrip test passes: a `SpaceLocalMetadata` written before this commit deserialises correctly after this commit (the field's wire/disk format is unchanged because of serde-transparency).
- [ ] Phase 7.5 federation introducer test uses `NodeXgid` types and passes.
- [ ] No other `String`-typed XGID field is retyped in this commit.
- [ ] `cargo test --workspace` is clean.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean.
- [ ] Commit message names the milestone: `XGID Adoption v1 — Commit 2: introducer_node_id retype`.

---

## Sequence and dependencies

- Commit 1 must merge before Commit 2 starts. Commit 2 depends on the `xgen-common` types existing.
- After both commits merge, the XGID Adoption v1 milestone closes. ROADMAP.md's Near future Retrofit Pass 1 entry becomes the next-available work slot.
- Phase 9 (Federation Event Propagation Phase 9) integration tests, when they resume, use the new XGID types from the start — no separate prep work needed because Commit 1 makes the types available crate-wide.

---

## Cross-references

- `DECISIONS.md` D-072 — architectural commitment
- `DECISIONS.md` D-073 — field-name-vs-type discipline (composition rule applied at every use site of an XGID type)
- `docs/xgen_appendix_j_en.md` — canonical expository document (taxonomy §J.2, construction §J.3, immutability §J.4, wire-format invariance §J.5, type representation §J.6, sub-axes §J.7, scope boundaries §J.8, rejected proposals §J.9–§J.10, adoption discipline §J.11)
- `docs/xgen_ch3_specification.md` §3.0 — normative section
- `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5.6 — originating precedent for `introducer_node_id`
- `tasks/XGID_ADOPTION_DESIGN.md` — design-phase task file (flips to COMPLETED when this implementation runbook is ready for Clair to pick up)
- `docs/ROADMAP.md` — Near future section carries the five Retrofit Passes that follow this milestone

---

*End of XGID Adoption v1 Implementation Runbook.*  
