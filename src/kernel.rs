// Hey! I hope like my project, I'm 11 and coding on a tablet TYSM.
#![no_std]
#![no_main]

use limine::request::FramebufferRequest;
use core::arch::asm;

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_response.framebuffers().next() {
            let ptr = fb.addr() as *mut u32;
            for i in 0..(fb.width() * fb.height()) {
                unsafe { *ptr.add(i as usize) = 0x1a1b26; }
            }
        }
    }
    loop { asm!(
        "hlt"
        )}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {asm!("hlt")} }
