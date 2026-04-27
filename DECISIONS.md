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

## D-002 — Layer 1: Keypair Encryption Scheme

**Date:** 2026-04-27
**Layer:** 1 — Cryptographic Foundation
**Spec reference:** 3.5.1

The spec requires keypairs to be "encrypted at rest" but does not prescribe the encryption algorithm. Chose **ChaCha20-Poly1305** (AEAD) with **Argon2id** key derivation.

- **ChaCha20-Poly1305** — modern, well-audited AEAD cipher. No timing side-channels from table lookups (unlike AES without hardware acceleration). Available in the `chacha20poly1305` crate.
- **Argon2id** — current recommended KDF for password-based key derivation (RFC 9106). Resistant to GPU and side-channel attacks. Parameters for Phase 1: m=64MB, t=3, p=1 — tuned for interactive use.
- **Phase 1 passphrase** — Local Node mode uses an empty string passphrase. The file is still encrypted (the AEAD tag still provides integrity), but without meaningful key stretching. A non-empty passphrase is supported and works correctly. Production deployments must use a strong passphrase.

File format: JSON with `version`, `algorithm`, `kdf`, `salt` (base64url, 32 bytes), `nonce` (base64url, 12 bytes), `ciphertext` (base64url, 48 bytes = 32-byte key + 16-byte AEAD tag).

---

## D-003 — Layer 1: SigningKey Generation Without rand_core Feature

**Date:** 2026-04-27
**Layer:** 1 — Cryptographic Foundation
**Spec reference:** 3.5.1

`ed25519-dalek v2` exposes `SigningKey::generate(&mut rng)` only when the `rand_core` feature flag is enabled. To avoid adding a feature flag, keypair generation uses `OsRng.fill_bytes()` to produce 32 random bytes and constructs the key with `SigningKey::from_bytes()`. This is equivalent — `SigningKey::generate` does the same internally.

---
