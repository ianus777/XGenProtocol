// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! C1 lifecycle smoke (M9, `#[ignore]` — spawns a real binary).
//!
//! Validates the C1 pieces end-to-end against the real `xgen-node.exe`:
//! locate → `init` → `--service` spawn → connect the `.aicontrol` pipe →
//! `state` round-trip → kill-on-drop teardown. Out-of-band from the fast unit
//! suite; run explicitly:
//!
//! ```text
//! cargo build -p xgen-node -p xgen-client
//! cargo test -p xgen-mptest --test c1_lifecycle -- --ignored --nocapture
//! ```

use std::time::Duration;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::process::{instance_label, ManagedProcess};

#[tokio::test]
#[ignore = "heavy: spawns the real xgen-node binary; run with --ignored"]
async fn c1_spawn_node_and_aicontrol_state_roundtrip() {
    let bins = binloc::locate().expect("locate built binaries");

    let label = instance_label("C1", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &label, 8455, true, None)
        .expect("init + spawn node --service");

    // Connect to the node's `.aicontrol` pipe (retries until the server is up).
    let mut client = AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect to node aicontrol pipe");

    // The `state` verb is the in-process live node-state read (M9-D4 oracle
    // source); here it just proves the drive path works.
    let reply = client.send_verb("state").await.expect("send state");
    assert!(
        reply.is_ok(),
        "expected Ok reply to `state`, got: {reply:?}"
    );
    // `cmd` is echoed at the envelope level (sibling of `data`).
    assert_eq!(reply.cmd(), Some("state"));
    // The node `state` data carries live node-core fields (M9-D4 oracle source).
    let data = reply.data().expect("state reply carries data");
    assert!(
        data.get("node_id").and_then(|v| v.as_str()).is_some(),
        "state data missing node_id: {data:?}"
    );
    assert!(
        data.get("hosted_spaces").is_some(),
        "state data missing hosted_spaces: {data:?}"
    );

    // Teardown: dropping `node` kills the process and removes its instance dir.
    drop(client);
    drop(node);

    // Give the OS a beat to release the pipe before the test harness exits.
    tokio::time::sleep(Duration::from_millis(50)).await;
}
