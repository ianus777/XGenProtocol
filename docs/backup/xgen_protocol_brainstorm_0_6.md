# XGen Protocol — Brainstorm Summary
> Status: Governance, sustainability, and authentication model complete — entering Stage 2 Architecture  
> Version: 0.6  
> Date: April 2026  
> Changes from v0.5: Resolved Tensions section added (3 tensions + predefined starter modules decision), Authentication UX Principle added under Pillar 3

---

## The Name

**XGen** comes directly from **Generation X** — the pre-pensioner generation that built the early internet, lived through every major platform betrayal, and knows exactly what went wrong.

Flipped around — **XGen** — because this time they are building the solution, not being sold one.

The name has layers:
- **GenX** → the generation behind it
- **X** → extensible, open, unlimited — and "ex" as in ex-Discord, ex-Facebook, ex-Skype. Done with all of it.
- **Gen** → genesis, generate, a new beginning

GenX is also historically the **"forgotten generation"** — sandwiched between Boomers and Millennials, rarely anyone's target demographic. There's a quiet irony in that generation building the communication infrastructure everyone else eventually relies on.

> *Unassuming. Quietly excellent. Just works.*

> *"XGen Protocol was built by the generation that watched every good platform get destroyed. We're not building for the next generation — we're fixing what was broken for ours."*

---

## Origin & Motivation

Born from personal frustration with the **enshittification cycle** — platforms that start great, lock users in, then betray them for profit. Skype, Facebook/Cambridge Analytica, Discord — all followed the same pattern. The root cause is always **centralization + investor pressure.**

The enshittification cycle:
1. Platform is good → attracts users
2. Users are locked in → platform extracts value from them
3. Platform degrades for users → to serve business/advertisers
4. Platform dies or becomes irrelevant

**The goal:** Communication infrastructure that is structurally incapable of betraying its users.

---

## What It Is

Not a Discord clone. Not a better app. A **foundational open protocol** for real-time communication — the HTTP of community/chat/voice/video. Around which free, community-built applications naturally appear.

> *"The socket standard, not the appliance."*

> *"XGen is infrastructure. Like roads. Nobody owns the road. Anyone can drive."*

No one owns TCP/IP. No one owns HTTP. No one owns SMTP. Billions of clients exist for all of them — some excellent, some terrible, some forgotten. The protocol doesn't care. It endures.

XGen joins that lineage. **The protocol is the permanent thing. The clients are temporary expressions of it.**

### Core Capabilities the Protocol Must Handle

- Real-time text chat
- Voice and video calls
- Simultaneous streaming
- Large file transfer
- Community organization
- All wrapped in a **universal lightweight API**

---

## Five Philosophical Pillars

### 1. Open & Federated
- No single owner, no single server
- Anyone can build a client
- Anyone can run a node
- No central database to hack, sell, or subpoena
- Community governed

### 2. Verified Identity — No Anonymity
- Every user cryptographically identified
- Identity is a **keypair, not a server address** — you own your identity, no server can take it from you
- Real person behind every account
- Consequence-free anonymity explicitly removed
- Content can be private, identity cannot
- **Tradeoff consciously accepted:** not safe for authoritarian contexts, but structurally eliminates most platform abuse

### 3. Modular Tiered Authentication
Auth is pluggable — not baked into the protocol core. The protocol only cares about the resulting trust level assertion, not how it was produced.

| Tier | Use Case | Verification Method |
|---|---|---|
| Tier 1 — Community | Gaming, hobby, friends | Email + phone |
| Tier 2 — Professional | Freelancers, business | Government ID + business registration |
| Tier 3 — Corporate | Internal company comms | PKI certificates, IT managed, auditable |
| Tier 4 — Government | Agencies, healthcare, legal | National eID, FIDO2, hardware keys |

Compatible with existing standards: **eIDAS, NIST IAL, ISO 29115**

Certification happens at the **module level**, not the protocol level. Each jurisdiction certifies their own auth module. They all plug into the same protocol.

**Authentication UX Principle — Progressive, User-Initiated:**
Every user authenticates at Tier 1 on installation — this is the baseline, no exceptions. If a user wants to access a space requiring a higher tier, the system prompts them to upgrade their own authentication to the required level. Nobody is pre-authenticated above their actual verified level. Nobody can be granted access they haven't personally earned.

