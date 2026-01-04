#![no_std]
#![feature(abi_x86_interrupt)]

// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate alloc;

use crate::trampoline::BootInfo;

use crate::trampoline::{gdt, limine_requests};

pub mod color;
pub mod debug;
pub mod trampoline;
pub mod util;
pub mod wordle;

pub fn kernel_main(boot_info: BootInfo) -> ! {
    wordle::run(boot_info);

    hcf();
}

pub fn hcf() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
