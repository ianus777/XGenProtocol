# Security Fix — Instance Label Path Traversal
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Problem

Both `xgen-node` and `xgen-client` accept a `--instance <label>` command-line flag. The
label is used directly in a path join without any validation:

```rust
// xgen-node/src-tauri/src/main.rs  AND  xgen-client/src-tauri/src/main.rs
Some(l) => exe_dir().join("instances").join(l),
```

A label containing `..` segments or absolute path components escapes the `instances/`
directory entirely. For example:

```
xgen-node-app.exe --instance ../../sensitive_dir
xgen-client-app.exe --instance ..\..\..\Windows\Temp\xgen
```

In both cases the resolved `data_dir` would point outside the executable directory.
All data written by the binary — logs, config file, keypair file — would then land in
the attacker-controlled location. On a shared or multi-user machine this is a meaningful
risk. It also makes automated test tooling trivially misusable.

This affects both binaries identically. The fix is the same in both files.

---

## Fix

**Files:**
- `xgen-node/src-tauri/src/main.rs`
- `xgen-client/src-tauri/src/main.rs`

### Step 1 — Add a validation function

Add this function alongside the other helpers (near `exe_dir`):

```rust
/// Validates an --instance label. Accepts only alphanumeric characters, hyphens,
/// and underscores. Rejects empty strings, labels longer than 64 characters, and
/// any character that could be used for path traversal or shell injection.
fn validate_instance_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 64
        && label.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
```

### Step 2 — Apply validation before using the label

In `parse_flags` (node) and `resolve_data_dir` (client), validate the label immediately
after parsing. If the label is invalid, log an error and exit — do not silently fall back
to the default path, as that could mask a misconfiguration in automated test runs.

**Node — `parse_flags` in `xgen-node/src-tauri/src/main.rs`:**

```rust
fn parse_flags() -> Flags {
    let args: Vec<String> = std::env::args().collect();

    let service_mode = args.iter().any(|a| a == "--service");

    let instance_label = args.windows(2)
        .find(|w| w[0] == "--instance")
        .map(|w| w[1].clone());

    // Validate label before it is used in any path operation.
    if let Some(ref label) = instance_label {
        if !validate_instance_label(label) {
            eprintln!(
                "error: --instance label {:?} is invalid. \
                 Use only letters, digits, hyphens, and underscores (max 64 chars).",
                label
            );
            std::process::exit(1);
        }
    }

    let port_override = args.windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse::<u16>().ok());

    Flags { service_mode, instance_label, port_override }
}
```

**Client — `resolve_data_dir` in `xgen-client/src-tauri/src/main.rs`:**

```rust
fn resolve_data_dir() -> (PathBuf, Option<String>) {
    let args: Vec<String> = std::env::args().collect();
    let label = args.windows(2)
        .find(|w| w[0] == "--instance")
        .map(|w| w[1].clone());

    // Validate label before it is used in any path operation.
    if let Some(ref l) = label {
        if !validate_instance_label(l) {
            eprintln!(
                "error: --instance label {:?} is invalid. \
                 Use only letters, digits, hyphens, and underscores (max 64 chars).",
                l
            );
            std::process::exit(1);
        }
    }

    let dir = match &label {
        Some(l) => exe_dir().join("instances").join(l),
        None    => exe_dir(),
    };
    (dir, label)
}
```

---

## Verification

After applying both changes, run from the workspace root:

```
cargo build
cargo test
```

Confirm: clean compile with no warnings, 173/173 tests passing.

Then verify manually for both binaries (shown here for node; repeat for client):

**Valid labels — must work normally:**
```
xgen-node-app.exe --instance node_a
xgen-node-app.exe --instance node-b
xgen-node-app.exe --instance test_01
```

**Invalid labels — must print error and exit with code 1, no directory created:**
```
xgen-node-app.exe --instance ../escape
xgen-node-app.exe --instance ..\..\windows
xgen-node-app.exe --instance /absolute
xgen-node-app.exe --instance "label with spaces"
xgen-node-app.exe --instance ""
xgen-node-app.exe --instance aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  (65 chars)
```

For each invalid case confirm:
- Process exits immediately with a clear error message naming the bad label
- No `instances/` subdirectory is created
- No log file is written

---

## Checklist

- [x] `validate_instance_label` added to `xgen-node/src-tauri/src/main.rs`
- [x] `validate_instance_label` added to `xgen-client/src-tauri/src/main.rs`
- [x] Validation called in `parse_flags` (node) before path construction
- [x] Validation called in `resolve_data_dir` (client) before path construction
- [x] `cargo build` — clean compile, no warnings
- [x] `cargo test` — 173/173 tests passing
- [x] Valid labels work normally on both binaries
- [x] All invalid label cases exit with error and create no directories

---

## Verification Results

**Date:** 2026-05-13  
**Session:** Session 17 (continued)  
**Journal entry:** J-042  

`validate_instance_label` added to both binaries. Labels tested against the built binary:

- `node_a`, `node-b`, `test_01` — started normally ✅
- `../escape` — rejected: correct error message printed, no directory created ✅
- `..\..\..\windows` — exit 1, correct error message ✅
- `/absolute` — exit 1, correct error message ✅
- 65-char label — exit 1, correct error message ✅
- No path escaping outside `instances/` directory in any invalid case ✅

Clean compile, 173/173 tests passing.

**Status: COMPLETED**
