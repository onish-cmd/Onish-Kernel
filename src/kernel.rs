// Hey! I hope like my project, I'm 11 and coding on a tablet TYSM.
#![no_std]
#![no_main]

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

extern crate limine;
use limine::request::FramebufferRequest;
use core::arch::asm;
use vibe_framebuffer::Cursor;
use spleen_font::FONT_16X32;
use core::fmt::{self, Write};
use core::fmt;

impl fmt::Write for Cursor {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.draw_char(c);
        }
        Ok(())
    }
}

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static mut UI_CURSOR: Option<Cursor> = None;

pub fn _print(args: fmt::Arguments) {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.write_fmt(args).unwrap();
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe { if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_response.framebuffers().next() {
            let font = vibe_framebuffer::Font::new(FONT_16X32);
            UI_CURSOR = Some(Cursor::new(
                fb.addr() as *mut u32, 
                fb.width(), 
                fb.height()
            ));
            cursor.font = Some(font);
        }
        }
    }
    println!("Vibe OS is alive!");
    clear_screen(0x001A1B26);
    loop { unsafe {
            asm!(
                "hlt"
            )
        }
    }
}

pub fn clear_screen(color: u32) {
    unsafe { 
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.clear(color);
        }   
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { 
    println!("KERNEL PANIC: {}", info);
    loop { 
        unsafe { 
            asm!("hlt")
        } 
    }
}
