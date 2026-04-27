// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// Website: https://www.alchemydump.com
// Licensed under the PolyForm Noncommercial License 1.0.0
// See: https://polyformproject.org/licenses/noncommercial/1.0.0/

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
pub const GIT_HASH: &str = env!("BUILD_GIT_HASH");

// Version format: [state].[section].[session].[build]
//   state:   0 = building, 1 = Phase 1+2 complete
//   section: spec section in progress (1-16, maps to 3.1-3.16)
//   session: increments each work session
//   build:   yymmdd-hhmm (auto, from build timestamp)
pub fn full_version() -> String {
    // BUILD_TIMESTAMP is "yyyy-mm-dd hh:mm:ss UTC" — extract yymmdd-hhmm
    let build = build_short(BUILD_TIMESTAMP);
    format!("{}.{}", VERSION, build)
}

fn build_short(ts: &str) -> String {
    // "2026-04-27 10:50:56 UTC" -> "260427-1050"
    let chars: Vec<char> = ts.chars().collect();
    if chars.len() < 16 {
        return "000000-0000".to_string();
    }
    let yy = &ts[2..4];
    let mm = &ts[5..7];
    let dd = &ts[8..10];
    let hh = &ts[11..13];
    let mi = &ts[14..16];
    format!("{}{}{}-{}{}", yy, mm, dd, hh, mi)
}

pub fn print_banner(binary_name: &str) {
    println!("----------------------------------------");
    println!("  {}  v{}  ({})", binary_name, full_version(), GIT_HASH);
    println!("  Built: {}", BUILD_TIMESTAMP);
    println!("  XGen Protocol — Phase 1");
    println!("----------------------------------------");
}
