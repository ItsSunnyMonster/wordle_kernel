use core::fmt;

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X18_BOLD},
    pixelcolor::Rgb888,
    prelude::Point,
    text::renderer::TextRenderer,
};
use spin::Mutex;

use crate::{rendering::FRAMEBUFFER, util::InfallibleResultExt};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DEBUG_WRITER: Mutex<DebugWriter> = Mutex::new(DebugWriter::new());
}

pub struct DebugWriter {
    style: MonoTextStyle<'static, Rgb888>,
    position: Point,
}

impl DebugWriter {
    pub fn new() -> Self {
        Self {
            style: MonoTextStyle::new(&FONT_9X18_BOLD, Rgb888::new(243, 139, 168)),
            position: Point::new(20, 30),
        }
    }

    pub fn write(&mut self, s: &str) {
        for c in s.chars() {
            // New line or line wrap
            if c == '\n'
                || self.position.x
                    >= FRAMEBUFFER.lock().fb.width() as i32
                        - 20
                        - self.style.font.character_size.width as i32
            {
                self.position.y += self.style.line_height() as i32;
                self.position.x = 20;
                continue;
            }

            let mut tmp = [0; 4];

            self.position = self
                .style
                .draw_string(
                    c.encode_utf8(&mut tmp),
                    self.position,
                    embedded_graphics::text::Baseline::Bottom,
                    &mut *FRAMEBUFFER.lock(),
                )
                .infallible();
        }
    }
}

impl fmt::Write for DebugWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::text::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    () => (eprint!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    DEBUG_WRITER
        .lock()
        .write_fmt(args)
        .expect("Writing to DebugWriter never returns an error.");
}
