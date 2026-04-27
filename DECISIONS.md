# XGen Protocol — Implementation Decisions

Every decision that goes beyond spec prescription is recorded here before advancing to the next layer.
Format: title, date, layer, spec reference, decision narrative.

---

## D-000 — Historic First Compile

**Date:** 2026-04-27
**Layer:** 0 (pre-implementation baseline)
**Spec reference:** —

The first successful compile of the XGen Protocol codebase. No protocol logic implemented — both `xgen-node` and `xgen-client` were pure stubs printing a placeholder line. Marked retroactively as version `0.0.0` in semantic terms: state=0 (building), section=0 (no section started), session=0.

The compile itself took seconds. However, the first two attempts froze overnight and for several hours respectively due to Google Drive file locking on build artifacts. Resolved by moving `CARGO_TARGET_DIR` to a local path (`C:/cargo-targets/XGenProtocol`) outside the synced folder.

Tagged on GitHub as `v0.1.0` (build infrastructure baseline). Real versioning — `[state].[section].[session].[build]` — begins with D-001 and the first line of Wire Format code.

---

## D-001 — Versioning Scheme

**Date:** 2026-04-27
**Layer:** 0 (pre-implementation baseline)
**Spec reference:** —

Adopted a four-component version format: `[state].[section].[session].[build]`

- **state** — 0 while building from scratch; 1 when Phase 1 + Phase 2 complete and stable
- **section** — spec section actively being implemented (1–16, mapping to sections 3.1–3.16)
- **session** — increments each work session within a section
- **build** — auto-generated at compile time as `yymmdd-hhmm`

`Cargo.toml` stores the three-part `state.section.session`. The fourth part is appended at runtime by `xgen-common::build_info`. Both binaries print the full four-part version on startup alongside the git hash and UTC build timestamp.

Section=0 reserved for pre-implementation (stubs only). Section numbering begins at 1 when the first Wire Format code is written.

---
