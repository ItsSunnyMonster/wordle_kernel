#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

use crate::{interrupts::init_idt, rendering::FRAMEBUFFER, util::InfallibleResultExt};

mod interrupts;
mod limine_structs;
mod rendering;
mod text;
mod util;

// SAFETY:  must have a stable, unmangled symbol because it is called by Limine.
//          the ABI matches the expected System V calling convention.
#[unsafe(no_mangle)]
extern "C" fn kernel_main() -> ! {
    assert!(limine_structs::BASE_REVISION.is_supported());

    init_idt();

    FRAMEBUFFER
        .lock()
        .clear(Rgb888::new(24, 24, 37))
        .infallible();

    // UNSAFETY: intentional page fault to test interrupt handling.
    unsafe {
        *(0xdeadbeef as *mut u8) = 0x43;
    }

    eprintln!("Hello world!");

    hcf();
}

#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    eprintln!("{}", info);
    hcf();
}

fn hcf() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
