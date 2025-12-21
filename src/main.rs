#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X18_BOLD},
    pixelcolor::Rgb888,
    prelude::*,
    text::Text,
};

use crate::{rendering::FramebufferWriter, util::InfallibleResultExt};

mod limine_structs;
mod rendering;
mod util;

// SAFETY:  must have a stable, unmangled symbol because it is called by Limine.
//          the ABI matches the expected System V calling convention.
#[unsafe(no_mangle)]
extern "C" fn kernel_main() -> ! {
    assert!(limine_structs::BASE_REVISION.is_supported());

    if let Some(framebuffer_response) = limine_structs::FRAMEBUFFER_REQUEST.get_response()
        && let Some(framebuffer) = framebuffer_response.framebuffers().next()
    {
        let mut writer = FramebufferWriter::new(&framebuffer);
        writer.clear(Rgb888::new(24, 24, 37)).infallible();

        let style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb888::new(243, 139, 168));
        Text::new(
            "Hello World!\nThis is supposedly an error.",
            Point::new(20, 30),
            style,
        )
        .draw(&mut writer)
        .infallible();
    }

    hcf();
}

#[panic_handler]
fn rust_panic(_info: &PanicInfo) -> ! {
    hcf();
}

fn hcf() -> ! {
    loop {
        // SAFETY: the program is halted.
        unsafe {
            asm!("hlt");
        }
    }
}
