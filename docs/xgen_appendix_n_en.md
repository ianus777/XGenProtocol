# XGen Protocol — Appendix N: Auth-Module / Plugin Framework Descriptors
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-18  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

This appendix is the canonical **data-structure reference** for the module / plugin framework spine — the host-neutral descriptor and identity vocabulary every module/plugin slot shares (Storage-Engine / Plugin-Framework milestone, SE-D2). These types live in `xgen-common` so a later *client* slot inherits the exact same vocabulary as the first *node* slot for free. There is one unified handshake mechanism; the code term **`kind`** carries the system/ui distinction:

- a **module** is a *system* plugin (`host = node`) — e.g. the node storage-engine slot, an Auth Module;
- a **plugin** is a *ui* plugin (`host = client`).

**Trust posture (SE-D2, §8 security record):** the descriptor is a **const compiled into the implementation's own code** — there is no manifest file, because a compile-time framework has no host↔plugin gap to bridge. Metadata is *authoritative*, location is *never trusted*, and an unknown descriptor is **rejected loudly** at the registration site. The GUID/descriptor handshake verifies *declaration honesty*, never *behaviour*.

**Division of labour across documents:**
- **This appendix (N)** — the framework descriptor/identity types and the durability ladder.
- **Appendix L** — the EventStore storage context: how a selected engine's `AssuranceClass` gates node start-up (the SE-D4 tier→engine gate) and how `Descriptor` registers the per-Space store.
- **Ch3 / Ch4** — the normative module-registration and conformance rules.
- **Appendix M §M.3–§M.6** — the *Auth-Module policy* descriptors (`ModulePolicy` / `ModuleKind`) carried in a Trust Assertion; distinct from the *framework* descriptor here.

**Source:** `xgen-common/src/module.rs` (SE-D2; durability ladder SE-D4; M10 Auth Module reference set).

**Convention notes:**
- `ModuleKindId` and `ModuleImplId` are **UUIDv4, never an `Xgid`** (Appendix J). An `Xgid` is protocol-assigned and federates; a module GUID is **local, developer-assigned, and never crosses the wire**. Conflating them would put a dev-local identifier on the federation surface.
- The **kind GUID** is generated once per slot and *copied verbatim* by every implementer (it names the slot). The **impl GUID** is generated once per implementation by its author.
- `Descriptor` is `Serialize`-only — `name` is a `&'static str` (compiled in), so it intentionally has no `Deserialize`.

---

## N.1 `ModuleKindId`

**Source:** `xgen-common/src/module.rs`  
**Spec:** SE-D2  
**Description:** Identity of a module/plugin **slot** — e.g. "the node storage-engine slot". Generated once per slot and copied verbatim by every implementer, so the host can recognise "this crate claims to fill *that* slot". `#[serde(transparent)]` — serialises as a plain UUID string.

| Field | Type | Description |
|---|---|---|
| *(tuple .0)* | `Uuid` | UUIDv4 slot identity. Const-constructible via `from_u128`. Never an `Xgid`; never federates. |

## N.2 `ModuleImplId`

**Source:** `xgen-common/src/module.rs`  
**Spec:** SE-D2  
**Description:** Identity of a single module/plugin **implementation** filling a slot. Generated once per implementation by its author (minted via `new_v4`, then pasted as a `from_u128` const). `#[serde(transparent)]` — serialises as a plain UUID string.

| Field | Type | Description |
|---|---|---|
| *(tuple .0)* | `Uuid` | UUIDv4 implementation identity. Never an `Xgid`; never federates. |

## N.3 `AssuranceClass`

**Source:** `xgen-common/src/module.rs`  
**Spec:** SE-D4  
**Description:** The durability class a storage engine advertises. An ordered ladder (`BestEffort < Durable`, so `Ord` reads as "more assurance"), kept deliberately small in v1 — extend when a second engine lands. The tier→engine gate (SE-D4) compares this to the node's asserted tier and **refuses to start** if the selected engine under-delivers. Serialises `snake_case`.

| Variant | Wire string | `max_tier()` | Description |
|---|---|---|---|
| `BestEffort` | `"best_effort"` | 1 | Don't-corrupt / don't-silently-lose, but *not* crash-proof. The vanilla in-memory + atomic-file floor (D-084). Serves tier 1 only. |
| `Durable` | `"durable"` | 4 | fsync-on-commit, crash-survivable. Required at tiers 2–4 (§L.8 conformance). Serves the full 1–4 range. |

**Helpers:** `satisfies_tier(tier)` returns `tier <= max_tier()` — the SE-D4 gate rejects start-up when it returns `false`. `label()` returns the `snake_case` wire string, used by the SE-D8 capability advert (`NodeState.storage`, Appendix I §VI / observability).

## N.4 `Descriptor`

**Source:** `xgen-common/src/module.rs`  
**Spec:** SE-D2  
**Description:** A module/plugin's self-description — a **const in the implementation's own code**. No manifest, no file: the metadata is authoritative, the location is never trusted. `Copy` and const-constructible. `Serialize`-only.

| Field | Type | Description |
|---|---|---|
| `kind_id` | `ModuleKindId` | The slot this implementation claims to fill (copied verbatim from the slot's GUID). See §N.1. |
| `impl_id` | `ModuleImplId` | This implementation's own GUID. See §N.2. |
| `name` | `&'static str` | Human-readable implementation name (compiled in), e.g. `"xgen-store-sqlite"`. |
| `assurance` | `AssuranceClass` | The durability class this implementation advertises. See §N.3. |

---

*End of Appendix N*
