// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::vec::Vec;
use bevy::ecs::resource::Resource;
use framebuffer::Framebuffer;

pub mod framebuffer;
pub mod gdt;
pub mod interrupts;
pub mod limine_requests;
pub mod memory;

#[derive(Resource)]
pub struct BootInfo {
    pub framebuffers: Vec<Framebuffer>,
}
