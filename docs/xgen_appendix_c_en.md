# XGen Protocol — Appendix C: Primitive Schemas & Inheritance Diagrams
> Status: wip
> Version: 0.1
> Date: April 2026
> Last edited: April 2026
> Language: English
> Author: JozefN
> License: BSL 1.1 (converts to GPL upon project handover)
> Note: Slovak translation planned — will be saved as xgen_appendix_c_sk.md

---

## Purpose

This appendix provides a complete reference of all XGen Protocol primitive schemas and their inheritance relationships in a single document. It is intended as a theoretical implementation guide — a developer starting work on a Node, client, or Auth Module should read this appendix alongside Chapter 3 (Specification) to understand both the data structures and how they relate to each other.

All diagrams use [Mermaid](https://mermaid.js.org/) class diagram syntax, which renders natively in GitHub, VS Code with the Mermaid extension, and most modern markdown viewers.

---

## C.1 — Inheritance & Relationship Overview

The top-level view of all primitives and their relationships. Read the arrows as:
- `<|--` inheritance (is-a)
- `*--` composition (owns, lifecycle depends on parent)
- `o--` aggregation (references, independent lifecycle)
- `--` association (relates to)
- Numbers on arrows indicate cardinality.

```mermaid
classDiagram

    %% ── Infrastructure layer ──────────────────────────────────────
    class Machine {
        +hardware: physical|virtual
    }
    class Node {
        +id: xgen_uri
        +name: string
        +capabilities: Capability[]
        +capacity: low|medium|high
        +auth_tiers: int[]
        +jurisdiction: string
        +version: string
        +signature: Signature
        +meta_atts: MetaAtts
    }

    %% ── Protocol primitives ───────────────────────────────────────
    class Space {
        +id: xgen_uri
        +name: string
        +description: string
        +created_at: timestamp
        +created_by: Identity
        +home_node: Node
        +auth_tier_min: int
        +visibility: public|private|invite_only
        +roles: Role[]
        +members: Member[]
        +rooms: Room[]
        +board: BoardEntry[]
        +invite_code: string
        +meta_atts: MetaAtts
    }
    class DMSpace {
        +type: dm
        +visibility: invite_only
        +members: Identity[2..*]
        +rooms: Room[1]
        +roles: empty
        +invite_code: null
        +discoverable: false
    }
    class Room {
        +id: xgen_uri
        +space: Space
        +type: RoomType
        +name: string
        +topic: string
        +created_at: timestamp
        +created_by: Identity
        +auth_tier_min: int
        +permissions: Permission[]
        +members: Member[]
        +board: BoardEntry[]
        +meta_atts: MetaAtts
    }
    class Thread {
        +id: xgen_uri
        +room: Room
        +created_by: Identity
        +created_at: timestamp
        +origin_event: Event
        +title: string
        +status: open|resolved|archived
        +auth_tier_min: int
        +meta_atts: MetaAtts
    }
    class Event {
        +id: hash_uri
        +type: EventType
        +room: Room
        +sender: Identity
        +timestamp: timestamp
        +prev_events: Event[]
        +content: object
        +signature: Signature
        +meta_atts: MetaAtts
    }

    %% ── Identity layer ────────────────────────────────────────────
    class Identity {
        +id: pubkey_uri
        +display_name: string
        +created_at: timestamp
        +home_node: Node
        +current_key: PublicKey
        +previous_keys: PublicKey[]
        +trust_assertion: TrustAssertion
        +devices: Device[]
        +meta_atts: MetaAtts
    }
    class TrustAssertion {
        +identity: Identity
        +tier: int
        +issued_at: timestamp
        +expires_at: timestamp
        +issuer: AuthModule
        +jurisdiction: string
        +signature: Signature
        +meta_atts: MetaAtts
    }
    class Device {
        +id: xgen_uri
        +identity: Identity
        +public_key: PublicKey
        +authorised_at: timestamp
        +authorised_by: Device
        +name: string
    }
    class AuthModule {
        +id: xgen_uri
        +tier: int
        +version: string
        +jurisdiction: string
        +status: ModuleStatus
        +signature: Signature
    }

    %% ── Social layer ──────────────────────────────────────────────
    class Contact {
        +identity: Identity
        +alias: string
        +note: string
        +added_at: timestamp
        +meta_atts: MetaAtts
    }
    class IdentityPrivate {
        +contacts: Contact[]
        +blocked_identities: Identity[]
        +dm_privacy_setting: DMPrivacy
        +identity_level_mutes: Identity[]
        +meta_atts: MetaAtts
    }

    %% ── Supporting types ──────────────────────────────────────────
    class Role {
        +id: xgen_uri
        +name: string
        +color: string
        +permissions: Permission[]
        +position: int
        +meta_atts: MetaAtts
    }
    class BoardEntry {
        +event_id: hash_uri
        +pinned_by: Identity
        +pinned_at: timestamp
        +label: string
    }
    class Signature {
        +algorithm: string
        +key_id: string
        +bytes: base64
    }
    class MetaAtts {
        +entries: map~string_any~
    }

    %% ── Relationships ─────────────────────────────────────────────

    %% Infrastructure
    Machine "1" *-- "1..*" Node : hosts
    Node "1" *-- "0..*" Space : hosts

    %% Space hierarchy
    Space <|-- DMSpace : specialises
    Space "1" *-- "1..*" Room : contains
    Space "1" *-- "1..*" Role : defines
    Space "1" o-- "0..*" BoardEntry : board

    %% Room hierarchy
    Room "1" *-- "0..*" Thread : contains
    Room "1" *-- "1..*" Event : log
    Room "1" o-- "0..*" BoardEntry : board

    %% Thread
    Thread "1" o-- "1" Event : origin_event
    Thread "1" *-- "0..*" Event : replies

    %% Event
    Event "0..*" o-- "0..*" Event : prev_events
    Event "0..*" o-- "1" Identity : sender

    %% Identity
    Identity "1" *-- "1" TrustAssertion : has
    Identity "1" *-- "1..*" Device : registers
    Identity "1" *-- "1" IdentityPrivate : owns
    Identity "1" o-- "1" Node : home_node

    %% TrustAssertion
    TrustAssertion "0..*" o-- "1" AuthModule : issued_by

    %% Device
    Device "0..*" o-- "0..1" Device : authorised_by

    %% Social
    IdentityPrivate "1" *-- "0..*" Contact : contains
    Contact "0..*" o-- "1" Identity : references

    %% Supporting
    Space "1" o-- "0..*" Identity : members
    Room "1" o-- "0..*" Identity : members
    Event "1" *-- "1" Signature : has
    Event "1" *-- "1" MetaAtts : has
    Identity "1" *-- "1" MetaAtts : has
    Node "1" *-- "1" MetaAtts : has
```

---

## C.2 — Event Primitive

The atom of the protocol. Every action in XGen is an Event.

```mermaid
classDiagram
    class Event {
        +id: hash_uri
        +type: EventType
        +room: xgen_uri
        +sender: xgen_uri
        +timestamp: int
        +prev_events: hash_uri[]
        +content: object
        +signature: Signature
        +meta_atts: MetaAtts
    }
    class EventType {
        <<enumeration>>
        message.text
        message.rich
        message.edit
        message.delete
        message.reaction
        message.reply
        file.upload
        call.start
        call.join
        call.leave
        call.end
        stream.start
        stream.end
        thread.create
        thread.resolved
        poll.create
        poll.vote
        room.member.join
        room.member.leave
        room.member.kick
        room.member.ban
        room.name.change
        room.topic.change
        room.permission.change
        room.pin.add
        room.pin.remove
        space.member.join
        space.member.leave
        space.role.assign
        space.role.revoke
        space.settings.change
        node.federation.join
        node.federation.leave
        identity.key.rotate
        identity.auth.upgrade
        identity.device.add
        identity.device.revoke
        identity.node.migrate
        space.node.migrate
        room.created
        room.archived
        bridge.message.in
        bridge.message.out
        bridge.member.in
    }
    class Signature {
        +algorithm: string
        +key_id: string
        +bytes: base64
    }
    class MetaAtts {
        +entries: map~string_any~
    }

    Event "1" *-- "1" Signature : signed_by
    Event "1" *-- "1" MetaAtts : extended_by
    Event ..> EventType : typed_as
    Event "0..*" o-- "0..*" Event : prev_events
```

---

## C.3 — Thread Primitive

A scoped, bounded conversation within a Room. Not a Room. Not a reply chain.

```mermaid
classDiagram
    class Thread {
        +id: xgen_uri
        +room: xgen_uri
        +created_by: xgen_uri
        +created_at: int
        +origin_event: hash_uri
        +title: string
        +status: ThreadStatus
        +auth_tier_min: int
        +meta_atts: MetaAtts
    }
    class ThreadStatus {
        <<enumeration>>
        open
        resolved
        archived
    }
    class Event {
        +id: hash_uri
        +type: EventType
    }
    class Room {
        +id: xgen_uri
    }

    Thread "0..*" o-- "1" Room : belongs_to
    Thread "1" o-- "1" Event : origin_event
    Thread "1" *-- "0..*" Event : replies
    Thread ..> ThreadStatus : has_status
    Thread "1" *-- "1" MetaAtts : extended_by
```

---

## C.4 — Room Primitive

The core communication unit. A persistent, federated container of Events and Threads.

```mermaid
classDiagram
    class Room {
        +id: xgen_uri
        +space: xgen_uri
        +type: RoomType
        +name: string
        +topic: string
        +created_at: int
        +created_by: xgen_uri
        +auth_tier_min: int
        +permissions: Permission[]
        +members: xgen_uri[]
        +board: BoardEntry[]
        +meta_atts: MetaAtts
    }
    class RoomType {
        <<enumeration>>
        room.text
        room.voice
        room.video
        room.forum
        room.announcements
        room.stage
    }
    class BoardEntry {
        +event_id: hash_uri
        +pinned_by: xgen_uri
        +pinned_at: int
        +label: string
    }
    class Thread {
        +id: xgen_uri
    }
    class Event {
        +id: hash_uri
    }

    Room ..> RoomType : typed_as
    Room "1" *-- "1..*" Event : event_log
    Room "1" *-- "0..*" Thread : contains
    Room "1" *-- "0..*" BoardEntry : board
    Room "1" *-- "1" MetaAtts : extended_by
```

---

## C.5 — Space Primitive

The top-level container. A governed, portable, cryptographically-identified community.

```mermaid
classDiagram
    class Space {
        +id: xgen_uri
        +name: string
        +description: string
        +created_at: int
        +created_by: xgen_uri
        +home_node: xgen_uri
        +auth_tier_min: int
        +visibility: Visibility
        +roles: Role[]
        +members: Member[]
        +rooms: xgen_uri[]
        +board: BoardEntry[]
        +invite_code: string
        +meta_atts: MetaAtts
    }
    class DMSpace {
        +type: dm
        +visibility: invite_only
        +members: xgen_uri[2..*]
        +rooms: xgen_uri[1]
        +roles: empty
        +invite_code: null
        +discoverable: false
    }
    class SpaceLifecycle {
        <<enumeration>>
        created
        active
        archived
        migrated
    }
    class Visibility {
        <<enumeration>>
        public
        private
        invite_only
    }
    class Role {
        +id: xgen_uri
        +name: string
        +color: string
        +permissions: Permission[]
        +position: int
        +meta_atts: MetaAtts
    }
    class BuiltInRole {
        <<enumeration>>
        Owner
        Admin
        Moderator
        Member
        Guest
    }
    class Room {
        +id: xgen_uri
    }
    class BoardEntry {
        +event_id: hash_uri
        +pinned_by: xgen_uri
        +pinned_at: int
        +label: string
    }

    Space <|-- DMSpace : specialises
    Space ..> Visibility : has_visibility
    Space ..> SpaceLifecycle : has_lifecycle
    Space "1" *-- "1..*" Room : contains
    Space "1" *-- "1..*" Role : defines
    Space "1" *-- "0..*" BoardEntry : board
    Role ..> BuiltInRole : extends_or_is
```

---

## C.6 — Node Primitive

The infrastructure unit. The concrete boundary between hardware and protocol.

```mermaid
classDiagram
    class Node {
        +id: xgen_uri
        +name: string
        +capabilities: Capability[]
        +capacity: Capacity
        +auth_tiers: int[]
        +jurisdiction: string
        +version: string
        +signature: Signature
        +meta_atts: MetaAtts
    }
    class Capability {
        <<enumeration>>
        messaging
        identity
        federation
        gateway
        media_relay
        file_storage
        bridge
        auth_tier_1
        auth_tier_2
        auth_tier_3
        auth_tier_4
    }
    class Capacity {
        <<enumeration>>
        low
        medium
        high
    }
    class NodeProfile {
        <<enumeration>>
        vanilla
        community
        full
        corporate
        government
    }
    class Machine {
        +hardware: physical|virtual
    }
    class Space {
        +id: xgen_uri
    }

    Machine "1" *-- "1..*" Node : runs
    Node "1" *-- "0..*" Space : hosts
    Node ..> Capability : declares
    Node ..> Capacity : has_capacity
    Node ..> NodeProfile : typical_profile
    Node "1" *-- "1" Signature : signed_by
    Node "1" *-- "1" MetaAtts : extended_by
```

---

## C.7 — Identity Primitive

The server-independent keypair. The user's permanent protocol presence.

```mermaid
classDiagram
    class Identity {
        +id: pubkey_uri
        +display_name: string
        +created_at: int
        +home_node: xgen_uri
        +current_key: PublicKey
        +previous_keys: PublicKey[]
        +trust_assertion: TrustAssertion
        +devices: Device[]
        +meta_atts: MetaAtts
    }
    class IdentityPrivate {
        +contacts: Contact[]
        +blocked_identities: xgen_uri[]
        +dm_privacy_setting: DMPrivacy
        +identity_level_mutes: xgen_uri[]
        +meta_atts: MetaAtts
    }
    class IdentityLifecycle {
        <<enumeration>>
        created
        active
        suspended
        migrated
        orphaned
    }
    class DMPrivacy {
        <<enumeration>>
        open
        contacts_only
        spaces_only
        closed
    }
    class Device {
        +id: xgen_uri
        +identity: xgen_uri
        +public_key: PublicKey
        +authorised_at: int
        +authorised_by: xgen_uri
        +name: string
    }
    class TrustAssertion {
        +identity: xgen_uri
        +tier: int
        +issued_at: int
        +expires_at: int
        +issuer: xgen_uri
        +jurisdiction: string
        +signature: Signature
        +meta_atts: MetaAtts
    }
    class PublicKey {
        +algorithm: string
        +key_id: string
        +bytes: base64
    }

    Identity "1" *-- "1" IdentityPrivate : private_record
    Identity "1" *-- "1" TrustAssertion : assertion
    Identity "1" *-- "1..*" Device : devices
    Identity "1" *-- "1" MetaAtts : extended_by
    Identity ..> IdentityLifecycle : has_lifecycle
    IdentityPrivate ..> DMPrivacy : dm_privacy
    Device "0..*" o-- "0..1" Device : authorised_by
    TrustAssertion "1" *-- "1" Signature : signed_by
```

---

## C.8 — Auth Module & Trust Assertion

The pluggable authentication slot and its standardised output.

```mermaid
classDiagram
    class AuthModule {
        +id: xgen_uri
        +tier: int
        +version: string
        +jurisdiction: string
        +status: ModuleStatus
        +signature: Signature
    }
    class ModuleStatus {
        <<enumeration>>
        specified
        developed
        certified
        active
        deprecated
        revoked
    }
    class TrustAssertion {
        +identity: xgen_uri
        +tier: int
        +issued_at: int
        +expires_at: int
        +issuer: xgen_uri
        +jurisdiction: string
        +signature: Signature
        +meta_atts: MetaAtts
    }
    class AuthTier {
        <<enumeration>>
        1_community
        2_professional
        3_corporate
        4_government
    }
    class Signature {
        +algorithm: string
        +key_id: string
        +bytes: base64
    }

    AuthModule ..> ModuleStatus : has_status
    AuthModule ..> AuthTier : implements_tier
    AuthModule "1" o-- "0..*" TrustAssertion : issues
    TrustAssertion "1" *-- "1" Signature : signed_by
    TrustAssertion "1" *-- "1" MetaAtts : extended_by
```

---

## C.9 — Contact Model & User Representation

The private social layer. Stored encrypted in the Identity private record.

```mermaid
classDiagram
    class Contact {
        +identity: xgen_uri
        +alias: string
        +note: string
        +added_at: int
        +meta_atts: ContactMetaAtts
    }
    class ContactMetaAtts {
        +xgen_contact_group: string
        +xgen_contact_tags: string[]
        +xgen_contact_priority: string
        +xgen_contact_met_at: string
        +xgen_contact_trust: string
        +xgen_contact_mute: bool
        +xgen_contact_favourite: bool
    }
    class UserRepresentation {
        <<concept>>
        1_contact_alias
        2_space_nickname
        3_global_display_name
    }
    class RepresentationScope {
        <<enumeration>>
        global
        space_scoped
        contact_private
    }

    Contact "1" *-- "1" ContactMetaAtts : meta
    Contact "0..*" o-- "1" Identity : references
    UserRepresentation ..> RepresentationScope : scoped_by
    Contact ..> UserRepresentation : overrides_at_level_1
```

---

## C.10 — Direct Message Space

A specialisation of Space. Minimal, private, no governance overhead.

```mermaid
classDiagram
    class Space {
        +id: xgen_uri
        +visibility: Visibility
        +roles: Role[]
        +members: Member[]
        +rooms: Room[]
        +auth_tier_min: int
        +meta_atts: MetaAtts
    }
    class DMSpace {
        +type: dm
        +visibility: invite_only
        +members: Identity[2..*]
        +rooms: Room[1]
        +roles: empty
        +invite_code: null
        +discoverable: false
    }
    class DMPrivacy {
        <<enumeration>>
        open
        contacts_only
        spaces_only
        closed
    }
    class DMLifecycle {
        <<enumeration>>
        pending
        accepted
        declined
        active
        promoted_to_space
    }

    Space <|-- DMSpace : specialises
    DMSpace ..> DMPrivacy : controlled_by
    DMSpace ..> DMLifecycle : has_lifecycle
    DMSpace "1" o-- "1..*" Identity : members
```

---

## C.11 — Cryptographic Primitives

The algorithm-agile signature and hash types used across all primitives.

```mermaid
classDiagram
    class Signature {
        +algorithm: string
        +key_id: string
        +bytes: base64
    }
    class HashURI {
        +algorithm: string
        +hex: string
    }
    class PublicKey {
        +algorithm: string
        +key_id: string
        +bytes: base64
    }
    class AlgorithmAgility {
        <<concept>>
        algorithm_declared_in_field
        not_hardcoded_in_protocol
        unknown_algorithms_handled_gracefully
    }
    class CurrentDefaults {
        <<note>>
        signature: ed25519
        hash: sha256
        post_quantum_ready: ml_dsa_65
    }

    Signature ..> AlgorithmAgility : follows
    HashURI ..> AlgorithmAgility : follows
    PublicKey ..> AlgorithmAgility : follows
    AlgorithmAgility ..> CurrentDefaults : defaults
```

---

## C.12 — meta-atts Universal Extension

The namespaced key-value map carried by every primitive.

```mermaid
classDiagram
    class MetaAtts {
        +entries: map~string_any~
    }
    class NamespaceConvention {
        <<concept>>
        xgen_dot: core_protocol_reserved
        xgen_contact_dot: contact_standard_keys
        org_dot: organisation_namespace
        app_dot: application_namespace
        custom: any_other_namespace
    }
    class PropagationPolicy {
        <<enumeration>>
        federated
        local_only
    }
    class Event
    class Room
    class Space
    class Thread
    class Node
    class Identity
    class Role
    class Contact
    class TrustAssertion

    MetaAtts ..> NamespaceConvention : namespaced_by
    MetaAtts ..> PropagationPolicy : propagated_or_local
    Event "1" *-- "1" MetaAtts
    Room "1" *-- "1" MetaAtts
    Space "1" *-- "1" MetaAtts
    Thread "1" *-- "1" MetaAtts
    Node "1" *-- "1" MetaAtts
    Identity "1" *-- "1" MetaAtts
    Role "1" *-- "1" MetaAtts
    Contact "1" *-- "1" MetaAtts
    TrustAssertion "1" *-- "1" MetaAtts
```

---

## C.13 — Federation Relationships

How Nodes, Spaces, Rooms, and Identities relate in a federated network.

```mermaid
classDiagram
    class Node {
        +id: xgen_uri
        +capabilities: Capability[]
        +federation_peers: Node[]
    }
    class Space {
        +id: xgen_uri
        +home_node: xgen_uri
    }
    class Room {
        +id: xgen_uri
        +federated_nodes: Node[]
    }
    class Identity {
        +id: pubkey_uri
        +home_node: xgen_uri
        +replica_nodes: Node[]
    }
    class FederationRelationship {
        +node_a: xgen_uri
        +node_b: xgen_uri
        +scope: room_scoped
        +established_at: int
        +status: active|suspended|terminated
    }
    class PresenceSignal {
        +identity: xgen_uri
        +space: xgen_uri
        +status: online|away|busy
        +expires_at: int
        +signed_by: device_key
    }

    Node "0..*" o-- "0..*" Node : federates_with
    Node "1" *-- "0..*" Space : hosts
    Room "0..*" o-- "1..*" Node : replicated_across
    Identity "1" o-- "1" Node : home_node
    Identity "1" o-- "0..*" Node : replica_nodes
    FederationRelationship ..> Room : scoped_to
    PresenceSignal ..> Space : scoped_to
    PresenceSignal ..> Identity : from
```

---

## C.14 — Reference Client Layers

The four-layer architecture of the reference client.

```mermaid
classDiagram
    class TransportLayer {
        <<Layer 1>>
        +connect(node_address)
        +send(raw_message)
        +on_message(raw_message)
        +on_connected()
        +on_disconnected()
        +reconnect_with_backoff()
    }
    class ProtocolLayer {
        <<Layer 2>>
        +submit_event(event)
        +on_event(event)
        +get_room_state(room_id)
        +get_event(event_id)
        +sign_event(event)
        +verify_event(event)
        +manage_dag(room_id)
        +resolve_state(room_id)
    }
    class ApplicationLayer {
        <<Layer 3>>
        +get_space_list()
        +get_room_timeline(room_id)
        +get_thread_list(room_id)
        +get_contact_list()
        +send_message(room_id, content)
        +on_notification(notification)
        +send_presence(space_id, status)
        +sync_private_identity_record()
    }
    class PresentationLayer {
        <<Layer 4>>
        +render_space_list()
        +render_room(room_id)
        +render_thread(thread_id)
        +render_contact_list()
        +render_board(room_id)
        +handle_input()
        +display_notification()
        +render_presence_indicators()
    }

    TransportLayer "1" -- "1" ProtocolLayer : raw_messages
    ProtocolLayer "1" -- "1" ApplicationLayer : events_and_state
    ApplicationLayer "1" -- "1" PresentationLayer : ui_data
```

---

## Session Log

### Session 1 — April 2026 (JozefN)
**Covered:** Appendix C created. Fourteen diagrams written in Mermaid class diagram syntax: C.1 full overview with all primitives and relationships, C.2 Event, C.3 Thread, C.4 Room, C.5 Space (with DMSpace specialisation), C.6 Node, C.7 Identity (with IdentityPrivate), C.8 Auth Module & Trust Assertion, C.9 Contact Model & User Representation, C.10 Direct Message Space, C.11 Cryptographic Primitives, C.12 meta-atts universal extension, C.13 Federation Relationships, C.14 Reference Client Layers.
