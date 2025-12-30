// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::Infallible;

use embedded_graphics::{Pixel, pixelcolor::Rgb888, prelude::*};
use lazy_static::lazy_static;
use limine::framebuffer::Framebuffer;
use spin::Mutex;

use crate::{hcf, trampoline::limine_requests::HHDM_REQUEST};

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
    pub addr_override: Option<&'static mut u8>,
}

impl<'a> FramebufferWriter<'a> {
    pub fn new(framebuffer: Framebuffer<'a>) -> Self {
        Self {
            fb: framebuffer,
            addr_override: None,
        }
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
            let addr = if let Some(addr_override) = &mut self.addr_override {
                core::ptr::from_mut(*addr_override)
            } else {
                self.fb.addr()
            };

            // self.fb
            //     .addr()
            //     .add((y * self.fb.pitch() + x * bytes_per_pixel) as usize)
            //     .cast::<u32>()
            //     .write(pixel_value);
            core::ptr::write_volatile(
                addr.add((y * self.fb.pitch() + x * bytes_per_pixel) as usize)
                    .cast::<u32>(),
                pixel_value,
            );
        }
    }

    /// # SAFETY
    /// The same framebuffer should be located in virtual memory at the new HHDM offset.
    pub unsafe fn override_addr(&mut self, new_hhdm: u64) {
        let new_addr = self.fb.addr() as u64
            - HHDM_REQUEST
                .get_response()
                .expect("Response should be provided by Limine.")
                .offset()
            + new_hhdm;

        // SAFETY: The caller ensures that the new HHDM results in valid memory.
        self.addr_override = unsafe { Some(&mut *(new_addr as *mut u8)) };
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
            if coord.x >= self.fb.width() as i32
                || coord.y >= self.fb.height() as i32
                || coord.x < 0
                || coord.y < 0
            {
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