Implications:
- **No proxy trust** — an admin cannot grant Tier 3 access to a Tier 1 user. The user must verify themselves.
- **Gradual onboarding** — most users never need to go beyond Tier 1. Higher tiers activate only when genuinely needed.
- **Consistent with no-anonymity pillar** — identity claims are always personal and verified, never delegated or assumed.
- **Clear UX on access denial** — when a Tier 1 user attempts to enter a Tier 3 space, the system tells them exactly what they need to do to qualify. The path is always visible.

### 4. Institutional Independence
- Never seeking government or corporate approval proactively
- When institutions build certified networks, our module plugs in naturally
- They come to us — we don't chase them
- Protocol stays fast, free, and unbureaucratic
- No vendor dependency for any institution that adopts it

### 5. Temporal Resilience — Exchange for the New One if Needed
Every component of XGen is designed to be replaced. If something better exists tomorrow, you swap the module. The protocol continues.

> *"XGen is not optimized for today's best answer. It is optimized for the ability to adopt tomorrow's better answer without breaking what was built yesterday."*

> *"XGen doesn't anticipate the future. It makes room for it."*

The protocol core is **intentionally thin** — it defines interfaces and contracts, not implementations. This is how TCP/IP survived 50 years. The interface is stable. What runs on top changes constantly.

**The natural swap surfaces — where change is most likely to be needed:**
- Encryption algorithm (e.g. Megolm → MLS → whatever comes after)
- Auth/verification method (new standards, new jurisdictions)
- Transport protocol (WebSockets → QUIC → whatever emerges)
- State resolution / federation algorithm (as scale demands improve)
- Node capability types (open enum — new types added without breaking existing ones)
- Governance rules (as the community and legal landscape evolves)

Everything else — the event model, the community primitive, the trust assertion format — is **stable by design**, because everything else depends on it.

---

## Target User

Not teenagers. Not anonymity seekers. The **pre-pensioner tech user** — roughly 45-60 years old today:
- Built the early internet, used IRC, ICQ, MSN, Skype
- Tired of being monetized and having things changed under their feet
- Values stability, ownership, and software that just works
- Remembers when platforms respected their users

> Simple enough for that generation. Powerful enough for enterprises. Open enough for developers.

---

## Architecture Concept (Layered)

```
┌─────────────────────────────────────────┐
│           APPLICATION LAYER             │
│  (Discord-like client, corporate app,   │
│   government portal, mobile app...)     │
├─────────────────────────────────────────┤
│           AUTH MODULE LAYER             │
│  (pluggable: eID, PKI, FIDO2, OAuth...) │
│  default vanilla format + special       │
│  institutional formats as needed        │
├─────────────────────────────────────────┤
│         TRUST ASSERTION LAYER           │
│  (standardized trust level claims)      │
├─────────────────────────────────────────┤
│            PROTOCOL CORE                │
│  (messaging, voice, video, files —      │
│   identity-aware but auth-agnostic)     │
├─────────────────────────────────────────┤
│           TRANSPORT LAYER               │
│  (WebSockets, WebRTC, QUIC...)          │
└─────────────────────────────────────────┘
```

Every layer defines a **stable interface downward** and accepts **swappable implementations upward.** No layer is allowed to assume the implementation details of the layer below it.

---

## Authentication Model — User-Side Identity

### Identity Lives With the User

Authentication tier is not a property of the server or community — it is a property of the **user's local installation**. At setup, the user authenticates once at their chosen tier. That credential travels with them across the network.

This inverts the typical model:
- **Typical model:** each service demands its own re-verification, fragmenting identity across platforms
- **XGen model:** user holds one portable verified credential; spaces simply declare a minimum tier requirement

When a user attempts to enter a space requiring a higher tier than they currently hold, the protocol prompts them to upgrade their own credential — the space does not manage identity, it only checks the trust assertion.

### Tiers Are Cumulative

Tiers are hierarchical and cumulative. A Tier 3 user implicitly satisfies Tier 1 and Tier 2 requirements — you cannot be corporate-verified without first being personally and professionally verified. Higher tiers absorb lower ones.

### Default Vanilla Format

The protocol ships with a **standard default credential format** covering all tiers. This works for the vast majority of deployments and requires no additional configuration. A user installs, authenticates at their tier, receives a credential in the default format, and participates in any space using that standard.

### Special-Format Spaces & Module-Specific Credentials

Some institutional deployments — government agencies, hospitals, corporate IT environments — will operate their own auth module with its own credential format and enrollment requirements. These modules may require:
- Their own login and password
- Their own enrollment process
- Their own credential structure (e.g. PKI certificate chains, national eID schemas)

The protocol does not attempt to normalize these formats. It provides the **slot** — the module handles everything inside it.

