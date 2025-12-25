use core::convert::Infallible;

use embedded_graphics::{Pixel, pixelcolor::Rgb888, prelude::*};
use lazy_static::lazy_static;
use limine::framebuffer::Framebuffer;
use spin::Mutex;

use crate::hcf;

lazy_static! {
    pub static ref DEBUG_FRAMEBUFFER: Mutex<FramebufferWriter<'static>> = {
        if let Some(framebuffer_response) =
            crate::limine_requests::FRAMEBUFFER_REQUEST.get_response()
            && let Some(framebuffer) = framebuffer_response.framebuffers().next()
        {
            let writer = FramebufferWriter::new(framebuffer);
            Mutex::new(writer)
        } else {
            hcf();
        }
    };
}

pub struct FramebufferWriter<'a> {
    pub fb: Framebuffer<'a>,
}

impl<'a> FramebufferWriter<'a> {
    pub fn new(framebuffer: Framebuffer<'a>) -> Self {
        Self { fb: framebuffer }
    }

    pub fn write_pixel(&mut self, x: u64, y: u64, r: u8, g: u8, b: u8) {
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

impl OriginDimensions for FramebufferWriter<'_> {
    fn size(&self) -> Size {
        Size::new(self.fb.width() as u32, self.fb.height() as u32)
    }
}

impl DrawTarget for FramebufferWriter<'_> {
    // NOTE: Not sure if this would work on every framebuffer. I also don't know how to support
    // more than one color type.
    type Color = Rgb888;

    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= self.fb.width() as i32 || coord.y >= self.fb.height() as i32 {
                continue;
            }

            self.write_pixel(
                coord.x as u64,
                coord.y as u64,
                color.r(),
                color.g(),
                color.b(),
            );
        }

        Ok(())
    }
}
