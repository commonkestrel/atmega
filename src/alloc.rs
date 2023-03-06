extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

const MALLOC_MARGIN = 32;


struct __freelist {
    sz: usize,
    nx: *mut _freelist,
}

extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;
    static mut __brkval: u8;
}

static __flp: *mut __freelist;

pub struct Allocator;

impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should not be called")
    }
}

unsafe impl Sync for Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

#[alloc_panic_handler]
fn alloc_panic_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