**Key principle: the protocol owns the trust assertion, not the credential format.** A module may use any internal structure it requires. What it returns to the protocol is always the same: a standardized trust level claim.

### User Identity in Practice

A user may therefore hold:
- **One primary XGen credential** — vanilla format, always present, works everywhere
- **One or more module-specific credentials** — enrolled on demand when entering special-format spaces, each anchored cryptographically to the same root identity

This mirrors the well-established enterprise SSO pattern — made open and explicit at the protocol level rather than proprietary and locked in.

---

## Node Architecture — One Type, Many Capabilities

### The Core Idea

XGen has **one node type.** Not a homeserver, an identity server, a media server, and a bridge server running separate software stacks. One node. One codebase. One install.

What a node *does* is determined by a `capabilities` field — an enumerated list of functions that node performs and advertises to the network. All applications built on XGen read this field and behave accordingly.

> *"Same software. Same protocol. Capabilities determine behavior."*

This comes directly from a need for simplicity. A protocol that requires multiple specialized server types to operate creates maintenance burden, kills self-hosting momentum, and introduces hidden hierarchy. XGen avoids all of this by design.

### Capability Advertisement

Every node announces itself:

```
node {
  id:           "xgen://node.someplace.com"
  capabilities: [messaging, identity, federation, file_storage]
  capacity:     medium
  auth_tiers:   [1, 2]
  media_relay:  false
  jurisdiction: "EU"
}
```

Other nodes and clients read this announcement and know exactly what to expect. No guessing. No configuration negotiation. The node tells the network what it is.

### Current Capability Enum

| Capability | Function |
|---|---|
| `messaging` | Stores and relays text messages and events |
| `identity` | Manages user identity and cryptographic keypairs |
| `federation` | Routes events and state between nodes |
| `media_relay` | Voice/video TURN relay for real-time calls |
| `file_storage` | Large file hosting and transfer |
| `auth_tier_1` | Handles Tier 1 community verification |
| `auth_tier_2` | Handles Tier 2 professional verification |
| `auth_tier_3` | Handles Tier 3 corporate PKI verification |
| `auth_tier_4` | Handles Tier 4 government eID verification |
| `bridge` | Connects XGen to external networks |
| `gateway` | Client entry point and connection management |

### The Open Enum Principle

**The capability list is intentionally open-ended.**

Today the enum contains the capabilities we can anticipate. Tomorrow someone invents a use case nobody anticipated — an AI agent node, a legal notarization node, a reputation scoring node, a healthcare records relay. That becomes a new enum value.

Old nodes ignore capability values they don't understand. New nodes that speak the new capability find each other and interact. The protocol didn't break. The network didn't fork. It simply grew.

This is how HTTP added new verbs without breaking existing servers. This is how the internet extended itself for 50 years without a central authority approving each new use case.

> *"XGen doesn't anticipate the future. It makes room for it."*

### Capability Combinations by Node Size

The same software runs everywhere. Capacity — not type — determines what a node actually does:

```
Home node (Raspberry Pi)
→ capabilities: [messaging, identity, federation]
→ capacity: low
→ serves: personal use, small family

Community node (small VPS)
→ capabilities: [messaging, identity, federation, file_storage, gateway]
→ capacity: medium
→ serves: small to mid-size community

Full node (dedicated server)
→ capabilities: [all including media_relay, all auth_tiers]
→ capacity: high
→ serves: large communities, enterprises, institutions
```

No hierarchy. No privileged node types. No chokepoints to capture or monetize. The network self-organizes around declared capacity.

### One Honest Complexity

Some capability combinations carry higher trust requirements. A node declaring `identity` is trusted by others to manage cryptographic keypairs responsibly. If it is compromised, the damage is significant. The spec will need to define what trust and verification is required before a node can advertise certain capabilities — particularly `identity` and `auth_tier_3/4`.

This is a Stage 3 specification problem. The architectural principle stands.

---

## The Community Primitive

### What It Is

The **Community** is XGen's analog to the Discord server concept — a named, owner-governed collection of rooms with its own permission hierarchy and identity.

Discord got the concept right. Matrix introduced Spaces as an answer but bolted them on late, with loose room ownership and no cascading permissions. XGen makes the Community a **first-class protocol primitive** — defined in the core spec, not improvised at the application layer.

A Community in XGen is:
- A named namespace with its own cryptographic identity
- A governed collection of rooms with cascading permission model
- Portable — it can migrate between nodes without losing history or identity
- Federation-aware from day one — members from different nodes participate naturally
- Owned by its members, not by any node or server

