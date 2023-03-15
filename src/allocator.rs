extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::{null_mut, NonNull};

const MALLOC_MARGIN: usize = 32;
const HEAP_SIZE: usize = 512;

struct Heap {
    pub used: usize,
    pub free: FreeList,
}

struct FreeList {
    pub first: Chunk,
    pub bottom: *mut u8,
    pub top: *mut u8,
}

struct Chunk {
    pub size: usize,
    pub next: Option<NonNull<u8>>,
}

impl FreeList {
    pub const fn empty() -> FreeList {
        FreeList {
            first: Chunk {size: 0, next: None},
            bottom: null_mut(),
            top: null_mut(),
        }
    }

    pub fn new(bottom: *mut u8, top: *mut u8) -> FreeList {
        FreeList { bottom, top, first: Chunk {size: 0, next: None} }
    }
}

unsafe impl GlobalAlloc for FreeList {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should not be called")
    }
}

unsafe impl Send for FreeList {}
unsafe impl Sync for FreeList {}

#[global_allocator]
static ALLOCATOR: FreeList = FreeList::empty();
