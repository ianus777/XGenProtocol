// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Bootstrap capability declaration and announcement extension (spec 3.14.1).
//
// A Node declares Bootstrap capability by adding "xgen.bootstrap" to
// capabilities.extensions in its NodeAnnouncement and populating the
// bootstrap_info field.

use serde::{Deserialize, Serialize};

use crate::node::announcement::NodeAnnouncement;

/// The capability token for Bootstrap Nodes (spec 3.14.1).
pub const BOOTSTRAP_CAPABILITY: &str = "xgen.bootstrap";

/// Bootstrap-specific announcement extension (spec 3.14.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapInfo {
    /// HTTPS URL where this Bootstrap Node's directory can be queried.
    pub directory_url: String,
    /// Whether this Bootstrap Node accepts new Node registrations.
    pub accepts_registrations: bool,
    /// Geographic region — operator-declared, for diversity routing.
    pub region: String,
    /// Human-readable operator name — informational only.
    pub operator: String,
}

/// Declare Bootstrap capability on an announcement.
///
/// Adds `"xgen.bootstrap"` to `capabilities.extensions` and sets `bootstrap_info`.
/// Re-signs the announcement with the provided signing key.
pub fn declare_bootstrap(
    ann: &mut NodeAnnouncement,
    bootstrap_info: BootstrapInfo,
    signing_key: &ed25519_dalek::SigningKey,
) {
    // Add capability token if not already present.
    if !ann.capabilities.extensions.contains(&BOOTSTRAP_CAPABILITY.to_string()) {
        ann.capabilities.extensions.push(BOOTSTRAP_CAPABILITY.to_string());
    }
    ann.bootstrap_info = Some(bootstrap_info);

    // Re-sign after modification.
    let canonical = ann.canonical_json();
    ann.signature = crate::crypto::signing::sign(signing_key, canonical.as_bytes());
}

/// Return true if the announcement declares the Bootstrap capability.
pub fn has_bootstrap_capability(ann: &NodeAnnouncement) -> bool {
    ann.capabilities.extensions.contains(&BOOTSTRAP_CAPABILITY.to_string())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::keypair;
    use crate::node::announcement::NodeAnnouncement;

    fn make_info() -> BootstrapInfo {
        BootstrapInfo {
            directory_url: "https://bootstrap.example.com/xgen-directory".to_string(),
            accepts_registrations: true,
            region: "EU".to_string(),
            operator: "Test Foundation".to_string(),
        }
    }

    #[test]
    fn bootstrap_capability_declared_in_announcement() {
        let key = keypair::generate();
        let mut ann = NodeAnnouncement::generate(&key, "wss://node.example.com/xgen", None, 1);

        // Not a Bootstrap Node yet.
        assert!(!has_bootstrap_capability(&ann));
        assert!(ann.bootstrap_info.is_none());

        // Declare Bootstrap capability.
        declare_bootstrap(&mut ann, make_info(), &key);

        assert!(has_bootstrap_capability(&ann));
        assert!(ann.bootstrap_info.is_some());
        let info = ann.bootstrap_info.as_ref().unwrap();
        assert_eq!(info.directory_url, "https://bootstrap.example.com/xgen-directory");
        assert!(info.accepts_registrations);
        assert_eq!(info.region, "EU");

        // Announcement signature must still verify after mutation.
        assert!(ann.verify().is_ok());
    }

    #[test]
    fn declare_bootstrap_is_idempotent() {
        let key = keypair::generate();
        let mut ann = NodeAnnouncement::generate(&key, "wss://node.example.com/xgen", None, 1);
        declare_bootstrap(&mut ann, make_info(), &key);
        declare_bootstrap(&mut ann, make_info(), &key);

        // Capability should appear exactly once.
        let count = ann
            .capabilities
            .extensions
            .iter()
            .filter(|c| c.as_str() == BOOTSTRAP_CAPABILITY)
            .count();
        assert_eq!(count, 1);
    }
}
