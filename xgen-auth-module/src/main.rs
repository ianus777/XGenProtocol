// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! `xgen-auth-module` CLI — minimal operator surface for the Tier-1 reference
//! Auth Module (M10.2). Two verbs only (not a product CLI): `keygen` makes the
//! module's keypair + prints its `AuthModuleXgid` URI (the issuer URI an operator
//! hands to `auth-module register`); `issue` signs a Tier-1 `TrustAssertion` for
//! an Identity (offline — no live endpoint, M10.2-D1).

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{Duration, SecondsFormat, Utc};
use clap::{Parser, Subcommand};
use xgen_auth_module::{issue_tier1, module_xgid};
use xgen_core::identity::keypair;

#[derive(Parser)]
#[command(name = "xgen-auth-module", about = "Tier-1 reference Auth Module (offline signer)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate the module keypair (encrypted at rest) and print its issuer URI.
    Keygen {
        /// Where to write the encrypted keypair (e.g. xgen-auth-module_keypair.enc).
        #[arg(long)]
        out: PathBuf,
        /// Passphrase for the keypair file (empty = unprotected, dev default).
        #[arg(long, default_value = "")]
        passphrase: String,
    },
    /// Issue a signed Tier-1 Trust Assertion for an Identity.
    Issue {
        /// The module keypair file to sign with.
        #[arg(long)]
        keypair: PathBuf,
        /// Passphrase for the keypair file.
        #[arg(long, default_value = "")]
        passphrase: String,
        /// The Identity this assertion is for (its `xgen://pubkey/ed25519:` URI).
        #[arg(long)]
        identity: String,
        /// Validity window in days from now.
        #[arg(long, default_value_t = 365)]
        valid_days: u32,
        /// Write the assertion JSON here (default: stdout).
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Keygen { out, passphrase } => {
            let key = keypair::generate();
            keypair::save(&key, &out, &passphrase)
                .map_err(|e| anyhow::anyhow!("save keypair: {e}"))?;
            println!("{}", module_xgid(&key));
            eprintln!("keypair written to {}", out.display());
        }
        Command::Issue {
            keypair: kp_path,
            passphrase,
            identity,
            valid_days,
            out,
        } => {
            let key = keypair::load(&kp_path, &passphrase)
                .map_err(|e| anyhow::anyhow!("load keypair: {e}"))?;
            let valid_until = (Utc::now() + Duration::days(valid_days as i64))
                .to_rfc3339_opts(SecondsFormat::Millis, true);
            let assertion = issue_tier1(&key, &identity, &valid_until);
            let json =
                serde_json::to_string_pretty(&assertion).context("serialise assertion")?;
            match out {
                Some(path) => {
                    std::fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
                    eprintln!("assertion written to {}", path.display());
                }
                None => println!("{json}"),
            }
        }
    }
    Ok(())
}
