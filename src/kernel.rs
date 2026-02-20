#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

// Standard vibe macros
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
extern crate alloc;

use core::arch::asm;
use core::fmt::{self, Write};
use limine::BaseRevision;
use limine::request::{FramebufferRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker};
// Exact paths from your documentation
use limine::memory_map::MemoryMapEntryType;
use linked_list_allocator::LockedHeap;
use spleen_font::FONT_16X32;
use vibe_framebuffer::{Cursor, Font};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(start: usize, size: usize) {
    unsafe { ALLOCATOR.lock().init(start as *mut u8, size); }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("VIBE OS: Heap Allocation Error - Layout: {:?}", layout);
}

static mut UI_CURSOR: Option<Cursor> = None;

pub fn _print(args: fmt::Arguments) {
    unsafe { if let Some(ref mut cursor) = UI_CURSOR { let _ = cursor.write_fmt(args); } }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    let memmap_response = MEMMAP_REQUEST.get_response().as_ref().expect("Memmap failed");

    // MANUAL POINTER ARITHMETIC (Bypassing broken methods)
    // Your source says: pub entries: *const *const MemoryMapEntry
    let entries_ptr = memmap_response.entries;
    let entry_count = memmap_response.entry_count as usize;
    let mut heap_addr: u64 = 0;
    let heap_size = 32 * 1024 * 1024;

    unsafe {
        let entries = core::slice::from_raw_parts(entries_ptr, entry_count);
        for entry_ptr in entries {
            let entry = &**entry_ptr;
            // Matches PascalCase 'Usable' and field 'entry_type' or 'typ'
            // Based on docs, it is likely 'entry_type'
            if entry.entry_type == MemoryMapEntryType::Usable && entry.length >= heap_size as u64 {
                heap_addr = entry.base;
                break;
            }
        }
    }

    if heap_addr == 0 { panic!("No usable RAM found for heap!"); }
    init_heap(heap_addr as usize, heap_size);

    unsafe {
        if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
            if let Some(fb) = fb_response.framebuffers().next() {
                let font = Font::new(FONT_16X32);
                let mut cursor = Cursor::new(fb.addr() as *mut u32, core::ptr::null_mut(), fb.width(), fb.height());
                cursor.font = Some(font);
                cursor.clear(0x1a1b26);
                UI_CURSOR = Some(cursor);
            }
        }
    }

    println!("Vibe OS: Kernel Space Initialized.");
    
    loop { unsafe { asm!("hlt") } }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.color_fg = 0xf7768e;
            cursor.x = 0;
            println!("\n[ VIBE OS FATAL ERROR ]\n{}", info);
        }
    }
    loop { unsafe { asm!("hlt") } }
}
