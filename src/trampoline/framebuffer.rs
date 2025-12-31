// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::Infallible;

use embedded_graphics::{
    Pixel,
    pixelcolor::Rgb888,
    prelude::{DrawTarget, OriginDimensions, RgbColor, Size},
};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Page, PageSize, PhysFrame, Size2MiB},
};

use crate::util::page_from_addr;

pub struct Framebuffer {
    back_buf_addr: &'static mut u8,
    front_addr: &'static mut u8,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    length: u64,
}

impl Framebuffer {
    pub const FRAMEBUFFER_BASE: Page<Size2MiB> = page_from_addr(0x2222_2220_0000);

    pub fn write_pixel(&mut self, x: u64, y: u64, mut r: u8, mut g: u8, mut b: u8) {
        assert!(x < self.width);
        assert!(y < self.height);

        let mut pixel_value = 0u32;

        r = ((r as u16 * ((1u16 << self.red_mask_size) - 1) + 127) / 255) as u8;
        g = ((g as u16 * ((1u16 << self.green_mask_size) - 1) + 127) / 255) as u8;
        b = ((b as u16 * ((1u16 << self.blue_mask_size) - 1) + 127) / 255) as u8;

        pixel_value |= (r as u32 & ((1 << self.red_mask_size) - 1)) << self.red_mask_shift;
        pixel_value |= (g as u32 & ((1 << self.green_mask_size) - 1)) << self.green_mask_shift;
        pixel_value |= (b as u32 & ((1 << self.blue_mask_size) - 1)) << self.blue_mask_shift;

        let bytes_per_pixel = (self.bpp / 8) as u64;

        // SAFETY: address is properly mapped and aligned.
        // no concurrent writes since the function takes &mut self
        unsafe {
            core::ptr::write_volatile(
                core::ptr::from_mut(self.back_buf_addr)
                    .add((y * self.pitch + x * bytes_per_pixel) as usize)
                    .cast::<u32>(),
                pixel_value,
            );
        }
    }

    // TODO: Potentially test out techniques like dirty rectangles etc if this is not fast enough.
    pub fn flush(&mut self) {
        // SAFETY: Both buffers are made sure to be valid and properly aligned by the caller of the
        // constructor. They are also non-overlapping.
        unsafe {
            core::ptr::copy_nonoverlapping(
                core::ptr::from_mut(self.back_buf_addr),
                core::ptr::from_mut(self.front_addr),
                self.length as usize,
            );
        }
    }

    /// # SAFETY
    /// Both the back_buf_addr and front_addr must point to valid memory which is readable and
    /// writable for the length of the framebuffer. They also must be properly aligned.
    /// The two buffers must also not overlap.
    pub unsafe fn from_limine_framebuffer(
        framebuffer: &limine::framebuffer::Framebuffer,
        back_buf_addr: &'static mut u8,
        front_addr: &'static mut u8,
    ) -> Self {
        Self {
            back_buf_addr,
            front_addr,
            width: framebuffer.width(),
            height: framebuffer.height(),
            pitch: framebuffer.pitch(),
            bpp: framebuffer.bpp(),
            red_mask_size: framebuffer.red_mask_size(),
            red_mask_shift: framebuffer.red_mask_shift(),
            green_mask_size: framebuffer.green_mask_size(),
            green_mask_shift: framebuffer.green_mask_shift(),
            blue_mask_size: framebuffer.blue_mask_size(),
            blue_mask_shift: framebuffer.blue_mask_shift(),
            length: framebuffer.pitch() * framebuffer.height(),
        }
    }
}

impl OriginDimensions for Framebuffer {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl DrawTarget for Framebuffer {
    type Color = Rgb888;

    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= self.width as i32
                || coord.y >= self.height as i32
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
