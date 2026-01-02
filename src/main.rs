#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate alloc;

use x86_64::VirtAddr;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::PageSize;
use x86_64::structures::paging::Size4KiB;

use crate::debug::serial;
use crate::debug::text;
use crate::trampoline::BootInfo;
use crate::trampoline::memory::HHDM_OFFSET;
use crate::trampoline::memory::get_pagetable;
use crate::trampoline::memory::map_framebuffers;
use core::{arch::asm, panic::PanicInfo};

use crate::trampoline::{gdt, interrupts, limine_requests, memory};

mod color;
mod debug;
mod trampoline;
mod util;
mod wordle;

/// # Setup order
/// 1. Exception handling
/// 2. Basic stack and heap
/// 3. Reclaim bootloader memory
/// 4. Initialize bevy, etc
// SAFETY:  must have a stable, unmangled symbol because it is called by Limine.
//          the ABI matches the expected System V calling convention.
#[unsafe(no_mangle)]
extern "C" fn trampoline_main() -> ! {
    debug_assert!(limine_requests::BASE_REVISION.is_supported());

    gdt::init();
    interrupts::init_idt();

    let mut frame_allocator = memory::initialize_paging();

    let page_table = get_pagetable();
    // SAFETY: get_pagetable returns address from CR3 which must be valid. HHDM_OFFSET is correct
    // as we mapped our own HHDM before with the call to initialize_paging.
    let mut offset_page_table =
        unsafe { OffsetPageTable::new(page_table, VirtAddr::new(HHDM_OFFSET)) };

    let framebuffers = map_framebuffers(&mut offset_page_table, &mut frame_allocator);

    // SAFETY: this switches the kernel stack, but then we call kernel_main after, which never
    // returns. Execution effectively starts afresh in kernel_main.
    unsafe {
        asm!(
            "mov rsp, {0}",
            in(reg) memory::STACK_BASE.start_address().as_u64() + memory::STACK_PAGES * Size4KiB::SIZE,
        );
    }

    kernel_main(BootInfo {
        framebuffers,
        frame_allocator,
    });
}

fn kernel_main(boot_info: BootInfo) -> ! {
    wordle::run(boot_info);

    hcf();
}

#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    eprintln!("{}", info);
    serial_println!("{}", info);
    hcf();
}

fn hcf() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
