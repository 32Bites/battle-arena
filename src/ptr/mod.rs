use std::{fmt::Debug, ptr::{NonNull, slice_from_raw_parts_mut}};

use crate::chunk::Chunk;

mod boxed;
mod r#ref;
mod ref_mut;

pub use boxed::*;
pub use r#ref::*;
pub use ref_mut::*;

#[derive(Debug)]
pub(crate) struct Ptr<T: ?Sized> {
    pub(crate) chunk: Chunk,
    pub(crate) ptr: NonNull<T>,
}

impl<T: ?Sized> Ptr<T> {
    #[inline]
    pub const fn new(chunk: Chunk, ptr: NonNull<T>) -> Self {
        Self { chunk, ptr }
    }

    #[inline]
    pub const unsafe fn new_unchecked(chunk: Chunk, ptr: *mut T) -> Self {
        Self::new(chunk, NonNull::new_unchecked(ptr))
    }

    #[inline]
    pub const fn cast<C>(self) -> Ptr<C> {
        Ptr::new(self.chunk, self.ptr.cast())
    }

    #[inline]
    pub const fn as_raw(self) -> *mut T {
        self.ptr.as_ptr()
    }

    #[inline]
    pub const fn chunk(self) -> Chunk {
        self.chunk
    }

    #[inline]
    pub unsafe fn deref_mut(&mut self) -> &mut T {
        self.ptr.as_mut()
    }

    #[inline]
    pub unsafe fn deref(&self) -> &T {
        self.ptr.as_ref()
    }

    #[inline]
    pub unsafe fn add_ref(self) -> u64 {
        let old = self.chunk.add_ref();
        println!("Added ref for {}-{}", self.chunk.size, self.chunk.index);
        old
    }

    #[inline]
    pub unsafe fn remove_ref(self) -> u64 {
        let old = self.chunk.remove_ref();
        println!("Removed ref for {}-{}", self.chunk.size, self.chunk.index);
        if old == 1 {
            self.chunk.reset_bump();

            if !self.chunk.is_current() {
                self.chunk.free().expect("failed to free chunk");
            }
        }
        old
    }
}

#[allow(dead_code)]
impl<T> Ptr<T> {
    #[inline]
    pub unsafe fn read(self) -> T {
        self.as_raw().read()
    }

    #[inline]
    pub unsafe fn replace(self, src: T) -> T {
        self.as_raw().replace(src)
    }

    #[inline]
    pub unsafe fn swap(self, with: *mut T) {
        self.as_raw().swap(with)
    }

    #[inline]
    pub unsafe fn write(self, val: T) {
        self.as_raw().write(val)
    }

    #[inline]
    pub unsafe fn write_bytes(self, val: u8, count: usize) {
        self.as_raw().write_bytes(val, count)
    }

    #[inline]
    pub unsafe fn add(self, count: usize) -> Self {
        Self::new_unchecked(self.chunk, self.as_raw().add(count))
    }

    #[inline]
    pub unsafe fn offset(self, offset: isize) -> Self {
        Self::new_unchecked(self.chunk, self.as_raw().offset(offset))
    }

    #[inline]
    pub unsafe fn slice(self, len: usize) -> Ptr<[T]> {
        let ptr = slice_from_raw_parts_mut(self.as_raw(), len);
        Ptr::new_unchecked(self.chunk, ptr)
    }
}

#[allow(dead_code)]
impl<T> Ptr<[T]> {
    #[inline]
    pub unsafe fn len(&self) -> usize {
        self.deref().len()
    }
}

impl<T: ?Sized> Clone for Ptr<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.chunk, self.ptr)
    }
}

impl<T: ?Sized> Copy for Ptr<T> {}