```
"Retro Gamers" Community  [xgen://community/retrogamers]
├── # announcements
├── # general
├── 📁 Games
│   ├── # nintendo
│   ├── # sega
│   └── # pc-classics
├── 📁 Off-topic
│   ├── # random
│   └── # music
└── 🔊 Voice Lounge
```

The building metaphor stays. But nobody owns the land under the building except the community itself.

### Key Design Differences from Discord and Matrix

| Property | Discord Server | Matrix Space | XGen Community |
|---|---|---|---|
| Protocol primitive | ✗ (app layer) | Partial | ✓ (core spec) |
| Cryptographic identity | ✗ | ✗ | ✓ |
| Portable between nodes | ✗ | ✗ | ✓ |
| Cascading permissions | ✓ | ✗ | ✓ |
| Community ownership | ✗ (Discord owns it) | ✗ | ✓ |
| Federation-native | ✗ | ✓ | ✓ |

### On Threads

Discord's thread implementation is widely considered problematic — threads are neither full channels nor clean inline conversations. They exist in an awkward middle state with inconsistent notifications and no clear lifecycle.

XGen will not copy this. The thread model will be designed deliberately in Stage 2, starting from the question: *what is a thread actually for, and how should it behave in a federated, identity-verified context?* The answer will emerge from the community and room primitives naturally — not be bolted on afterward.

The naming of the Community primitive itself remains open. "Space" is taken by Matrix. "Server" carries wrong connotations. "Community", "Place", "Hub", "Home" are all candidates. The right name will become clear once the Stage 2 architecture is more defined.

---

## The Reference Client Strategy

> *"XGen is a protocol that happens to have a client. Not a client that happens to have a protocol."*

This separates XGen from everything that came before — Discord, Element, Slack, Teams. All of them are clients that bolted on protocols when they needed to. XGen is the inversion.

The reference client exists to **prove the protocol works** and to set the ceiling of what is possible — not to define what XGen is. Any client, built by anyone, must be able to participate fully in the XGen network.

The reference client is itself modular, mirroring the protocol layers:

```
┌─────────────────────────────┐
│         UI SHELL            │  ← swappable, themeable, per use case
├─────────────────────────────┤
│      FEATURE MODULES        │  ← voice, video, files, communities
├─────────────────────────────┤
│       CORE CLIENT           │  ← identity, auth, messaging — stable
├─────────────────────────────┤
│      PROTOCOL ADAPTER       │  ← the only layer touching XGen protocol
└─────────────────────────────┘
```

This means:
- A corporate deployment swaps the UI shell, keeps the core
- A government deployment adds their auth module, keeps everything else
- A minimalist client strips the feature modules, core remains intact
- The protocol adapter is the only layer that needs to speak XGen — everything above it is just software

---

## Intellectual Lineage: What XGen Takes from Matrix — and What It Deliberately Leaves Behind

Matrix (matrix.org, est. 2014) is the most relevant predecessor to XGen. It is open, federated, and genuinely anti-enshittification. A decade of real-world deployment has produced invaluable lessons. XGen stands on Matrix's shoulders — not on its codebase.

> *The difference between Homo sapiens and Homo neanderthalensis was the ability to build on the previous generation. XGen does not ignore what Matrix learned. It compounds it.*

### What XGen Takes from Matrix

| Concept | What It Is | Why XGen Keeps It |
|---|---|---|
| Event as primitive | Every message/action is a signed, immutable event | Solid foundation for federation consistency |
| Room as organizing unit | The core container for communication | Battle-tested, universally understood |
| Federation topology | Rooms distributed across independent nodes | Proven model for decentralization |
| Open client-server API | REST-based, well-documented interface | Sound convention, wide developer familiarity |
| Encryption foundation | Olm/Megolm concepts (not necessarily implementation) | Decade of real-world hardening |

### What XGen Deliberately Leaves Behind

