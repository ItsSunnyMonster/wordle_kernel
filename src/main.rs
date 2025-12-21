#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use limine::framebuffer::Framebuffer;

mod limine_structs;

struct FramebufferWriter<'a> {
    pub fb: &'a Framebuffer<'a>,
}

impl<'a> FramebufferWriter<'a> {
    pub fn new(framebuffer: &'a Framebuffer<'a>) -> Self {
        Self { fb: framebuffer }
    }

    fn write_pixel(&mut self, x: u64, y: u64, r: u8, g: u8, b: u8) {
        assert!(x < self.fb.width());
        assert!(y < self.fb.height());

        let mut pixel_value = 0u32;

        pixel_value |=
            (r as u32 & ((1 << self.fb.red_mask_size()) - 1)) << self.fb.red_mask_shift();
        pixel_value |=
            (g as u32 & ((1 << self.fb.green_mask_size()) - 1)) << self.fb.green_mask_shift();
        pixel_value |=
            (b as u32 & ((1 << self.fb.blue_mask_size()) - 1)) << self.fb.blue_mask_shift();

        let bytes_per_pixel = (self.fb.bpp() / 8) as u64;

        // SAFETY: address is properly mapped and aligned.
        // no concurrent writes since the function takes &mut self
        unsafe {
            // self.fb
            //     .addr()
            //     .add((y * self.fb.pitch() + x * bytes_per_pixel) as usize)
            //     .cast::<u32>()
            //     .write(pixel_value);
            core::ptr::write_volatile(
                self.fb
                    .addr()
                    .add((y * self.fb.pitch() + x * bytes_per_pixel) as usize)
                    .cast::<u32>(),
                pixel_value,
            );
        }
    }
}

// SAFETY:  must have a stable, unmangled symbol because it is called by Limine.
//          the ABI matches the expected System V calling convention.
#[unsafe(no_mangle)]
extern "C" fn kernel_main() -> ! {
    assert!(limine_structs::BASE_REVISION.is_supported());

    if let Some(framebuffer_response) = limine_structs::FRAMEBUFFER_REQUEST.get_response()
        && let Some(framebuffer) = framebuffer_response.framebuffers().next()
    {
        let mut writer = FramebufferWriter::new(&framebuffer);
        for y in 0..framebuffer.height() {
            for x in 0..framebuffer.width() {
                writer.write_pixel(x, y, 203, 166, 247);
            }
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
