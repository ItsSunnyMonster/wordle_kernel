#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;

use crate::debug::serial;
use crate::debug::text;
use core::{arch::asm, panic::PanicInfo};

use crate::trampoline::{gdt, interrupts, limine_requests, memory};

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

    memory::initialize_paging();
    // SAFETY: this function is called after paging is set up.
    unsafe {
        memory::init_allocator();
    }

    // SAFETY: this switches the kernel stack, but then we call kernel_main after, which never
    // returns. Execution effectively starts afresh in kernel_main.
    unsafe {
        asm!(
            "mov rsp, {0}",
            in(reg) memory::STACK_BASE + memory::STACK_SIZE,
        );
    }

    kernel_main();
}

extern "C" fn kernel_main() -> ! {
    wordle::run();

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
