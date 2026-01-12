#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
fn print(str: &string) {
    let uart = 0x1000_0000 as *mut u8;
    unsafe {
    for &c in str {
        core::ptr::write_volatile(uart, c)
    }
}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    print("NISH");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) {
    print("PANIC");
    loop {}
}