#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

mod limine_structs;

// SAFETY:  must have a stable, unmangled symbol because it is called by Limine.
//          the ABI matches the expected System V calling convention.
#[unsafe(no_mangle)]
extern "C" fn kernel_main() -> ! {
    assert!(limine_structs::BASE_REVISION.is_supported());

    if let Some(framebuffer_response) = limine_structs::FRAMEBUFFER_REQUEST.get_response()
        && let Some(framebuffer) = framebuffer_response.framebuffers().next()
    {
        for i in 0..100_u64 {
            // Calculate the pixel offset using the framebuffer information we obtained above.
            // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
            let pixel_offset = i * framebuffer.pitch() + i * 4;

            // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
            unsafe {
                framebuffer
                    .addr()
                    .add(pixel_offset as usize)
                    .cast::<u32>()
                    .write(0xFFFFFFFF)
            };
        }
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