| Matrix Decision | The Problem | XGen's Answer |
|---|---|---|
| Identity tied to server (`@user:server.tld`) | Server dies → identity gone. Users don't own themselves. | Identity is a cryptographic keypair. Server is infrastructure. You are your key. |
| No protocol-level identity verification | Anyone registers anywhere as anyone. Abuse is structural. | Verified identity is a founding protocol assumption, not an application feature. |
| Encryption retrofitted, not designed in | Years of painful UX. Cross-device verification confusion. Lost message history. | Encryption and identity verification designed together from day one. |
| Spec lagged behind implementation | Synapse *was* the spec in practice. Third parties reverse-engineered behavior. | Spec-first development. No implementation is the reference. The spec is the reference. |
| State resolution doesn't scale | Large rooms (10k+ users, 500+ servers) create severe computational overhead. | State resolution algorithm designed with scale as a primary constraint. |
| Element S.A. de facto owns Matrix | Foundation exists on paper. One company controls the roadmap in practice. | Governance structurally independent from day one. No single entity can own the roadmap. |
| Synapse too heavy to self-host | Killed grassroots node adoption. Open protocol means nothing if running a node is inaccessible. | Reference node implementation lightweight by design. Self-hosting must be accessible. |
| Auth is application-level only | No protocol-level trust differentiation. Every user looks the same to the protocol. | Modular tiered auth is a protocol primitive. Trust level is a first-class protocol concept. |
| Multiple specialized server types | Heavy operational burden. Hidden hierarchy. Kills self-hosting. | One node type. Capabilities declared. Same software everywhere. |
| Space bolted on late | Loose ownership. No cascading permissions. Not a protocol primitive. | Community is a first-class protocol primitive from day one. |

### The Strategic Position

XGen is not a Matrix fork. It is not a Matrix competitor. It is what Matrix would have been if it had been designed with verified identity, modular trust tiers, a single node type, a proper community primitive, and spec-first discipline from the beginning.

Matrix proved the federation model works. XGen takes that proof and builds the layer Matrix never built.

---

## Future Pressures XGen Is Designed to Withstand

The protocol must be ready for pressures already visible on the horizon:

**Regulatory** — EU Digital Services Act, eIDAS 2.0, and similar frameworks globally are moving toward mandatory identity verification and data residency requirements. XGen's auth tier model is already structurally ahead of this. New regulatory requirements become new auth modules — not protocol revisions.

**Quantum** — Post-quantum cryptography is no longer theoretical. NIST finalized its first post-quantum standards in 2024. XGen's encryption layer must be algorithm-agile by design — swappable when quantum-resistant algorithms become mandatory.

**AI** — Automated agents will increasingly participate in communication networks. XGen's identity model must handle non-human verified identities (a corporate AI agent with a Tier 3 identity) without breaking the human-centric trust model. The open node capability enum makes `ai_agent` a future node type that requires no protocol revision to introduce.

**Jurisdictional** — Different countries will have different legal requirements. The federation model must support jurisdictional namespacing — a government deployment must be able to enforce local data residency rules without forking the protocol.

---

## Competitive Differentiation

| | Discord | Matrix | Signal | XGen |
|---|---|---|---|---|
| Open protocol | ✗ | ✓ | ✓ | ✓ |
| Verified identity | ✗ | ✗ | ✗ | ✓ |
| Identity server-independent | ✗ | ✗ | ✗ | ✓ |
| Modular auth tiers | ✗ | ✗ | ✗ | ✓ |
| User-side portable identity | ✗ | ✗ | ✗ | ✓ |
| Spec-first development | ✗ | ✗ | ✓ | ✓ |
| Single node type | ✗ | ✗ | ✗ | ✓ |
| Community as protocol primitive | ✗ | ✗ | ✗ | ✓ |
| Corporate ready | ✗ | Partial | ✗ | ✓ |
| Government pluggable | ✗ | Partial | ✗ | ✓ |
| Community moddable | ✗ | Partial | ✗ | ✓ |
| Lightweight self-hosting | N/A | Partial | N/A | ✓ |
| No enshittification | ✗ | ✓ | ✓ | ✓ |
| No single owner | ✗ | Nominally | ✓ | ✓ |
| Open capability extensibility | ✗ | ✗ | ✗ | ✓ |

---

## Historical Parallels That Validate This Path

| Technology | Origin | Outcome |
|---|---|---|
| Linux | Hobbyist project | Now runs 90% of government servers |
| TCP/IP | Academic experiment | Became the internet |
| HTTP | CERN internal tool | Became the web |
| PostgreSQL | University project | Now in banking, government, healthcare |
| Signal protocol | Open source | Adopted by WhatsApp, Google, Microsoft |
| BitTorrent | Independent | Proved single-node-type peer networks work at massive scale |
| Matrix protocol | Independent foundation | Proved federation works. XGen builds the next layer. |

**None asked for permission first. They built something excellent. The world followed.**

---

## Governance & Sustainability Model

### The Blender Blueprint

Blender is the closest existing model to what XGen needs to become. Not because it is a communication protocol — it isn't — but because its organizational journey maps almost exactly onto XGen's risks and ambitions.

