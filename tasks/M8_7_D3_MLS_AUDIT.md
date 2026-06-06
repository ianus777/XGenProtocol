# M8.7 — D3 MLS Operationalisation: Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & scope

D-071 Phase-0 audit for **M8.7 — D3 MLS operationalisation**: replace the Phase-2 E2E scheme with real RFC 9420 / openmls crypto behind the PG-05 🔷 INTERFACE-LOCKED Arc-H seam, plus concurrent-commit resolution.

**Activation gates (satisfied).** Arc H closed 🔷 INTERFACE-LOCKED at J-256 (Phase-2 scheme wired + server-blindness proven); Round-2 audit GO; INV-EXP closed J-298. No M8.7/MLS task docs exist yet — this is a fresh open.

**This is audit-only.** No code, no DECISIONS change. It grounds the change surface against live `main` and **frames** forks. The §5/§6 direction leans were **Joe-confirmed at J-299** as design-phase input; the **formal lock happens in the design phase**, not here.

---

## 2. As-built seam (grounded inventory)

The Phase-2 substrate (`xgen-core/src/encryption/`, D-052) presents the interface openmls will provide. Confirmed against live `main`:

- **`client_mls.rs`** — `ClientMlsGroup` (in-memory: `epoch: u64`, `epoch_secret: [u8;32]`, `members: HashSet`); `add_member`/`remove_member` → `advance_epoch` (SHA-256 chain); `derive_epoch_key` (SHA-256); `encrypt_message`/`decrypt_message` (v1, ChaCha20Poly1305); **`enc:` v2 envelope** (`encrypt_message_envelope`/`decrypt_message_envelope` — per-message random `CK` wrapped under the epoch key; AH-D1 / D-088 amendment; erasability witness `envelope_with_destroyed_key`).
- **`group.rs`** — Node-side `MlsGroupState` (opaque epoch counter + member/device sets, **no key material**) + `MlsGroupRegistry`.
- **`delivery_service.rs`** — blind DS routing (`MlsDeliveryService`, Welcome/Commit/Proposal queue) + `is_encrypted_content` (`enc:` prefix) + `handle_encrypted_content` (pass-through).
- **`key_package.rs`** — Node-side `KeyPackageStore` (≥3 pool `MIN_KEY_PACKAGE_POOL`, single-use FIFO, expiry-aware; `KeyPackageError` 5001/5002 from the §3.10.11 band). `StoredKeyPackage.mls_key_package` is an **opaque** base64url blob.

**Wiring (AH-D2/D3/D4):** `SpaceState.e2e_encryption` set-once/default-OFF; `state.mls_group_init` Node-readable genesis anchor with a per-Room state key (Appendix I — "a concurrent re-init cannot fork") → `RoomState.mls_epoch = Some(0)` (`state.rs:614`); `mls.commit` ingest advances `mls_epoch` (`apply_mls_commit`, `state.rs:825`); `mls.key_package` ingest hook populates the Node pool (`runtime.rs:636`).

**Dependency surface:** `openmls` / HPKE are **absent from every manifest** (`Cargo.toml` carries only `sha2` + `chacha20poly1305`). D3 was always parallel/timing-open (D-066).

---

## 3. Findings (MLS-A#)

- **A1 — the seam is real, the swap framing is partly optimistic.** Module prose says Phase-3 "only changes the key schedule" / "a swap of the wrap primitive." True for the *primitive* (SHA-256 epoch key → MLS key schedule; ChaCha20Poly1305 wrap → HPKE). **Not** true for the *lifecycle*: see A2.
- **A2 — Phase-2 holds an epoch counter + one symmetric secret, NOT MLS group state.** Real openmls is stateful (ratchet tree, secret tree, pending proposals/commits, signature keys). `ClientMlsGroup` has none of it. Real Welcome/Commit/Proposal **generation and processing** is **net-new**, not a swap.
- **A3 — no production MLS client drives encryption (Arc H C1 Finding 1, D-065).** Server-blindness is real + proven *in-process* only; `ops::send` live-encrypt + the client group lifecycle were explicitly deferred to D3. So the **caller wiring is also net-new**, and it is client-milestone-adjacent.
- **A4 — concurrent-commit is already self-fenced to D3 in code** (`apply_mls_commit`, `state.rs:810-824`): the applier takes the declared epoch, **single-committer happy path only**; "two members committing different epoch advances at the same frontier … the real RFC 9420 problem fenced to D3"; no `state_key_for_event` arm — under a genuine race the fold order silently decides `mls_epoch`. **This is the hard problem and the milestone's center of gravity.**
- **A5 — client-side KeyPackage generation + Credential↔XGID binding is net-new.** The Node stores opaque blobs; nothing generates real RFC 9420 KeyPackages. openmls needs a crypto provider + an MLS `Credential` — which must bind to the **XGID** (the no-anonymity identity model, D-082). New design surface.
- **A6 — group-state persistence is net-new.** `ClientMlsGroup` is in-memory; real openmls group state must be serialized + survive restart in the **client** store (Node stays blind — it must NOT hold key material). Interacts with the per-device key fan-out / multi-device arc.
- **A7 — determinism coupling (D-076).** MLS Commit/Welcome ride the DAG but are opaque to the Node; D-076 already locked canonical wire ordering partly *because of* MLS coupling (`federation_propagation_design.md` Q3.ii). Real commit ordering must respect the causal DAG — confirm no new wire-order obligation.

