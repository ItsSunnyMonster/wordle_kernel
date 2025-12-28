// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        local_path.join("kernel.ld").display()
    )
}