Ton Roosendaal built Blender as an in-house tool, spun it out into a commercial company (NaN), took venture capital, and watched investors shut the entire project down when the dot-com bubble burst in 2002. The code was held hostage. A community crowdfunding campaign raised €110,000 in seven weeks to buy the source code back and release it under the GNU GPL. The Blender Foundation was born from that rescue — a Dutch nonprofit that has run the project ever since. In 2025, after 23 years, Roosendaal stepped down as CEO — handing over to a team that had been growing inside the project for over a decade.

**Lessons mapped directly to XGen:**

| Blender lesson | XGen decision |
|---|---|
| VC money made the code hostage to investors | Never accept investment that could claim ownership of the protocol or the spec |
| GPL from day one made corporate capture legally impossible | XGen's license must structurally prevent capture — not just philosophically |
| Founder ran it 23 years before handing off | Succession must be planned from year one, not when burnout arrives |
| Successor grew inside the project for a decade | Community cultivation is a governance strategy, not an afterthought |
| Dutch nonprofit (Stichting) = proven, credible legal home | Netherlands is the natural legal home for XGen Foundation |
| Development Fund dominated income = dangerous fragility | No single income stream should exceed 30–40% of total revenue |

---

### Governance Structure

**Legal form:** A Dutch *Stichting* (nonprofit foundation), registered in the Netherlands. GDPR-native, well-understood by EU institutions, credible in the open source world, and structurally resistant to corporate takeover. Alternatives were investigated and ruled out: Switzerland (geopolitically neutral but outside EU grant frameworks), Estonia (philosophically aligned on digital identity but smaller ecosystem), Germany (strong ecosystem but more bureaucratic). No EU-wide nonprofit foundation statute exists or is currently proposed. Estonia noted as a potential strategic partner given its open source identity infrastructure (X-Road), not as a legal home.

**Control model — two tracks running in parallel from day one:**

*Track 1 — Benevolent Founder Control (early phase)*
A small founding board holds the protocol spec, the brand, and final decision authority. Moves fast. No committees for early decisions. This is not a weakness — it is a deliberate choice to protect quality and coherence during the most vulnerable phase.

*Track 2 — Successor Community Cultivation (from day one)*
Contributors are identified early, trusted progressively, given increasing responsibility. The council that eventually takes over is grown inside the project — not recruited from outside when crisis hits. The handoff is not an event. It is a slow, planned transfer of weight.

**Hard governance rules:**
- No single corporate contributor may hold formal governance influence
- The protocol specification is owned by the Foundation, not any individual or company
- Protocol changes require open RFC process — no closed-door modifications
- The founding board cannot sell, transfer, or license the protocol in ways that compromise its open nature

---

### Licensing & IP Mechanism

**Mechanism: BSL + CLA**

Every contributor signs a **Contributor License Agreement (CLA)** assigning copyright of their contributions to the Foundation. The Foundation then holds all IP and controls licensing during the early phase.

The code is released under a **Business Source License (BSL)** — source-available from day one, with automatic conversion to GPL written into the license itself. The conversion trigger is not a date but a project state: two independent client implementations exist and a stable RFC has been published. This means the transition to fully open is automatic and legally binding — not dependent on anyone's future goodwill or a decision that could be avoided.

This combination means:
- The small founding team controls the work legally during Stages 1–3
- Contributors know exactly what they're signing up for from day one
- The GPL transition is trustworthy and unavoidable — it cannot be quietly abandoned

---

### Sustainability Model — Five Income Streams

The core principle: **no single stream should exceed 30–40% of total income.** Blender's painful lesson is that donation dominance creates structural fragility — even in good years.

**Stream 1 — Community Donations**
Individual users and small teams contributing voluntarily. Essential for legitimacy and community connection. Structurally fragile at small scale — only reliable at large user base. Treated as a baseline, not a foundation.

**Stream 2 — Corporate Development Fund Membership**
Companies that build on or benefit from the protocol pay annual membership fees. Hard cap per contributor enforced — no single corporation may contribute more than 20% of this stream, preventing informal influence through financial dominance.

**Stream 3 — Certified Module Fees** *(unique to XGen — does not exist in Blender or any comparable open protocol)*
Organizations requiring an officially certified auth module — governments, hospitals, banks, legal firms — pay the Foundation to certify their module meets the relevant tier standard. They are not buying the protocol. They are buying the compliance stamp. This revenue stream is a direct consequence of XGen's tiered auth architecture. It is potentially the most significant and most stable stream.

**Stream 4 — Hosted Reference Infrastructure**
Running the reference node, the identity bootstrapping service, the developer sandbox environment. Not mandatory for anyone — the protocol is fully runnable without it. But convenient enough that organizations pay for managed access. Analogous to how Red Hat charged for support and services around free Linux.

