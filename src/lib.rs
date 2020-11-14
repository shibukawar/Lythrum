#![no_std]
#![feature(asm)]
#![feature(start)]

use core::panic::PanicInfo;
use vga::{Screen, ScreenWriter};

mod asm;
mod fonts;
mod vga;

#[no_mangle]
fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

#[no_mangle]
#[start]
pub extern "C" fn haribote_os() -> ! {
    let mut screen = Screen::new();
    screen.init();
    let mut writer = ScreenWriter::new(screen, vga::Color::White, 10, 10);
    use core::fmt::Write;
    write!(writer, "ABC\nabc\n").unwrap();
    write!(writer, "10 * 3 = {}\n", 10 * 3).unwrap();
    write!(
        writer,
        "I saw a girl with a telescope."
    ).unwrap();
    loop {
        hlt()
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // println!("{}", info);
    loop {
        hlt()
    }
}
