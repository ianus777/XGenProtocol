// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Scenario manifest (M9-D8 / Checkpoint #1).
//!
//! Each scenario lives in `docs/tests/multiparty_scenarios/<ID>/` with one
//! `<actor>.jsonl` batch per actor plus a `manifest.toml`. The manifest declares
//! the node topology, the actors (→ node assignment + batch file), the
//! federation links to establish, and the **cross-actor exports** that publish a
//! reply field under a `{{key}}` name.
//!
//! ## `{{key}}` cross-actor resolution shape (Checkpoint #1)
//! `bind`/`$` is **per-connection** (the binary's own substitution), so it
//! cannot carry a value from one actor to another. Cross-actor values use the
//! `{{key}}` placeholder, resolved by the orchestrator from the **exports**
//! table: each export maps `(actor, command-id, reply-field) → key`. **Imports
//! are implicit** — any `{{key}}` token appearing in a command's args is
//! resolved from the shared registry, and the consuming command **blocks** until
//! that key is published. This data-dependency edge is what orders the run (no
//! manual barrier needed for the common case): e.g. in MP-C-02 Bob's `join`
//! waits for Alice's exported `space_id`/`invite_event_id`, and Alice's `invite`
//! waits for Bob's exported `bob_identity_id`.
//!
//! `barriers` (optional, named) cover ordering that is *not* data-dependent
//! (e.g. "both members before both send" for a concurrent-frontier scenario).
//! Round-0 (MP-C-02) needs none; the field exists for the full batteries.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::Result;
use anyhow::{anyhow, Context};

/// A parsed scenario manifest plus the directory it was loaded from (used to
/// resolve relative `batch` paths).
#[derive(Debug, Clone)]
pub struct Scenario {
    pub manifest: Manifest,
    pub dir: PathBuf,
}

/// The `manifest.toml` schema.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Scenario ID, e.g. `"MP-C-02"`.
    pub scenario: String,
    #[serde(default)]
    pub description: String,
    /// Node topology — each entry is one spawned node.
    pub nodes: Vec<NodeSpec>,
    /// Actors driven through client `.aicontrol` pipes.
    pub actors: Vec<ActorSpec>,
    /// Federation links to establish before driving actors.
    #[serde(default)]
    pub federation: Vec<FederationLink>,
    /// Cross-actor reply exports (publish a reply field under a `{{key}}`).
    #[serde(default)]
    pub exports: Vec<Export>,
    /// Optional named ordering barriers (non-data-dependent ordering).
    #[serde(default)]
    pub barriers: Vec<Barrier>,
    /// Explicit cross-actor happens-after edges that data-dependency cannot
    /// express: a command waits for an exported `key` even though that key is
    /// **not** one of its args. (MP-C-02: Bob's `join` must follow Alice's
    /// `invite`, but `join`'s only input is `space_id` — which precedes the
    /// invite — so the ordering is declared here against the invite's exported
    /// `key`.)
    #[serde(default)]
    pub waits: Vec<Wait>,
    /// Ordered clock-control steps (MP-R1-D3). The clock is a scenario-director
    /// action, not an actor command, so it lives here rather than in a `.jsonl`.
    /// Each step drives the node's injected `MockClock` over the fenced F3 verbs
    /// (`--features harness-control`). Unblocks MP-A-01 (advance past an invite's
    /// `valid_until`, then replay).
    #[serde(default)]
    pub clock: Vec<Clock>,
}

/// One node in the topology.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeSpec {
    /// Topology-local label, e.g. `"a"`. Combined with a run nonce into the real
    /// instance label.
    pub label: String,
    /// WS listen port for this node.
    pub port: u16,
    /// Force Local-Node mode (default `true` — cooperative Round-0 wants it).
    #[serde(default = "default_true")]
    pub local: bool,
}

/// What kind of driver an actor is (MP-R1-D1 — per-actor dispatch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActorKind {
    /// A cooperative/logic-adversarial actor: a real `--service` client resident
    /// driven through its `.aicontrol` pipe by [`crate::batch::run_actor`]
    /// (the default).
    #[default]
    Batch,
    /// The test-only raw-wire hostile client ([`crate::injector`]) — it does not
    /// spawn a client process or go through `run_actor`; it speaks the transport
    /// directly to its node's `ws://`. The runner routes it to the injector path
    /// (wired in C7 — MP-A-05/09/10/12).
    Injector,
}

/// One actor (a client resident driven over `.aicontrol`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorSpec {
    /// Actor name, e.g. `"alice"`. Also the export namespace.
    pub name: String,
    /// The topology node label this actor's client connects to (`--node`).
    pub node: String,
    /// The per-actor JSONL batch file, relative to the scenario dir.
    pub batch: String,
    /// Run this actor as an AI resident (`--ai-mode`). Default `false`.
    #[serde(default)]
    pub ai_mode: bool,
    /// Actor kind (MP-R1-D1). Default [`ActorKind::Batch`].
    #[serde(default)]
    pub kind: ActorKind,
}

/// A directed federation link `from → to` (node labels).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FederationLink {
    pub from: String,
    pub to: String,
}