**Stream 5 — Grants** *(particularly well-suited to XGen)*
EU Horizon programme grants, NGI (Next Generation Internet) funding, national digital infrastructure programmes. Europe has been actively funding open protocol work. XGen's GDPR-native design, EU-compatible architecture, and eIDAS compatibility make it a strong candidate. This stream requires grant-writing capacity on the team — a specific skill that must be represented in the first hire decisions.

---

## Resolved Tensions

These are the hard philosophical contradictions surfaced and stress-tested during Stage 1. Each has a provisional answer. They are not closed forever — but they are no longer unexamined.

---

### Tension 1 — Government Identity Demands vs. Institutional Independence

**The problem:** The "no anonymity" pillar means identities exist somewhere. A government can demand access to them. Telegram had this philosophy too — and eventually Durov partially complied with the FSB before leaving Russia entirely. What stops XGen from the same fate?

**The answer:** Federation is the structural defense, not policy. There is no master identity registry to subpoena. A government demanding records from *a node* gets only that node's records. The protocol itself holds no central list. This is not accidental — it must be an explicit, non-negotiable architectural constraint, not an emergent property.

**Implication for architecture:** The protocol spec must explicitly prohibit any design pattern that allows a central identity aggregation point to exist — even optionally.

---

### Tension 2 — The Discord Bridge as a Trust Model Collision

**The problem:** A bridge to Discord isn't just a technical connector. Discord users are unverified, anonymous-by-default, and governed by a completely different trust model. Placing that next to XGen's cryptographically verified identity layer creates a messy boundary — who is responsible for what happens at that seam?

**The answer:** The bridge is one module among many — and it does not need to be fully aligned with Discord's model. Partial compatibility with Discord's large user community is enough. The key design decision is that users crossing the bridge must clearly understand they are leaving verified territory. The trust boundary is explicit, not hidden.

**Implication for architecture:** Bridge modules carry their own trust tier declaration. The protocol and client must visually and technically communicate when a user or message originates from outside the verified identity space.

---

### Tension 3 — Federated Identity + No Anonymity + GDPR Right to be Forgotten

**The problem:** EU law gives every citizen the right to have their data deleted — everywhere. But XGen has cryptographic identity baked into the protocol, and federation means records exist across potentially hundreds of independent nodes with no central delete button. Who executes the deletion? How? This is a genuinely unsolved problem in federated systems generally.

**The provisional answer:** Deletion scope and obligations are tied to the **authentication tier of the server or chat**, not to the protocol globally. Higher tiers (Tier 3 Corporate, Tier 4 Government) already imply rigorous data handling — they carry explicit, enforceable deletion propagation obligations as part of their certification. Lower tiers (Tier 1 Community) operate on a best-effort basis with reduced legal exposure.

**Status:** Directionally sound but not fully specified. Requires deeper legal and technical work in Stage 3. Noted here so it is not rediscovered later.

---

### Foundational Decision — Predefined Starter Modules

**Context:** A protocol without working reference modules is hard to adopt. Matrix struggled with this for years — the spec existed but the reference implementation felt incomplete.

**Decision:** XGen will ship with a set of predefined starter modules — built by the core team, equal in standing to any future community module. Among them: a mid-tier authentication module (Tier 1–2) that works out of the box and demonstrates the pluggable auth model in practice. These starter modules set the quality bar and lower the barrier to first adoption.

---

## Known Tradeoffs & Honest Limitations

- **No anonymity** = system is not safe for users in authoritarian regimes. Consciously accepted.
- **Federated moderation** = some bad actors will find dark corners. Mitigated by identity verification.
- **"Free forever"** = needs a sustainable funding model that doesn't compromise independence. A diversified five-stream model has been defined — see Governance & Sustainability section.
- **Discord bridge** = third-party clients exist in a ToS gray zone with Discord.
- **Network effects** = people are where their friends are. Hardest problem to solve.
- **Spec-first is slower** = building the spec before the implementation takes discipline and time. The alternative is worse.
- **Single node type = trust complexity** = some capabilities (identity, high auth tiers) carry higher responsibility. The spec must define what it takes to advertise them. Stage 3 problem.
- **GDPR right-to-be-forgotten** = federated identity + no anonymity + right to erasure are in direct tension. Unresolved.

---

## Open Questions (Next Discussion Topics)

