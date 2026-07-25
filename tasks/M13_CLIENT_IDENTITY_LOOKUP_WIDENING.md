# M13 — Client Identity Lookup Widening
> **Status**: PENDING  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is

**PENDING — written, not the current task.** Filed from the M-RP-ADDRESS-BOOK arc (J-583, J-584), where the gap was found while authoring the Leg-D runbook.

`identity.get` → `identity.record` is the **only** client-facing identity payload. It carries seven fields and omits three that locked address-book rules depend on. This milestone closes that.

🔑 **THE FINDING IN ONE LINE: nodes tell each other everything; nodes tell clients seven fields.** `identity.replicate` (`xgen-core/src/wire/types.rs:719-727`) carries `identity_record: Value` — the whole stored record — **plus an explicit `update_version: u64`**. Node↔node replication has full fidelity. **This milestone does not invent a data flow; it removes a narrowing at one boundary.**

---

## §1 — Objectives

**1 — Close third-party revocation blindness.** *The reason this milestone exists.* Revocation is today enforced in exactly one place: the revoked Identity's **own** session-open (`xgen-node/src/app.rs:1539`). Every other participant sees them as valid — the lookup returns `Some(record)` for a revoked Identity exactly as for a live one (`app.rs:3538-3551`). **In a protocol founded on verified identity, a revocation only the revoked party experiences is not a revocation; it is a logout.**

**2 — Give clients version comparison.** `update_version` on `identity.record`, so a client can tell a newer record from a stale one. The node already sends this field to **peers**; withholding it from clients is what leaves §5 V2 unimplementable in the address book.

**3 — Resolve the `trust_assertion` contradiction, then honour it.** ⚠️ **Two canonical documents disagree.** Appendix M line 21 states that *"the `identity.register` / `identity.record` wire messages (Appendix I §IV.1) carry it as an `object`"*. Appendix I §IV.1 lists `trust_assertion` on `identity.register` (line 449) and **omits it from `identity.record`** (lines 480-489). Code follows Appendix I. **Appendix M is making a claim about a table it does not own.** Decide which is right, correct the loser, make the code match.

🔑 **Objectives 1 and 2 are protocol ADDITIONS; objective 3 is a CONFORMANCE and INTEGRITY fix.** Different kinds of work — keep them distinguishable inside the milestone.

**Secondary objective / completion criterion:** every seeded test in M-RP-ADDRESS-BOOK Leg D (carol V2, dave revoked, frank badge) becomes a **live** test when this lands. A reasonable definition of done is *"the Option-C seeds are replaced by wire-driven fixtures."*

---

## §2 — Surfaces

- `docs/xgen_appendix_i_en.md` §IV.1 — the `identity.record` field table
- `docs/xgen_ch3_specification.md` §3.6.7
- `xgen-core/src/wire/types.rs:455-473` — the `Record` variant
- `xgen-node/src/app.rs:3538-3551` — the lookup handler
- client consumption via `ops::identity_get` (built in M-RP-ADDRESS-BOOK Leg D Step 1)
- **federation: likely NO change** — `identity.replicate` already carries all of it. **Verify, do not assume.**

📌 **Backward compatibility has a precedent to copy:** the F17 `is_ai` pattern (`types.rs:468`) — `skip_serializing_if` when false/absent, so existing lookups stay byte-identical. The same trick applies to all three fields.

---

## §3 — Decided

🔒 **D-127 — a revoked Identity returns its record WITH `revoked` set, never `identity.not_found`.** `not_found` is reserved for **erasure**. Conflating them would lose the distinction between a compromised key and a withdrawn human. See D-127 for the full reasoning and provenance.

🔒 **`revoked_at` ships as a plain RFC-3339 timestamp** (fidelity, no disclosure judgement — Chat's call).

🔒 **`reason` does NOT go on the wire as free text.** Admin-authored, unvalidatable, readable by anyone who looks the person up, and capable of defaming a named human. **Use a closed enum:** `key_compromise` · `identity_disputed` · `administrative` · `user_request`. **Free text stays node-local for audit**, where it already lives. (Chat's call, delegated by Joe.)

---

## §4 — Open, and JOE'S

⚠️ **`trust_assertion` on the wire: floor vs card.** Joe's visit-card model (§5) supplies a better split than "whole assertion vs derived expiry": the **minimal derived facts (tier + validity window) are protocol FLOOR** — tier gating *is* channel establishment when a Space gates on tier — while the **full assertion with issuer and claims is DISCLOSURE**, belonging to the card. **Proposed, not locked.**

---

## §5 — The visit card — a shape M13 must NOT foreclose

Joe's model: an Identity publishes a card. Some fields the system requires; the rest are the holder's choice; **an Identity that shares nothing has a blank card.** Joe's test for what is mandatory: **whatever establishes the communication channel.**

Applied:

- **Floor (never optional):** `identity_id` (routing + verification) · `home_node` (routing) · `devices` (signature verification) · `revoked` (⚠️ **if revocation were a disclosure choice, revocation would be voluntary**) · `is_ai` (§3.6.10 transparency requirement, immutable at registration — cannot be a disclosure choice for the same reason)
- **Optional (the card):** `display_name` — **already `Opt` in the spec, so the protocol already agrees with Joe** · `registered_at` — provenance, not channel · everything the card later grows

⚠️ **The card is a MODEL, not a field addition — and M13 as named is a field addition.** Per-identity variable payloads, a client that can never assume any optional field is present, and a declaration format. **Keep M13 narrow**; the card layers on top. But **choose M13's field set so the card can layer rather than having to replace it.**

📌 **Tier-required claims (Joe's addition) are already half-specified** — `has_claim` is documented as *"§3.8.5 check 7 — the Node-policy required-claims gate"*, and `ModulePolicy` is *"forward-extensible by design"*. Policy locked in **D-128**: proofs by default, narrow encryption permission, and a ceiling on what a tier may demand. **M10 Auth Module Reference Set territory, NOT M13.**

---

## §6 — Not in scope

- **The `identity.update` emitter** — nothing emits it (J-576); separately filed, orthogonal.
- **The visit-card model** — its own milestone; M13 only avoids foreclosing it.
- **Tier-required claims and the disclosure ceiling** — D-128, **M10**.
- **Reputation.** Stable identity + encounter history is exactly the substrate a reputation system needs, and it would be built here by accident. ⚠️ **On a no-anonymity protocol, reputation attaches to a real, permanent, legally-real person** — mob dynamics and unappealable scores stop being annoyances and become durable harm to a named human. Ch2-weight decision. **Substrate yes; feature not now; and it must NOT shape M13's field set.**

---

## §7 — Dependency

**M-RP-ADDRESS-BOOK Leg D does not block on this** — it ships with the three wire-dependent rules driven from book-internal seeds (Option C, J-583). **M13 converts those seeds into live paths.** Order between them is Joe's; neither blocks the other.