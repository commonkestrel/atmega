extern crate alloc;

use alloc::alloc::{ GlobalAlloc, Layout };
use core::{ ptr::null_mut, mem };

mod libc {
    extern "C" {
        pub fn malloc(len: usize) -> *mut ();
        pub fn free(p: *mut ());
    }
}

pub struct Alloc;

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let offset = layout.align() - 1 + mem::size_of::<*mut ()>();
        let original = libc::malloc(layout.size() + offset);
        if original.is_null() {
            return null_mut();
        }

        let aligned = (((original as usize) + offset) & !(layout.align() - 1)) as *mut u8;

        let before = aligned.sub(mem::size_of::<*mut ()>()) as *mut *mut ();
        *before = original;

        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let allocated = *((ptr as *mut *mut ()).sub(mem::size_of::<*mut ()>()));
        libc::free(allocated);
    }
}

unsafe impl Send for Alloc {}
unsafe impl Sync for Alloc {}

#[global_allocator]
pub static ALLOCATOR: Alloc = Alloc;
