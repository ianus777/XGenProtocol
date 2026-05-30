// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Federation module — handshake state machine and relationship registry (spec 3.4).

pub mod federation_policy;
pub mod handshake;
pub mod pending_queue;
pub mod registry;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::registry::{FederationRegistry, FederationRelationship};
    use super::handshake::FederationSession;
    use chrono::{SecondsFormat, Utc};
    use xgen_common::xgid::{NodeXgid, Xgid};

    fn nd_xgid(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }

    /// Verify that the registry correctly stores and retrieves a session built
    /// from a completed FederationSession.
    #[test]
    fn registry_stores_session_and_round_trips() {
        let session = FederationSession {
            peer_node_id: "xgen://pubkey/ed25519:AAAA".to_string(),
            session_id: "xgen://hash/sha256:sess".to_string(),
            negotiated_serialisation: "json".to_string(),
            negotiated_version: "0.1".to_string(),
            shared_spaces: vec!["xgen://hash/sha256:space1".to_string()],
            peer_url: None,
            peer_tips: BTreeMap::new(),
        };

        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let rel = FederationRelationship::from_session(&session, ts);

        let mut reg = FederationRegistry::new();
        reg.upsert(rel);

        let stored = reg.get(&nd_xgid("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert_eq!(stored.session_id, "xgen://hash/sha256:sess");
        assert_eq!(stored.negotiated_serialisation, "json");
    }
}
