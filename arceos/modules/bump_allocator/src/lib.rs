#![no_std]

use core::alloc::Layout;
use allocator::{AllocError, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start_vaddr: usize, size: usize) {
        self.start = start_vaddr;
        self.end = start_vaddr + size;
        self.b_pos = start_vaddr;
        self.p_pos = self.end;
        self.count = 0;
    }

    fn add_memory(&mut self, _start_vaddr: usize, _size: usize) -> Result<(), AllocError> {
        Err(AllocError::NoMemory)
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> Result<core::ptr::NonNull<u8>, AllocError> {
        let size = layout.size();
        let align = layout.align();
        let align_mask = align - 1;
        let new_pos = (self.b_pos + align_mask) & !align_mask;
        let new_end = new_pos + size;
        if new_end <= self.p_pos {
            self.b_pos = new_end;
            self.count += 1;
            Ok(unsafe { core::ptr::NonNull::new_unchecked(new_pos as *mut u8) })
        } else {
            Err(AllocError::NoMemory)
        }
    }

    fn dealloc(&mut self, _pos: core::ptr::NonNull<u8>, _layout: Layout) {
        // Do nothing for now
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        (self.b_pos - self.start) + (self.end - self.p_pos)
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> Result<usize, AllocError> {
        let size = num_pages * PAGE_SIZE;
        let align = align_pow2;
        let align_mask = align - 1;
        let new_end = (self.p_pos - align_mask) & !align_mask;
        let new_pos = new_end - size;
        if new_pos >= self.b_pos {
            self.p_pos = new_pos;
            Ok(new_pos)
        } else {
            Err(AllocError::NoMemory)
        }
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Do nothing for now
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        ((self.end - self.p_pos) + PAGE_SIZE - 1) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / PAGE_SIZE
    }
}