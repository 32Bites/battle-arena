use std::{alloc::Layout, cell::Cell};

use crate::{chunk::{FreeList, Chunk}, ptr::Ptr};

/// Handles chunks of a certain size.
#[derive(Debug)]
pub struct ChunkList {
    /// Size of each chunk
    size: usize,
    /// How many chunks there are
    len: Cell<usize>,
    /// Last freshly allocated chunk.
    head: Cell<Option<Chunk>>,
    /// Current chunk being operated on.
    current: Cell<Option<Chunk>>,
    /// Pointer to the free list head
    free_list: FreeList,
}

impl ChunkList {
    /// Create a default chunk list
    pub fn new(size: usize) -> Self {
        Self::with_capacity(size, 4)
    }

    /// Create a chunk list with `cap` chunks.
    /// If `cap` is zero, this is no different than creating
    /// an empty chunk list.
    pub fn with_capacity(size: usize, cap: usize) -> Self {
        let list = Self::empty(size);
        list.reserve(cap);

        list
    }

    /// Allocate n chunks
    pub fn reserve(&self, n: usize) {
        for _ in 0..n {
            self.allocate_chunk();
        }
    }

    /// Create a chunk list with no chunks (yet).
    pub fn empty(size: usize) -> Self {
        assert!(size.is_power_of_two(), "chunk size must be a power of two");

        Self {
            size,
            len: Cell::new(0),
            head: Cell::new(None),
            current: Cell::new(None),
            free_list: FreeList::new(),
        }
    }

    /// Allocate a new chunk
    /// and push it onto the chunk
    /// stack and free list.
    fn allocate_chunk(&self) -> Chunk {
        let index = self.len.get();
        let chunk = unsafe { Chunk::allocate(self.size, index, self.head.get(), self.free_list) };
        chunk.free().unwrap();

        self.head.set(Some(chunk));
        self.len.set(index + 1);

        chunk
    }

    /// Pops a chunk from the free list or it allocates a new one.
    fn pop_or_alloc(&self) -> Chunk {
        if self.free_list.peek().is_none() {
            self.allocate_chunk();
        }
        self.free_list.pop().expect("failed to get a chunk")
    }

    /// Gets the current chunk.
    /// If it does not exist, we get a new current chunk.
    /// If it does exist but cannot fit the layout provided,
    ///     we unmark it and get a new current chunk.
    /// If it does exist and can fit the layout, no changes are made.
    ///
    /// It is up to the caller to ensure that the provided layout
    /// actually can fit within an empty chunk.
    fn get_current(&self, layout: Layout) -> Chunk {
        if let Some(current) = self.current.get() {
            // Check that the current chunk can fit a layout.
            if current.can_fit(layout) {
                return current;
            }

            // Disable the current flag
            current.toggle_current();
        }

        // Either there was no current, or the previous current chunk could not fit the value
        let new_current = self.pop_or_alloc();
        new_current.toggle_current();
        self.current.set(Some(new_current));

        new_current
    }

    pub(crate) fn allocate(&self, layout: Layout) -> Ptr<u8> {
        let chunk = self.get_current(layout);
        let ptr = chunk.alloc_layout(layout);

        Ptr::new(chunk, ptr)
    }
}

impl Drop for ChunkList {
    fn drop(&mut self) {
        unsafe {
            self.free_list.drop();
            if let Some(chunk) = self.head.take() {
                chunk.drop();
            }
        }
    }
}
