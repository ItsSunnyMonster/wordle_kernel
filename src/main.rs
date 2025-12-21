#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

use crate::{rendering::FRAMEBUFFER, util::InfallibleResultExt};

mod limine_structs;
mod rendering;
mod text;
mod util;

// SAFETY:  must have a stable, unmangled symbol because it is called by Limine.
//          the ABI matches the expected System V calling convention.
#[unsafe(no_mangle)]
extern "C" fn kernel_main() -> ! {
    assert!(limine_structs::BASE_REVISION.is_supported());

    FRAMEBUFFER
        .lock()
        .clear(Rgb888::new(24, 24, 37))
        .infallible();

    eprintln!("Hello World!");
    eprint!("This is supposedly an error :3");
    eprintln!("");

    eprintln!(
        "AHSGKFASDGLKHDKFSDHGLDKFJASGHLKFJKASLGHSDLFASKLGDFJAKSLGHKDLFJASKLGHSKDLFJAKLSGHKLFHJKSHGKLJDFKAHSGKLJDFAKLSHGKLDFGJHAKLSGHDKLJFKLASHGASDKLHKLSDJFKLASHDGLKJFKLDHKSLDHJFKLASHGKLJFKLASHDGKLJDFKLASHGKLDJFKLASHGDKLJFKLASHGDKLJFKLASHGDKLJHGKLASHGKLDJFKLASGHDKLHFGKLASGHKLDHGKLASDHFKLASJKLDGASKLDFAKLSDGKLFHAKLSDGKLDFHAKLSDGJKLFJKLSDHGKLFHJKLASDHGKLSDJF"
    );

    panic!("HOLY SHIT WE PANICKED!!!");
}

#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    eprintln!("{}", info);
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
