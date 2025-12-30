// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use bevy::prelude::*;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, RgbColor},
};

use crate::{serial_println, trampoline::BootInfo, util::InfallibleResultExt};

pub fn run(boot_info: BootInfo) {
    App::new()
        .insert_resource(boot_info)
        .add_systems(Startup, hello_world)
        .run();
}

fn hello_world(mut boot_info: ResMut<BootInfo>) {
    serial_println!("Hello World!");
    boot_info.framebuffers[0].clear(Rgb888::BLUE).infallible();
    boot_info.framebuffers[0].flush();
    serial_println!("After clear");
}
