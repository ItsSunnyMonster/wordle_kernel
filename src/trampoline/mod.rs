// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::vec::Vec;
use bevy::ecs::resource::Resource;
use framebuffer::Framebuffer;

use crate::trampoline::memory::allocators::ProperFrameAllocator;

pub mod framebuffer;
pub mod gdt;
pub mod happy_new_year;
pub mod interrupts;
pub mod limine_requests;
pub mod memory;

#[derive(Resource)]
#[allow(dead_code)]
pub struct BootInfo {
    pub framebuffers: Vec<Framebuffer>,
    pub frame_allocator: ProperFrameAllocator,
}