/// Publish `actor.command`'s reply `field` under `key` for `{{key}}` consumers.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    /// Producing actor name.
    pub actor: String,
    /// The producing command's `id` (the `"id"` in its JSONL line).
    pub command: String,
    /// The reply `data` field to publish, e.g. `"space_id"`.
    pub field: String,
    /// The `{{key}}` name consumers reference.
    pub key: String,
}

/// A named ordering barrier (optional; for non-data-dependent ordering).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Barrier {
    /// Barrier name.
    pub name: String,
    /// Actors that must reach this barrier (each after the listed command id)
    /// before any actor proceeds past it.
    pub members: Vec<BarrierMember>,
}

/// One actor's participation in a barrier: it arrives after `after_command`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BarrierMember {
    pub actor: String,
    /// The command id after which this actor reaches the barrier.
    pub after_command: String,
}

/// An explicit happens-after edge: `actor`'s command `command` waits for the
/// exported `key` before it is sent (ordering beyond args data-dependency).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wait {
    pub actor: String,
    pub command: String,
    pub key: String,
}

/// A clock-control operation (MP-R1-D3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClockOp {
    /// `clock advance <duration>` — move the node's MockClock forward by `value`
    /// (a duration, e.g. `"15d"`).
    Advance,
    /// `clock set <rfc3339>` — pin the node's MockClock to the absolute instant
    /// `value` (RFC 3339).
    Set,
}

/// One ordered clock-control step (MP-R1-D3). Steps fire in manifest order; a
/// step with an `after` key blocks (via the shared registry) until that
/// exported key is published before it sends its F3 verb.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clock {
    /// Topology node label whose injected `MockClock` this step drives.
    pub node: String,
    /// `advance` | `set`.
    pub op: ClockOp,
    /// Duration (for `advance`, e.g. `"15d"`) or RFC 3339 instant (for `set`).
    pub value: String,
    /// Optional export-key (or barrier) this step waits on before firing. A step
    /// with no `after` fires at the start of the director's clock phase.
    #[serde(default)]
    pub after: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Manifest {
    /// Parse a manifest from a TOML string.
    pub fn parse(toml_str: &str) -> Result<Manifest> {
        let m: Manifest = toml::from_str(toml_str).context("parsing manifest.toml")?;
        m.validate()?;
        Ok(m)
    }

    /// Structural validation: non-empty topology/actors, references resolve.
    fn validate(&self) -> Result<()> {
        if self.nodes.is_empty() {
            return Err(anyhow!("manifest `{}` declares no nodes", self.scenario));
        }
        if self.actors.is_empty() {
            return Err(anyhow!("manifest `{}` declares no actors", self.scenario));
        }
        let has_node = |label: &str| self.nodes.iter().any(|n| n.label == label);
        let has_actor = |name: &str| self.actors.iter().any(|a| a.name == name);
        for a in &self.actors {
            if !has_node(&a.node) {
                return Err(anyhow!(
                    "actor `{}` references unknown node `{}`",
                    a.name,
                    a.node
                ));
            }
        }
        for f in &self.federation {
            if !has_node(&f.from) || !has_node(&f.to) {
                return Err(anyhow!(
                    "federation link {} → {} references an unknown node",
                    f.from,
                    f.to
                ));
            }
        }
        for e in &self.exports {
            if !has_actor(&e.actor) {
                return Err(anyhow!(
                    "export key `{}` references unknown actor `{}`",
                    e.key,
                    e.actor
                ));
            }
        }
        for c in &self.clock {
            if !has_node(&c.node) {
                return Err(anyhow!(
                    "clock step references unknown node `{}`",
                    c.node
                ));
            }
        }
        Ok(())
    }

    /// Find the actor spec by name.
    pub fn actor(&self, name: &str) -> Option<&ActorSpec> {
        self.actors.iter().find(|a| a.name == name)
    }

    /// Exports produced by a given actor.
    pub fn exports_of<'a>(&'a self, actor: &'a str) -> impl Iterator<Item = &'a Export> + 'a {
        self.exports.iter().filter(move |e| e.actor == actor)
    }

    /// Extra wait-keys per command for an actor (`command_id → [key, …]`), from
    /// the `[[waits]]` table.
    pub fn waits_of(&self, actor: &str) -> std::collections::HashMap<String, Vec<String>> {
        let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for w in self.waits.iter().filter(|w| w.actor == actor) {
            map.entry(w.command.clone()).or_default().push(w.key.clone());
        }
        map
    }
}

