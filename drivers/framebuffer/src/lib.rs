#![no_std]
#![feature(abi_x86_interrupt)]

use core::fmt;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

// --- PSF2 Font Structures ---
#[repr(C)]
pub struct Psf2Header {
    pub magic: [u8; 4],
    pub version: u32,
    pub header_size: u32,
    pub flags: u32,
    pub length: u32,
    pub char_size: u32,
    pub height: u32,
    pub width: u32,
}

const PSF2_MAGIC: [u8; 4] = [0x72, 0xb5, 0x4a, 0x86];

pub struct Font<'a> {
    pub header: &'a Psf2Header,
    pub glyphs: &'a [u8],
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let header = unsafe { &*(data.as_ptr() as *const Psf2Header) };
        if header.magic != PSF2_MAGIC {
            panic!("Invalid PSF2 font!");
        }
        let glyphs_start = header.header_size as usize;
        Self {
            header,
            glyphs: &data[glyphs_start..],
        }
    }

    pub fn get_glyph(&self, c: char) -> &[u8] {
        let index = c as usize;
        let start = index * self.header.char_size as usize;
        let end = start + self.header.char_size as usize;
        &self.glyphs[start..end]
    }
}

// --- Cursor with Backbuffer ---
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub color_bg: u32,
    pub color_fg: u32,
    pub fb_ptr: *mut u32,
    pub backbuffer_ptr: *mut u32,
    pub width: usize,
    pub height: usize,
    pub font: Option<Font<'static>>,
    pub dirty: bool, // Tell 60Hz timer to blit
}

impl Cursor {
    pub fn new(ptr: *mut u32, back_ptr: *mut u32, width: u64, height: u64) -> Self {
        Self {
            x: 0,
            y: 0,
            color_bg: 0x1a1b26, // Tokyo Night Dark
            color_fg: 0xc0caf5, // Tokyo Night Text
            fb_ptr: ptr,
            backbuffer_ptr: back_ptr,
            width: width as usize,
            height: height as usize,
            font: None,
            dirty: true,
        }
    }

    pub unsafe fn blit(&mut self) {
        core::ptr::copy_nonoverlapping(self.backbuffer_ptr, self.fb_ptr, self.width * self.height);
    }

    pub unsafe fn clear(&mut self, color: u32) {
        self.color_bg = color;
        // Fast fill
        for i in 0..(self.width * self.height) {
            *self.backbuffer_ptr.add(i) = color;
        }
        self.dirty = true;
    }

    pub unsafe fn write_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            *self.backbuffer_ptr.add(y * self.width + x) = color;
            self.dirty = true;
        }
    }

    pub fn scroll_up(&mut self) {
        let f_height = self.font.as_ref().map_or(16, |f| f.header.height as usize);
        let row_size = self.width * f_height;
        let total_size = self.width * self.height;

        unsafe {
            core::ptr::copy(
                self.backbuffer_ptr.add(row_size),
                self.backbuffer_ptr,
                total_size - row_size,
            );
            // Clear the new line
            let bottom_ptr = self.backbuffer_ptr.add(total_size - row_size);
            for i in 0..row_size {
                *bottom_ptr.add(i) = self.color_bg;
            }
        }
        self.y -= f_height;
        self.dirty = true;
    }
pub fn draw_char(&mut self, c: char) {
    // 1. Get the font metrics and header out of self immediately
    let (f_width, f_height, bpr) = if let Some(ref font) = self.font {
        (font.header.width as usize, font.header.height as usize, (font.header.width + 7) / 8)
    } else {
        return;
    };

    if c == '\n' {
        self.x = 0;
        self.y += f_height;
    } else {
        if self.x + f_width > self.width {
            self.x = 0;
            self.y += f_height;
        }

        // 2. Get the glyph data as a reference. 
        // We use a temporary scope to satisfy the borrow checker.
        let glyph = self.font.as_ref().unwrap().get_glyph(c);

        // 3. Extract the variables we need for writing so we don't have to touch 'self' fields in the loop
        let start_x = self.x;
        let start_y = self.y;
        let fg = self.color_fg;

        for py in 0..f_height {
            for px in 0..f_width {
                let byte = glyph[py * bpr as usize + px / 8];
                if (byte >> (7 - (px % 8))) & 1 == 1 {
                    // 4. Wrap the unsafe call
                    unsafe {
                        self.write_pixel(start_x + px, start_y + py, fg);
                    }
                }
            }
        }
        self.x += f_width;
    }

    if self.y + f_height > self.height {
        self.scroll_up();
    }
}
}

impl fmt::Write for Cursor {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.draw_char(c);
        }
        Ok(())
    }
}
