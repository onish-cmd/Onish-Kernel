#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate limine;

use core::arch::asm;
use core::fmt::{self, Write};
use limine::request::{
    FramebufferRequest, HhdmRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker,
    StackSizeRequest,
};
use limine::BaseRevision;
use limine::memory_map::EntryType;
use linked_list_allocator::LockedHeap;
use spleen_font::FONT_16X32;
use vibe_framebuffer::{Cursor, Font};

// --- Limine Requests ---

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(0x10000); // 64KB Stack

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

// --- Global Allocator & UI ---

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static mut UI_CURSOR: Option<Cursor> = None;

// --- Macros ---

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            let _ = cursor.write_fmt(args);
        }
    }
}

// --- Initialization Functions ---

pub fn init_heap(start: usize, size: usize) {
    unsafe {
        ALLOCATOR.lock().init(start as *mut u8, size);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("VIBE OS: Heap Allocation Error - Layout: {:?}", layout);
}

// --- Entry Point ---

#[no_mangle]
pub extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    // 1. Get HHDM Offset (Required to turn physical RAM addresses into virtual ones)
    let hhdm_offset = HHDM_REQUEST
        .get_response()
        .as_ref()
        .expect("VIBE ERROR: HHDM failed")
        .offset();

    // 2. Dynamic Heap Search
    let memmap_response = MEMMAP_REQUEST
        .get_response()
        .as_ref()
        .expect("VIBE ERROR: Memmap failed");

    let heap_size = 16 * 1024 * 1024; // 16MB Dynamic Heap
    let mut heap_virt_addr: u64 = 0;

    for entry in memmap_response.entries() {
        // Only use USABLE RAM, and avoid the first 16MB to stay clear of Kernel/BIOS
        if entry.entry_type == EntryType::USABLE && entry.base >= 0x1000000 {
            if entry.length >= heap_size as u64 {
                heap_virt_addr = entry.base + hhdm_offset;
                break;
            }
        }
    }

    if heap_virt_addr == 0 {
        // Use raw framebuffer panic if possible, else just halt
        hcf(); 
    }

    // Initialize the Dynamic Heap!
    init_heap(heap_virt_addr as usize, heap_size);

    // 3. Initialize Framebuffer & UI
    if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response().as_ref() {
        if let Some(fb) = fb_response.framebuffers().next() {
            let font = Font::new(FONT_16X32);
            let fb_addr = fb.addr() as *mut u32;

            unsafe {
                // For the 8500G/Mi TV, we draw directly to the front buffer for now 
                // to avoid double-buffering hangs until we refine the blit() logic.
                let mut cursor = Cursor::new(
                    fb_addr,
                    fb_addr, 
                    fb.width(),
                    fb.height(),
                );

                cursor.font = Some(font);
                cursor.clear(0x1a1b26); // Tokyo Night Dark Background
                UI_CURSOR = Some(cursor);
            }
        }
    }

    // 4. Success Output
    println!("Vibe OS: Kernel Initialized successfully.");
    println!("Architecture: x86_64 (Zen 4 Target)");
    println!("Memory Management: Dynamic Heap @ {:#x}", heap_virt_addr);
    
    // Test the alloc crate
    let mut v = alloc::vec::Vec::new();
    v.push("Tokyo Night");
    v.push("Vibe OS");
    println!("Active Theme: {}", v[0]);

    hcf();
}

// --- Error Handling ---

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        // Attempt to print via the UI_CURSOR
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.color_fg = 0xf7768e; // Tokyo Night Red/Pink
            println!("\n[!] KERNEL PANIC");
            println!("{}", info);
        } else {
            // FALLBACK: Raw hardware write if UI isn't ready
            if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response().as_ref() {
                if let Some(fb) = fb_response.framebuffers().next() {
                    let fb_addr = fb.addr() as *mut u32;
                    let size = (fb.width() * fb.height()) as usize;
                    for i in 0..size {
                        core::ptr::write_volatile(fb_addr.add(i), 0xf7768e);
                    }
                }
            }
        }
    }
    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}