1. ~~**Governance model**~~ — ✓ Resolved. Dutch Stichting. See Governance & Sustainability section.
2. ~~**Sustainability model**~~ — ✓ Resolved. Five-stream model defined. See Governance & Sustainability section.
3. ~~**Legal incorporation**~~ — ✓ Resolved. Netherlands Stichting confirmed. See Governance & Sustainability section.
4. **Identity model detail** — Exact cryptographic design of server-independent identity. Key recovery. Key rotation.
4. **Encryption layer** — MLS (RFC 9420, 2023) vs Megolm-derived approach. Algorithm agility design.
5. **Protocol core spec** — What does the actual technical specification look like? Event schema. Room model. Federation algorithm.
6. **State resolution at scale** — How do we solve what Matrix's algorithm doesn't?
7. **Community primitive detail** — Naming, exact ownership model, permission cascade design, portability mechanism.
8. **Thread model** — What is a thread for? How does it behave in a federated context? Designed from first principles, not copied.
9. **Node capability trust levels** — What verification is required to advertise high-responsibility capabilities like `identity`?
10. **Discord bridge strategy** — How exactly do we stay compatible without violating ToS?
11. **First team** — Who are the first people needed to make this real?
12. **Reference node** — Language choice. Lightweight by design. What does accessible self-hosting look like?
13. **Credential upgrade UX** — When a Tier 1 user hits a Tier 2 space, what does that moment feel like? Door or wall?
14. **Module-specific credential lifecycle** — Who manages revocation, expiry, re-enrollment for institutional modules?

---

## Current Development Stage

```
Stage 1 — Philosophy      ✓ LARGELY COMPLETE
"Why this must exist and what it must stand for"

Stage 2 — Architecture    ← YOU ARE HERE
"How the pieces relate to each other conceptually"

Stage 3 — Specification
"Formal protocol definition, RFCs, schemas"

Stage 4 — Reference Implementation
"Working code that proves the spec is viable"

Stage 5 — Open Protocol
"Adopted, implemented by multiple independent parties"
```

At this stage XGen is a **vision document** — a coherent set of answers to *why* and *what*, but not yet touching *how*. More precisely: a **founding philosophy document / manifesto with architecture instincts.**

That is not a weakness. The best protocols started exactly here.

---

## One Sentence Version

> *XGen Protocol is an open, federated, identity-verified communication protocol — with modular trust tiers, server-independent user-side portable identity, a single extensible node type, and community as a first-class primitive — structurally incapable of enshittification, around which a community freely builds the applications the world actually needs.*

---

*Document generated from brainstorming sessions — April 2026*  
*Version 0.6 — Full reconstruction: resolved tensions, auth UX principle, governance & sustainability merged*  
*Built by GenX. For everyone.*  
*Ready for sharing with collaborators*

---

## Session Log

### Session 1 — April 2026
**Covered:** Name & origin, core philosophy, four pillars, architecture concept, competitive differentiation, historical parallels, known tradeoffs. → v0.1

### Session 2 — April 2026
**Covered:** Three core philosophical tensions stress-tested and provisionally resolved (government identity demands, Discord bridge trust collision, GDPR right-to-be-forgotten). Foundational decision on predefined starter modules captured. → v0.2

### Session 3 — April 2026
**Covered:** Governance model defined (Dutch Stichting nonprofit, two-track founder control + community cultivation, hard governance rules). Sustainability model defined (five income streams, 30–40% cap rule, Blender blueprint lessons mapped). Licensing & IP mechanism defined (BSL + CLA, GPL conversion triggered by project state — two independent client implementations + stable RFC). Legal incorporation confirmed (Netherlands Stichting, EU alternatives investigated and ruled out). Authentication UX principle added (progressive, user-initiated — Tier 1 at installation, self-upgrade required for higher tiers). → v0.3

### Session 4 — April 2026
**Covered:** Node type architecture (single node type, capability advertisement, open enum principle). Community as first-class protocol primitive. Reference client strategy. Matrix intellectual lineage — what to keep, what to leave behind. Future pressures (regulatory, quantum, AI, jurisdictional). Fifth philosophical pillar added (Temporal Resilience). → v0.3 GitHub

### Session 5 — April 2026
**Covered:** Authentication model expanded — user-side portable identity, cumulative tiers, vanilla default format, module-specific institutional credentials. Stage marker updated (Stage 1 largely complete, entering Stage 2). Full document reconstructed and merged from all sessions. → v0.6

**Next session to begin with:**
> **Governance & sustainability deep-dive** is complete. **Matrix/Element deep-dive** is the next priority — mapping their failures as a negative blueprint before any spec work begins.
