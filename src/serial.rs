use core::fmt;
use core::fmt::Write;

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    static ref SERIAL: Mutex<SerialPort> = {
        // SAFETY: 0x3F8 is the COM1 serial port's address. We should have permission to write to
        // it.
        let mut port = unsafe { uart_16550::SerialPort::new(0x3F8) };
        port.init();

        Mutex::new(port)
    };
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_serial_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _serial_print(args: fmt::Arguments) {
    SERIAL
        .lock()
        .write_fmt(args)
        .expect("Writing to serial port never returns an error.");
}