impl Scenario {
    /// Load a scenario directory: parse `manifest.toml`, verify each actor's
    /// batch file exists.
    pub fn load(dir: impl AsRef<Path>) -> Result<Scenario> {
        let dir = dir.as_ref().to_path_buf();
        let manifest_path = dir.join("manifest.toml");
        let text = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("reading {}", manifest_path.display()))?;
        let manifest = Manifest::parse(&text)?;
        for a in &manifest.actors {
            let bp = dir.join(&a.batch);
            if !bp.exists() {
                return Err(anyhow!(
                    "actor `{}` batch file not found: {}",
                    a.name,
                    bp.display()
                ));
            }
        }
        Ok(Scenario { manifest, dir })
    }

    /// Absolute path to an actor's batch file.
    pub fn batch_path(&self, actor: &ActorSpec) -> PathBuf {
        self.dir.join(&actor.batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MP_C_02: &str = r#"
scenario = "MP-C-02"
description = "invite & join across two nodes"

[[nodes]]
label = "a"
port = 8401

[[nodes]]
label = "b"
port = 8402

[[federation]]
from = "a"
to = "b"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"

[[actors]]
name = "bob"
node = "b"
batch = "bob.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"

[[exports]]
actor = "alice"
command = "a3"
field = "event_id"
key = "invite_event_id"

[[exports]]
actor = "bob"
command = "b1"
field = "identity_id"
key = "bob_identity_id"
"#;

    #[test]
    fn parses_mp_c_02_manifest() {
        let m = Manifest::parse(MP_C_02).unwrap();
        assert_eq!(m.scenario, "MP-C-02");
        assert_eq!(m.nodes.len(), 2);
        assert_eq!(m.actors.len(), 2);
        assert_eq!(m.federation.len(), 1);
        assert_eq!(m.exports.len(), 3);
        // local defaults to true.
        assert!(m.nodes[0].local);
        let alice_exports: Vec<_> = m.exports_of("alice").collect();
        assert_eq!(alice_exports.len(), 2);
    }

    #[test]
    fn actor_kind_defaults_to_batch_and_parses_injector() {
        let toml = r#"
scenario = "X"
[[nodes]]
label = "a"
port = 8401
[[actors]]
name = "alice"
node = "a"
batch = "a.jsonl"
[[actors]]
name = "mallory"
node = "a"
batch = "m.jsonl"
kind = "injector"
"#;
        let m = Manifest::parse(toml).unwrap();
        assert_eq!(m.actor("alice").unwrap().kind, ActorKind::Batch);
        assert_eq!(m.actor("mallory").unwrap().kind, ActorKind::Injector);
    }

    #[test]
    fn parses_clock_steps_and_validates_node() {
        let toml = r#"
scenario = "MP-A-01"
[[nodes]]
label = "a"
port = 8401
[[actors]]
name = "alice"
node = "a"
batch = "a.jsonl"
[[clock]]
node = "a"
op = "advance"
value = "15d"
after = "invite_ready"
[[clock]]
node = "a"
op = "set"
value = "2099-01-01T00:00:00.000Z"
"#;
        let m = Manifest::parse(toml).unwrap();
        assert_eq!(m.clock.len(), 2);
        assert_eq!(m.clock[0].op, ClockOp::Advance);
        assert_eq!(m.clock[0].value, "15d");
        assert_eq!(m.clock[0].after.as_deref(), Some("invite_ready"));
        assert_eq!(m.clock[1].op, ClockOp::Set);
        assert!(m.clock[1].after.is_none());
    }

    #[test]
    fn rejects_clock_step_on_unknown_node() {
        let bad = r#"
scenario = "X"
[[nodes]]
label = "a"
port = 8401
[[actors]]
name = "alice"
node = "a"
batch = "a.jsonl"
[[clock]]
node = "ghost"
op = "advance"
value = "1d"
"#;
        let r = Manifest::parse(bad);
        assert!(r.is_err());
        assert!(format!("{:#}", r.unwrap_err()).contains("unknown node"));
    }

    #[test]
    fn parses_waits_and_groups_by_command() {
        let toml = r#"
scenario = "X"
[[nodes]]
label = "a"
port = 8401
[[actors]]
name = "alice"
node = "a"
batch = "a.jsonl"
[[actors]]
name = "bob"
node = "a"
batch = "b.jsonl"
[[waits]]
actor = "bob"
command = "b2"
key = "invite_ready"
[[waits]]
actor = "bob"
command = "b2"
key = "second_key"
"#;
        let m = Manifest::parse(toml).unwrap();
        let bob_waits = m.waits_of("bob");
        assert_eq!(bob_waits.get("b2").unwrap().len(), 2);
        assert!(m.waits_of("alice").is_empty());
    }

    #[test]
    fn rejects_actor_on_unknown_node() {
        let bad = r#"
scenario = "X"
[[nodes]]
label = "a"
port = 8401
[[actors]]
name = "alice"
node = "ghost"
batch = "alice.jsonl"
"#;
        let r = Manifest::parse(bad);
        assert!(r.is_err());
        assert!(format!("{:#}", r.unwrap_err()).contains("unknown node"));
    }

    #[test]
    fn rejects_unknown_top_level_field() {
        let bad = r#"
scenario = "X"
bogus = true
[[nodes]]
label = "a"
port = 8401
[[actors]]
name = "alice"
node = "a"
batch = "a.jsonl"
"#;
        assert!(Manifest::parse(bad).is_err());
    }

    #[test]
    fn rejects_empty_topology() {
        let bad = r#"
scenario = "X"
nodes = []
actors = []
"#;
        assert!(Manifest::parse(bad).is_err());
    }
}
