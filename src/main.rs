#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::{arch::asm, panic::PanicInfo};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

use crate::{rendering::FRAMEBUFFER, util::InfallibleResultExt};

mod gdt;
mod interrupts;
mod limine_requests;
mod memory;
mod rendering;
mod serial;
mod text;
mod util;

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

    // SAFETY: this assembly snippet calls kernel_main which then never returns. Execution
    // effectively starts afresh in kernel_main.
    unsafe {
        asm!(
            "mov rsp, {0}",
            "call {1}",
            in(reg) memory::STACK_BASE + memory::STACK_SIZE,
            sym kernel_main,
            options(noreturn)
        );
    }
}

extern "C" fn kernel_main() -> ! {
    FRAMEBUFFER
        .lock()
        .clear(Rgb888::new(24, 24, 37))
        .infallible();

    // // Trigger stack overflow
    // #[allow(unconditional_recursion)]
    // fn stack_overflow() {
    //     stack_overflow();
    //     unsafe {
    //         core::ptr::read_volatile(&0);
    //     }
    // }
    //
    // stack_overflow();

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 0x43;
    // }

    serial_println!("Hello world!");

    eprintln!("Hello world!");

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
