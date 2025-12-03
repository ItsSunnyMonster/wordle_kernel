#![no_std]
#![no_main]

mod multiboot2;

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

global_asm!(include_str!("boot.s"));

#[panic_handler]
fn rust_panic(_info: &PanicInfo) -> ! {
    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}
