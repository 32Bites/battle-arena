use std::{alloc::Layout, cell::UnsafeCell};

use chunk::ChunkList;
use ptr::{Boxed, Ptr};

mod chunk;
pub mod ptr;

/// Minimum block size, must be a power of 2.
pub const MIN_BLOCK_SIZE: usize = 256;

/// Used for index conversions
pub const MIN_BLOCK_POW: u32 = MIN_BLOCK_SIZE.trailing_zeros();

#[inline]
const fn index_to_chunk_size(index: usize) -> usize {
    1 << (index + MIN_BLOCK_POW as usize)
}

#[inline]
const fn size_to_index(size: usize) -> usize {
    size.next_power_of_two()
        .trailing_zeros()
        .saturating_sub(MIN_BLOCK_POW) as usize
}

#[derive(Debug)]
pub struct Arena {
    /// The basic idea is every index corresponds to a power of two.
    /// This index can be used to calculate it's corresponding
    /// power of two, which is the chunk size of the chunks
    /// in the chunk list stored at the index.
    chunks: UnsafeCell<Vec<ChunkList>>,
}

impl Arena {
    /// Create a new empty arena
    pub fn new() -> Self {
        Self {
            chunks: UnsafeCell::new(Vec::new()),
        }
    }

    pub(crate) fn allocate(&self, layout: Layout) -> Ptr<u8> {
        let list = self.list_for_size(layout.size());
        let ptr = list.allocate(layout);

        ptr
    }

    /// Allocate a layout in the arena
    pub fn alloc_layout(&self, layout: Layout) -> Boxed<'_, [u8]> {
        let ptr = self.allocate(layout);
        unsafe { Boxed::new(ptr.slice(layout.size())) }
    }

    /// Allocate a value in the arena
    pub fn alloc<T>(&self, value: T) -> Boxed<'_, T> {
        let layout = Layout::new::<T>();
        let ptr = self.allocate(layout).cast::<T>();

        unsafe {
            ptr.write(value);
            Boxed::new(ptr)
        }
    }

    pub fn alloc_slice_fill_with<T>(
        &self,
        len: usize,
        mut f: impl FnMut(usize) -> T,
    ) -> Boxed<'_, [T]> {
        let layout = Layout::array::<T>(len).expect("invalid slice layout");
        let ptr = self.allocate(layout).cast::<T>();
        unsafe {
            for i in 0..len {
                ptr.add(i).write(f(i));
            }

            Boxed::new(ptr.slice(len))
        }
    }

    #[inline]
    pub fn alloc_slice_copy<T: Copy>(&self, source: &[T]) -> Boxed<'_, [T]> {
        self.alloc_slice_fill_with(source.len(), |i| source[i])
    }

    #[inline]
    pub fn alloc_slice_clone<T: Clone>(&self, source: &[T]) -> Boxed<'_, [T]> {
        self.alloc_slice_fill_with(source.len(), |i| source[i].clone())
    }

    #[inline]
    pub fn alloc_slice_fill_copy<T: Copy>(&self, len: usize, value: &T) -> Boxed<'_, [T]> {
        self.alloc_slice_fill_with(len, |_| *value)
    }

    #[inline]
    pub fn alloc_slice_fill_clone<T: Clone>(&self, len: usize, value: &T) -> Boxed<'_, [T]> {
        self.alloc_slice_fill_with(len, |_| value.clone())
    }

    #[inline]
    pub fn alloc_slice_fill_default<T: Default>(&self, len: usize) -> Boxed<'_, [T]> {
        self.alloc_slice_fill_with(len, |_| T::default())
    }

    #[inline]
    pub fn alloc_str(&self, source: &str) -> Boxed<'_, str> {
        let string = self.alloc_slice_copy(source.as_bytes());
        let (chunk, raw) = {
            let ptr = string.into_ptr();
            (ptr.chunk(), ptr.as_raw() as *mut str)
        };
        unsafe {
            let ptr = Ptr::new_unchecked(chunk, raw);
            Boxed::from_ptr(ptr)
        }
    }

    /// Returns the maximum chunk size in this arena.
    pub fn max_size(&self) -> usize {
        unsafe {
            let chunks = &*self.chunks.get();
            chunks.len().checked_sub(1).map_or(0, index_to_chunk_size)
        }
    }

    /// Reserves the next `n` chunk lists.
    pub fn reserve_next(&self, n: usize) {
        let chunks = unsafe { &mut *self.chunks.get() };

        let start = chunks.len();
        let end = start + n;

        chunks.reserve_exact(n);
        chunks.extend((start..end).map(|index| ChunkList::new(index_to_chunk_size(index))))
    }

    /// Find a chunk list for a size, or allocate one for it and the sizes leading up to it.
    pub(crate) fn list_for_size(&self, size: usize) -> &ChunkList {
        let index = size_to_index(size);
        let chunks = unsafe { &*self.chunks.get() };
        let length = chunks.len();

        // List already exists
        if index < length {
            return &chunks[index];
        }

        // Allocate the new lists.
        let new_length = index + 1;
        self.reserve_next(new_length - length);

        &chunks[index]
    }
}

#[test]
fn pow() {
    let arena = Arena::new();
    for _ in 0..2 {
        let _: Vec<_> = (0..10)
            .map(|_| arena.alloc_slice_fill_copy(128, &0xFF_u8))
            .collect();
    }
}
