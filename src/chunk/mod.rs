mod footer;
mod free_list;
mod list;

pub use footer::*;
pub use free_list::*;
pub use list::*;

use std::{
    alloc::{self, Layout},
    fmt::Pointer,
    ops::Deref,
    ptr::NonNull,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Chunk(NonNull<ChunkFooter>);

impl Chunk {
    /// Attempt to create the memory layout for a chunk in memory.
    /// Returns the layout and footer offset upon success.
    fn layout(size: usize) -> Option<(Layout, usize)> {
        let data = Layout::from_size_align(size, size).ok()?;
        let footer = Layout::new::<ChunkFooter>();
        let (layout, footer_offset) = data.extend(footer).ok()?;

        Some((layout.pad_to_align(), footer_offset))
    }

    /// Allocate a new chunk
    pub(crate) unsafe fn allocate(size: usize, index: usize, next: Option<Chunk>, free_list: FreeList) -> Chunk {
        let (layout, footer_offset) = Self::layout(size).expect("invalid chunk layout");

        // Allocate
        let start = match NonNull::new(alloc::alloc(layout)) {
            Some(start) => start,
            None => alloc::handle_alloc_error(layout),
        };

        // Get the footer memory and set it
        let footer = start.as_ptr().add(footer_offset).cast::<ChunkFooter>();
        let footer = NonNull::new_unchecked(footer);
        footer
            .as_ptr()
            .write(ChunkFooter::new(start, size, index, next, free_list));

        Self(footer)
    }

    /// Calculate the pointer for a provided layout, if it can fit
    /// Reference: https://fitzgeraldnick.com/2019/11/01/always-bump-downwards.html
    fn calc_pointer(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Round the bump pointer to the needed alignment
        let ptr = self.bump.get().as_ptr() as usize;
        let mut new_ptr = ptr.checked_sub(size)?;
        new_ptr &= !(align - 1);

        let new_ptr = NonNull::new(new_ptr as *mut u8)?;

        // Too large
        if new_ptr.as_ptr() < self.start.as_ptr() {
            return None;
        }

        Some(new_ptr)
    }

    /// Check if this chunk can fit a layout within it.
    pub fn can_fit(&self, layout: Layout) -> bool {
        self.calc_pointer(layout.size(), layout.align()).is_some()
    }

    /// Allocate a layout within this chunk
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        let ptr = self
            .calc_pointer(layout.size(), layout.align())
            .expect("cannot allocate!");
        self.bump.set(ptr);

        ptr
    }

    /// Free this chunk.
    pub fn free(&self) -> Result<(), FreeError> {
        self.free_list.push(*self)
    }

    pub(crate) unsafe fn reset_bump(&self) {
        let reset_bump = unsafe { NonNull::new_unchecked(self.start.as_ptr().add(self.size)) };
        self.bump.set(reset_bump);
    }

    /// Deallocate this chunk and it's inner chunks
    pub(crate) unsafe fn drop(self) {
        let mut next_chunk = Some(self);

        while let Some(chunk) = next_chunk.take() {
            println!("Dropping {}-{}", chunk.size, chunk.index);
            println!(
                "\tcurrent: {}; free: {}; refs: {}",
                chunk.is_current(),
                chunk.is_free(),
                chunk.refs()
            );
            // Ensure there are zero references
            assert!(
                chunk.refs() == 0,
                "attempting to deallocate a chunk that still has references"
            );

            // Set the next chunk
            next_chunk = chunk.next;

            // Prepare for deallocation
            let ptr = chunk.start.as_ptr();
            let (layout, _) = Chunk::layout(chunk.size).expect("this should be impossible");

            // Deallocate
            alloc::dealloc(ptr, layout);
        }
    }
}

// This makes life an awful lot easier
impl Deref for Chunk {
    type Target = ChunkFooter;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl Pointer for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
