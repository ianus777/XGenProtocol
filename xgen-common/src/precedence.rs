// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! D-068 — CLI flag precedence over config file.
//!
//! The rule, structurally: **flag > env > config > default**. Uniform across
//! both binaries, applied to every setting that can be specified in more than
//! one place. See `DECISIONS.md` D-068 for the locked rationale.
//!
//! This module shipped from the CLI Precedence Audit (`tasks/CLI_PRECEDENCE_AUDIT.md`,
//! J-079) which surfaced two defect shapes: a flag-threading bug on
//! `xgen-node --port` and four parallel hardcoded subscriber-init blocks
//! ignoring `[logging].level`. The helpers below are the single resolution
//! site every flag-with-config-equivalent routes through after the audit.
//!
//! ## Helper scope
//!
//! - **In scope** (use `resolve_setting`): value-typed settings where the
//!   four precedence tiers are well-defined. Today: `--port` (`u16`),
//!   `--log-level` (`String`), `--node` (`String`), `--config <path>`
//!   (`PathBuf`). Future flags shadowing future config fields slot in.
//! - **Out of scope** (kept as-is): boolean toggles with no off-switch
//!   (`--local` one-way OR; `--quiet`, `--service` no config equivalent),
//!   mode selectors (`--ai-mode`), control-flow flags (`--check-config`,
//!   `--print-config`, `--pid`, `--ping`, `--health`, `--stop`,
//!   `--reload-config`), path-composition flags (`--instance` is resolved
//!   before config load, no config equivalent).
//!
//! ## clap interaction (D-068 rule)
//!
//! Do **not** set `clap::default_value` on any flag whose precedence is
//! resolved by `resolve_setting`. The helper distinguishes "operator passed
//! `--flag X`" from "operator did not pass the flag" via `Option<T>`; if clap
//! supplies a default, the flag is never `None` and the helper's flag tier
//! always wins, defeating the whole resolution chain. Let the flag be
//! `Option<T>` and resolve the default at the helper.

/// Resolve a value-typed setting from the four-tier D-068 precedence order.
///
/// Each upper tier wins when present (`Some(_)`); falls through to the next
/// tier when absent (`None`). The default is always supplied, so the return
/// type is `T`, not `Option<T>`.
///
/// Generic over `T: Clone` so the same call shape resolves `u16` (port),
/// `String` (log level, node endpoint), `PathBuf` (config path), or any
/// future value-typed setting. Semantics are identical in every case:
/// most-recent operator intent wins.
pub fn resolve_setting<T: Clone>(
    flag: Option<T>,
    env: Option<T>,
    config: Option<T>,
    default: T,
) -> T {
    flag.or(env).or(config).unwrap_or(default)
}

/// Resolve the effective log level per D-068, baking in `XGEN_LOG` awareness.
///
/// This is the only specialisation of `resolve_setting` shipped today. It
/// exists because four parallel subscriber-init sites (`service.rs`,
/// `ai_service.rs`, both `desktop.rs` files) were each implementing the same
/// flag>env>fallback dance with the env-var name hardcoded — and three of
/// the four were silently dropping the config tier (D-068 violation, J-079
/// §4.3.2).
///
/// Replaces an `EnvFilter::new("debug")` literal at every call site.
pub fn resolve_log_level(flag: Option<&str>, config_level: Option<&str>) -> String {
    let env = std::env::var("XGEN_LOG").ok();
    resolve_setting(
        flag.map(String::from),
        env,
        config_level.map(String::from),
        "debug".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_setting ────────────────────────────────────────────────────

    #[test]
    fn resolve_setting_flag_wins_over_env() {
        let r: u16 = resolve_setting(Some(9192u16), Some(9292), Some(9091), 8080);
        assert_eq!(r, 9192);
    }

    #[test]
    fn resolve_setting_env_wins_over_config() {
        let r: u16 = resolve_setting(None::<u16>, Some(9292), Some(9091), 8080);
        assert_eq!(r, 9292);
    }

    #[test]
    fn resolve_setting_config_wins_over_default() {
        let r: u16 = resolve_setting(None::<u16>, None::<u16>, Some(9091), 8080);
        assert_eq!(r, 9091);
    }

    #[test]
    fn resolve_setting_default_when_all_none() {
        let r: u16 = resolve_setting(None::<u16>, None::<u16>, None::<u16>, 8080);
        assert_eq!(r, 8080);
    }

    #[test]
    fn resolve_setting_generic_over_string() {
        let r: String = resolve_setting(
            Some("flag".to_string()),
            Some("env".to_string()),
            Some("config".to_string()),
            "default".to_string(),
        );
        assert_eq!(r, "flag");
    }

    #[test]
    fn resolve_setting_generic_over_u16() {
        let r: u16 = resolve_setting(Some(1u16), Some(2), Some(3), 4);
        assert_eq!(r, 1);
    }

    // ── resolve_log_level ──────────────────────────────────────────────────
    //
    // These tests touch process env (XGEN_LOG); they must serialise to avoid
    // cross-test contamination. The std test harness runs tests in parallel
    // by default. Pre-Phase-9 we relied on each test bracketing its work
    // with `std::env::remove_var` to start from a known-clean state — but
    // that doesn't prevent two threads from racing on `set_var`/`remove_var`
    // between each other's reads. Phase 9 Commit 2 (task file §3 Commit 2,
    // Lock Q3 option (i)) annotates the family with `#[serial_test::serial]`
    // so the four tests run one at a time within the xgen-common bucket.
    //
    // Each test still brackets its work with std::env::remove_var so the
    // pre-serial structural invariant survives — the annotation closes the
    // race; the bracketing keeps the test self-contained.

    fn with_xgen_log<F: FnOnce()>(value: Option<&str>, f: F) {
        std::env::remove_var("XGEN_LOG");
        if let Some(v) = value {
            std::env::set_var("XGEN_LOG", v);
        }
        f();
        std::env::remove_var("XGEN_LOG");
    }

    #[test]
    #[serial_test::serial(xgen_log_env)]
    fn resolve_log_level_flag_wins_over_env_xgen_log() {
        with_xgen_log(Some("error"), || {
            let r = resolve_log_level(Some("debug"), Some("warn"));
            assert_eq!(r, "debug");
        });
    }

    #[test]
    #[serial_test::serial(xgen_log_env)]
    fn resolve_log_level_env_wins_over_config() {
        with_xgen_log(Some("warn"), || {
            let r = resolve_log_level(None, Some("info"));
            assert_eq!(r, "warn");
        });
    }

    #[test]
    #[serial_test::serial(xgen_log_env)]
    fn resolve_log_level_config_wins_over_default() {
        with_xgen_log(None, || {
            let r = resolve_log_level(None, Some("error"));
            assert_eq!(r, "error");
        });
    }

    #[test]
    #[serial_test::serial(xgen_log_env)]
    fn resolve_log_level_default_debug_when_all_absent() {
        with_xgen_log(None, || {
            let r = resolve_log_level(None, None);
            assert_eq!(r, "debug");
        });
    }
}
