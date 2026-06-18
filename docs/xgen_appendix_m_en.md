# XGen Protocol — Appendix M: Trust Assertions & Auth-Tier Evidence
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

This appendix is the canonical **data-structure reference** for the Trust Assertion — the signed statement an Auth Module issues certifying that an Identity has been verified to a declared Auth Tier (§3.8.4–§3.8.5). It is the third `SignedPrimitive` in the protocol, alongside the `Event` (Appendix I §I.1) and the `NodeAnnouncement` (Appendix I §VII.1): self-certifying, with a reproducible canonical form and an Ed25519 signature checked against the `issuer` public key.

**Division of labour across documents (one source of truth per concern):**
- **This appendix (M)** — the data-structure tables: field names, wire keys, types, the canonical signed form, and the signature format.
- **Ch3 §3.8** — the normative validation rules (`validate_assertion`: trust, expiry, required-claims gating, tier resolution).
- **Appendix C** — the primitive schema and inheritance diagram (TrustAssertion as a `SignedPrimitive`).
- **Appendix I §V.1** — `IdentityRecord.trust_assertion`, which carries an assertion as an opaque `Option<Value>` once stored; the `identity.register` / `identity.record` wire messages (Appendix I §IV.1) carry it as an `object`.

**Source:** `xgen-common/src/trust_assertion.rs` (Arc E / PG-03, AE-D1..D5; module-policy descriptor M10.1-D3/D4 / Arc-I AI-D8).

**Convention notes:**
- All field names use `snake_case`.
- `null` is forbidden; absent optional fields are omitted entirely.
- Datetime values use RFC 3339 UTC: `"2026-04-26T10:06:00.000Z"`.
- The `issuer` and `identity_id` are `xgen://pubkey/ed25519:<base64url>` URIs.
- Open-namespace forward compatibility: every structure below carries a flattened `extra` map that preserves unknown keys verbatim across round-trip (mirrors `AiCapabilities.extra`, Appendix I §V.3).

---

## M.1 `TrustAssertion`

**Source:** `xgen-common/src/trust_assertion.rs`  
**Spec:** §3.8.4–§3.8.5  
**Description:** A signed Trust Assertion. The wire field set is exactly the schema below; `valid_until` (not `expires_at`) is wire-authoritative (AE-D1). The Rust field `kind` serialises as the wire key `type`.

| Field | Wire key | Type | Req/Opt | Description |
|---|---|---|---|---|
| `kind` | `type` | `String` / string | Req | Canonical discriminator — always `"trust_assertion"`. Part of the signed bytes; serde-defaulted so a payload that omits it still reproduces the signed form. |
| `tier` | `tier` | `u32` / number | Req | Auth Tier this assertion certifies (1–4). |
| `issuer` | `issuer` | `String` / string | Req | `xgen://pubkey/ed25519:<base64url>` of the issuing Auth Module. The verifying key is re-derived from this field (not from the signature string). |
| `identity_id` | `identity_id` | `String` / string | Req | `xgen://pubkey/ed25519:<base64url>` of the Identity this assertion is for. |
| `issued_at` | `issued_at` | `String` / string | Req | RFC 3339 UTC timestamp the assertion was issued. |
| `valid_until` | `valid_until` | `String` / string | Req | RFC 3339 UTC expiry timestamp (AE-D1 wire name — NOT `expires_at`). |
| `claims` | `claims` | `TrustClaims` / object | Req | Verification claims. See §M.2. |
| `signature` | `signature` | `Option<String>` / string | Opt† | `ed25519:<base64url-pubkey>:<base64url-sig>` over the canonical form. Excluded from the signed bytes. |

† `signature` is absent only while constructing an unsigned assertion. A verifier rejects an assertion with no signature (`SignatureMissing`).

**Canonical form (§3.8.5, AE-D2):**  
Compact JSON (no whitespace), object keys sorted, UTF-8, with the **Trust-Assertion field order** `type, tier, issuer, identity_id, issued_at, valid_until, claims`. `signature` is excluded by omission. The `claims` object is canonicalised recursively (every nested key sorted), so a descriptor carried at any depth under `claims.extra` (§M.3–§M.6) is covered by the signature. The machinery is `canonical::canonical_object_json` (the same helper the `NodeAnnouncement` and `identity.register` canonicalise through — NOT `canonical_event_bytes`, which carries the Event-envelope order).

**Signature format:**  
`signature = "ed25519:" + base64url(issuer_public_key) + ":" + base64url(sig_bytes)`. The signature covers the canonical bytes and is verified against the `issuer` field's key; the key embedded in the signature string is ignored, so an assertion that names one issuer but is signed by another fails verification.

**Verify scope:** `TrustAssertion::verify` checks only signature shape and Ed25519 validity. Trust (is the issuer trusted?), expiry, and required-claims gating are the Node's `validate_assertion` (ch3 §3.8.5), not this method.

---

## M.2 `TrustClaims`

**Source:** `xgen-common/src/trust_assertion.rs`  
**Spec:** §3.8.4  
**Description:** The `claims` object of a Trust Assertion. `tier_verified` is the only mandatory claim; the rest are optional and reflect what the Auth Module actually verified. Plaintext contact details are never permitted — only salted hashes propagate (Option A privacy). Unknown claim keys are preserved round-trip via the flattened `extra` map.

