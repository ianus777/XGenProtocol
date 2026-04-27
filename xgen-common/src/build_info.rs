// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// Website: https://www.alchemydump.com
// Licensed under the PolyForm Noncommercial License 1.0.0
// See: https://polyformproject.org/licenses/noncommercial/1.0.0/

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
pub const GIT_HASH: &str = env!("BUILD_GIT_HASH");

pub fn print_banner(binary_name: &str) {
    println!("----------------------------------------");
    println!("  {}  v{}  ({})", binary_name, VERSION, GIT_HASH);
    println!("  Built: {}", BUILD_TIMESTAMP);
    println!("  XGen Protocol — Phase 1");
    println!("----------------------------------------");
}
