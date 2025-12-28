// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use bevy::prelude::*;

use crate::eprintln;

pub fn run() {
    App::new().add_systems(Startup, hello_world).run();
}

fn hello_world() {
    eprintln!("Hello World!");
}