---

## 4. Honest scope verdict (D-065)

**M8.7 as named bundles three distinct efforts, not one swap:**

- **(S) Primitive swap** — small, genuinely behind the interface: key schedule + HPKE wrap.
- **(L) Real openmls lifecycle + client wiring** — large, net-new: group state, KeyPackage gen, Credential↔XGID, persistence, `ops::send` live-encrypt. Client-milestone-adjacent.
- **(R) Concurrent-commit resolution** — a protocol-design problem (A4), wants its own design + Joe-lock.

The "interface-locked = drop-in swap" expectation holds for **S** only. **L** and **R** are real arcs. The audit's recommendation (§6) is to partition rather than run one monolith milestone.

---

## 5. Design forks (framed; F-B direction Joe-confirmed at J-299 as a design-phase lean, formal lock in design)

- **F-A — scope partition.** One milestone vs. split S / L / R. **Joe-confirmed lean (J-299): split** — see §6.
- **F-B — concurrent-commit strategy (the core fork).**
  1. *openmls-native* — one commit per epoch wins; losers re-propose. Needs an arbiter; in a DS-mediated federated DAG, who arbitrates the winner deterministically across nodes?
  2. *DAG-arbitrated* — lift epoch advance into a conflict domain (`state_key_for_event` arm), let Layer-4 resolution pick a deterministic winner; losing committers' clients detect the loss and rebuild group state from the resolved DAG.
  3. *single-committer constraint* — designate one committer per Room (admin), sidestep the race at a capability cost; defer true concurrency.
  **Joe-confirmed lean (J-299): a HYBRID of (1)+(2), refined from the original "(2)" lean.** The home node serializes commits **as the DS** on the live path (MLS-native ordering), and the **DAG supplies the deterministic tiebreak** (the option-2 `state_key_for_event` epoch arm; D-076 wire-order determinism already present) as the convergence floor for catch-up / offline-home cases. Replicas **trust the home's resolved order and do not re-adjudicate** — the same admission-only / pairwise-trust shape J-298 locked (F-5 / D-089). **Option 3 rejected** (capability regression against the multiparty goal). Formal lock + the home-DS serialization point ground in the design phase.
- **F-C — group-state persistence shape** (client store schema; restart survival; Node-blind invariant preserved). *Belongs to the L arc (§6).*
- **F-D — KeyPackage generation + crypto provider** (`openmls_rust_crypto` vs alternative) **+ Credential↔XGID binding** (D-082). *Belongs to the L arc (§6).*
- **F-E — determinism** (A7): confirm MLS messages need no wire-order obligation beyond D-076.
- **F-F — dependency surface**: openmls pulls a heavy tree; pin + vendor policy; build/network impact (crates.io is allowed). License compatibility with GPL-2.0-or-later core.

---

## 6. Milestone shape (Joe-confirmed direction at J-299; design phase locks the specifics)

- **M8.7 proper = R + S** — concurrent-commit resolution (the F-B hybrid: home-DS serialization + the DAG `state_key_for_event` epoch tiebreak) **+** the primitive swap behind the interface, on the existing wired single-committer path. The smallest protocol-complete unit; gates correctness. **In-process proof target:** two committers at one frontier → both replicas converge on the **same** `mls_epoch` winner (no live openmls client required).
- **L (production MLS client)** — real openmls group lifecycle, **loser rollback-and-replay**, KeyPackage generation, Credential↔XGID, group-state persistence — spun **OUT** to its own arc, sequenced with the client/UI milestone (depends on multi-device + client store; A3/A5/A6/F-C/F-D).

**Rationale:** keep the protocol-design problem (R) on the node/core surface where the audit→design→Joe-lock discipline lives; keep the client crypto lifecycle (L) with the rest of the client work. S rides R cheaply. **Mirrors Arc H** (ship substrate + in-process proof, defer the production client).

**R/L seam (sharpened at J-299):** **R decides the winning commit** (core/node, deterministic, provable in-process); **L executes the loser's rollback-and-replay** (real openmls client). The boundary is "decide the winner" vs "rebuild the loser."

**Honest risk for the design coverage ledger (D-065):** proving R *fully* without a real openmls client means the in-process harness **simulates** the loser-rebuild rather than exercising it — the Arc H C1 Finding 1 analogue. Acceptable, but it **must be named** in the design, not glossed.

**Internal flow (D-069):** this audit → design (lock F-A…F-F, Joe-lock F-B + the home-DS serialization point) → runbook → Clair → close.

---

## 7. Out of scope / untouched

- PG-02 GDPR content-erasure *implementation* — downstream of D3 real crypto (cascade `D-088 → PG-05 → D3`); flagged, not scoped here.
- M9 / multiparty convergence — orthogonal (E2E crypto ≠ convergence); E2E independent of M9.
- The `enc:` v2 envelope byte layout (AH-D1) — locked; D3 swaps the wrap primitive inside it, not the layout.
- Node DS blindness invariant — preserved, never relaxed.
- The L arc (production MLS client) — split out per §6; its forks (F-C, F-D, loser-rebuild) are deferred to that arc.

---

Per D-065 (honest scope: M8.7 is three efforts, not one swap) + D-069 + D-071 + D-074. Next-active after this audit: M8.7 design phase (R + S). Not pushed — Joe pushes.
