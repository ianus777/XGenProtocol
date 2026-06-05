# Multiparty Test S6 — Findings (M8 / Wave 3 / C5)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this records

The **C5** result of M8 Wave 3 (runbook `tasks/M8_MULTIPARTY_IMPL.md` §5 C5; design
§3 S6 row). E2E content-blindness (Arc H PG-05): encrypted fan-out to N members; the Node
never sees plaintext (M3); KeyPackage pool consume + replenish; epoch-advance on `mls.commit`.
First baseline (no historical "A"; M8-D3). B stamp: `8b14aa8` (≡ `676b9c1`).

---

## Proofs (new, `m8_s6_e2e.rs`)

| Test | What it proves | Verdict |
|---|---|---|
| `s6_e2e_space_with_n_members_keeps_node_content_blind` | E2E-on Space with **3 members**; an encrypted `message.text` is stored **opaque** by the Node (byte-identical `enc:` blob; `is_encrypted_content` true), the plaintext marker appears **nowhere** in the Node-visible event (**M3 content-blindness**), and an epoch-key-holding member recovers the plaintext while the Node cannot. | **PASS** |
| `s6_keypackage_pool_consumed_and_replenish_flagged_on_multijoin` | Seed 4 KeyPackages via `mls.key_package` ingest (the `record_key_package` hook); two (modelled) joins consume two (single-use); `available_count` drops 4→2 and `needs_replenish` flips **true** once below `MIN_KEY_PACKAGE_POOL` (3). | **PASS** |
| `s6_epoch_advances_on_single_committer_mls_commit` | `mls_group_init` sets genesis epoch 0; successive `mls.commit` events advance `RoomState.mls_epoch` 0→1→2 deterministically (opaque counter, no key material). Single-committer happy path. | **PASS** |

```
$ cargo test -p xgen-node --lib m8_s6
running 3 tests
test ...s6_keypackage_pool_consumed_and_replenish_flagged_on_multijoin ... ok
test ...s6_epoch_advances_on_single_committer_mls_commit ... ok
test ...s6_e2e_space_with_n_members_keeps_node_content_blind ... ok
test result: ok. 3 passed; 0 failed; ...
```
Full workspace after Wave 3: **1167 passed / 0 failed / 2 ignored**; clippy clean default **and**
`--all-features`.

---

## Honest live-vs-dormant boundary (Arc H, D-065)

**Live + exercised:** the `enc:` v2 envelope, the Node's content-blind validate/ingest/store
path, `SpaceState.e2e_encryption` (set-once at create), the KeyPackage store +
`record_key_package` ingest hook + `request_key_package`/`needs_replenish`, and the
`mls_group_init`/`mls.commit` epoch appliers. **Dormant / D3 (recorded, not built):** the
production MLS client driving live `ops::send` encryption, Welcome/Commit `MlsDeliveryService`
targeted routing, and the replenish-request round-trip ride the eventual production MLS client.
The member group here is the in-process `ClientMlsGroup` (fixed seed) exactly as AH-D5
specifies — content-blindness is true + checkable on the real Node ingest path regardless of
where the key lifecycle eventually lives. **PG-05 is interface-locked, not done** (J-257); the
real-crypto upgrade is the named D3 cascade (`D-088 content-erasure → PG-05 → D3`), an M9/D3
input, not an M8 build.

**Fence (do not over-claim):** this is *content*-blindness only. Metadata
(who/when/which-Space/volume) stays legible to the Node by design — not claimed.

---

## The four metrics (M8-D2)

- **M1 — Delivery.** Characterized: the encrypted message is accepted + stored (fan-out
  delivery to live member connections is the binary surface; the content-blindness invariant
  is independent of member count).
- **M2 — Convergence.** Not the focus of S6 (covered by C2/C3); the epoch counter advances
  deterministically (single-committer) — convergent by construction (no `state_key`, causal).
- **M3 — Integrity / content-blindness.** **Zero plaintext** in the Node-visible event
  (explicit assertion); opaque byte-identical store; no `ERROR`/unexpected `WARN`.
- **M4 — Latency (informational; throughput NOT measured).** In-process; no network latency.

---

## CP-4 placement

Content-blindness is a deterministic invariant — real processes add no signal to "is plaintext
in the Node store" (M8-D6) — so S6 is workspace-homed. The operator-realistic binary E2E
fan-out rides the production MLS client (D3), so there is no live-encrypt binary path to run on
B today (recorded, not a gap).

---

## Definition of Done — C5

- [x] N-member E2E Space; encrypted message stored opaque; **zero plaintext** (M3); member
  decrypts, Node cannot.
- [x] KeyPackage pool consumed + `needs_replenish` flagged below MIN on multi-join.
- [x] Epoch advances on `mls.commit` (single-committer; commit-race D3-fenced).
- [x] Live-vs-dormant boundary recorded (PG-05 interface-locked; real crypto = D3/M9 input).
- [x] M1–M4 recorded; CP-4 placement noted.
- [x] `cargo test --workspace` 1167/0/2; clippy clean both feature sets.

---

*End of MULTIPARTY_S6_findings.md — C5 complete.*
