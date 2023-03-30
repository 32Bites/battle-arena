use std::{
    cell::{Cell, UnsafeCell},
    ptr::NonNull,
};

use crate::chunk::{Chunk, FreeList};

#[repr(C)]
#[derive(Debug)]
pub struct ChunkFooter {
    /// Size of the data stored in the chunk (ignoring any padding, and the footer).
    /// This can be used to reset the bump pointer, or to calculate the layout
    /// of the heap allocation containing the chunk.
    pub(crate) size: usize,

    /// Index of this chunk in it's chunk list
    pub(crate) index: usize,

    /// Pointer to the start of the allocation.
    pub(crate) start: NonNull<u8>,

    /// Bump allocation pointer.
    pub(crate) bump: Cell<NonNull<u8>>,

    /// Next chunk
    pub(crate) next: Option<Chunk>,

    /// Next free chunk
    pub(crate) next_free: Cell<Option<Chunk>>,

    /// Pointer to the free list head
    pub(crate) free_list: FreeList,

    /// Chunk flags, contains information about
    /// whether this chunk is free, the current chunk,
    /// the reference count, the next free chunk, and
    /// the bump pointer position.
    pub(crate) flags: UnsafeCell<u64>,
}

const CURRENT_BIT: u64 = !(u64::MAX >> 1);
const FREE_BIT: u64 = CURRENT_BIT >> 1;
const REF_COUNT: u64 = !(CURRENT_BIT | FREE_BIT);

impl ChunkFooter {
    pub const fn new(
        start: NonNull<u8>,
        size: usize,
        index: usize,
        next: Option<Chunk>,
        free_list: FreeList,
    ) -> Self {
        let bump = unsafe { NonNull::new_unchecked(start.as_ptr().add(size)) };

        Self {
            size,
            start,
            index,
            next,
            free_list,
            flags: UnsafeCell::new(0),
            bump: Cell::new(bump),
            next_free: Cell::new(None),
        }
    }

    #[inline]
    fn flags_ptr(&self) -> *mut u64 {
        self.flags.get()
    }

    #[inline]
    pub fn flags(&self) -> u64 {
        unsafe { *self.flags_ptr() }
    }

    /// Get the reference count
    #[inline]
    pub fn refs(&self) -> u64 {
        self.flags() & REF_COUNT
    }

    /// Increment the reference count, returning the previous count.
    /// Panics on reference count overflow.
    #[inline]
    pub fn add_ref(&self) -> u64 {
        let previous = self.refs();
        assert!(previous != REF_COUNT, "reference counter will overflow");
        unsafe {
            // Since the lower bits are where we store the counter
            // and since we checked for an overflow, this should work.
            *self.flags_ptr() += 1;
        }

        previous
    }

    /// Decrement the reference count, returning the previous count.
    /// Panics on reference count underflow.
    #[inline]
    pub fn remove_ref(&self) -> u64 {
        let previous = self.refs();
        assert!(previous != 0, "reference counter will underflow");
        unsafe {
            // Since the lower bits are where we store the counter
            // and since we checked for an underflow, this should work.
            *self.flags_ptr() -= 1;
        }

        previous
    }

    /// Checks if the free bit is set.
    #[inline]
    pub fn is_free(&self) -> bool {
        self.flags() & FREE_BIT != 0
    }

    /// Checks if the current bit is set.
    #[inline]
    pub fn is_current(&self) -> bool {
        self.flags() & CURRENT_BIT != 0
    }

    /// Toggle the free bit.
    #[inline]
    pub fn toggle_free(&self) {
        unsafe {
            *self.flags_ptr() ^= FREE_BIT;
        }
    }

    /// Toggle the current bit.
    #[inline]
    pub fn toggle_current(&self) {
        unsafe {
            *self.flags_ptr() ^= CURRENT_BIT;
        }
    }
}