| Field | Wire key | Type | Req/Opt | Description |
|---|---|---|---|---|
| `tier_verified` | `tier_verified` | `bool` | Req | MANDATORY — the Auth Module certifies this Identity meets the Tier standard. |
| `email_verified` | `email_verified` | `Option<bool>` | Opt | Whether an email was verified. Omitted when absent. |
| `phone_verified` | `phone_verified` | `Option<bool>` | Opt | Whether a phone was verified. Omitted when absent. |
| `email_hash` | `email_hash` | `Option<String>` | Opt | Salted SHA-256 hash of the email (§3.8.4). Plaintext never permitted. |
| `phone_hash` | `phone_hash` | `Option<String>` | Opt | Salted SHA-256 hash of the phone. |
| `extra` | *(flattened)* | `BTreeMap<String, Value>` | Opt | Unknown claim keys, preserved round-trip (open-namespace forward compat). Flattened into the `claims` object via `#[serde(flatten)]`. Carries the reserved descriptor keys of §M.7. |

**`has_claim(key)` semantics (§3.8.5 check 7 — the Node-policy required-claims gate):** known boolean claims read their field; a hash claim is satisfied by presence; an unknown key consults `extra` (truthy when `true` or a non-empty string, falsy when `false`, `null`, or absent).

---

## M.3 `ModulePolicy`

**Source:** `xgen-common/src/trust_assertion.rs`  
**Spec:** Arc-I AI-D8 (M10.1-D3)  
**Description:** An Auth Module's declared policy, carried **under `claims.extra["module_policy"]`** (NOT a top-level `TrustAssertion` field — the canonical field set is fixed by §3.8.5; riding `claims.extra` keeps it inside the signed bytes). Forward-extensible by design: `erasability` is its *first* member, not its only one. M10.1 carries the descriptor as expression only — there is no enforcement consumer yet (default-by-tier resolution and the tier-gate read are the D3-gated consumer's responsibility).

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `erasability` | `Option<Erasability>` | Opt | The module's declared erasure/retention policy (AI-D8 first member). See §M.4. |
| `extra` | `BTreeMap<String, Value>` | Opt | Unknown module-policy members, preserved round-trip. Flattened. |

## M.4 `Erasability`

**Source:** `xgen-common/src/trust_assertion.rs`  
**Spec:** Arc-I AI-D8 / AI-D4  
**Description:** The erasability sub-descriptor. Itself forward-extensible — `retention` is the declared posture; unknown members are preserved verbatim.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `retention` | `Option<Retention>` | Opt | The declared retention posture — one of AI-D4's protocol-fixed endpoints. See §M.5. |
| `extra` | `BTreeMap<String, Value>` | Opt | Unknown erasability members, preserved round-trip. Flattened. |

## M.5 `Retention`

**Source:** `xgen-common/src/trust_assertion.rs`  
**Spec:** Arc-I AI-D4  
**Description:** The protocol-fixed erasability endpoints, bounded by the AI-D4 gradient (T1 = max-erasable, T4 = no record destruction). A T2/T3 module declares one of these. M10.1 carries the value; enforcement of the gradient (and the default for an absent descriptor) is the deferred D3-gated consumer. Serialises lowercase.

| Variant | Wire string | Description |
|---|---|---|
| `Erasable` | `"erasable"` | Content / identity-binding may be erased (T1 endpoint; module-declarable). |
| `Retained` | `"retained"` | No record destruction — legal-hold retention (T4 endpoint; module-declarable). |

## M.6 `ModuleKind`

**Source:** `xgen-common/src/trust_assertion.rs`  
**Spec:** M10.1-D4 / AI-A7  
**Description:** An Auth Module's self-declared kind, carried **under `claims.extra["module_kind"]`**. **Expression, not trust** — a Node honours a module only via its explicit `trusted_auth_modules` gate (M10-A-04); this label never grants trust. Absent (or present-but-unparseable) resolves to the default `Reference`. Serialises lowercase.

| Variant | Wire string | Description |
|---|---|---|
| `Reference` | `"reference"` | The project's reference build — the unlabelled baseline; legacy assertions with no `module_kind` are reference by definition (the confirmed default). |
| `Mock` | `"mock"` | The parameterised T2–T4 demonstrator (M10.3). |

---

## M.7 Reserved `claims.extra` keys

These keys are reserved by the protocol within the `TrustClaims.extra` map. Because they live under `claims`, they are covered by the assertion signature (§M.1 canonical form). Implementations MUST use these literal key strings.

| Constant | Wire key | Carries | Description |
|---|---|---|---|
| `TrustClaims::MODULE_KIND_KEY` | `module_kind` | `ModuleKind` (§M.6) | The signed module self-label. Read via `claims.module_kind()`; defaults to `Reference`. |
| `TrustClaims::MODULE_POLICY_KEY` | `module_policy` | `ModulePolicy` (§M.3) | The signed AI-D8 module-policy descriptor. Read via `claims.module_policy()`; `None` when absent or malformed (lenient read for this expression-only arc). |

Both must be set **before signing** so they join the canonical bytes (`set_module_kind` / `set_module_policy`).

---

## M.8 `TrustAssertionVerifyError`

**Source:** `xgen-common/src/trust_assertion.rs`  
**Description:** Failure variants from `TrustAssertion::verify` — signature-shape and Ed25519 failures only (trust/expiry/claims are out of scope here, handled by the Node's `validate_assertion`).

| Variant | Meaning |
|---|---|
| `SignatureMissing` | The `signature` field is absent. |
| `SignatureMalformed` | The signature string is not a well-formed `ed25519:<key>:<sig>`. |
| `IssuerKey(String)` | The `issuer` is not a decodable `xgen://pubkey/ed25519:` URI (carries the decode error). |
| `SignatureInvalid` | Ed25519 verification against the issuer key failed (covers tampering and wrong-signer forgery). |

---

*End of Appendix M*
