

use std::{ptr::NonNull, cell::Cell};

use thiserror::Error;

use crate::chunk::Chunk;


#[derive(Debug, Clone, Copy)]
pub struct FreeList(NonNull<Cell<Option<Chunk>>>);

impl FreeList {
    pub fn new() -> Self {
        Self(Box::leak(Box::new(Cell::new(None))).into())
    }

    pub fn peek(&self) -> Option<Chunk> {
        unsafe { self.0.as_ref().get() }
    }

    /// Returns an error specifying why a chunk cannot be freed if
    /// it cannot be freed. Otherwise it returns Ok(()) if it can be
    /// freed.
    pub fn can_push(chunk: Chunk) -> Result<(), FreeError> {
        if chunk.is_free() {
            return Err(FreeError::AlreadyFree);
        }

        if chunk.is_current() {
            return Err(FreeError::IsCurrent);
        }

        let refs = chunk.refs();

        if refs != 0 {
            return Err(FreeError::RefCount(refs));
        }

        Ok(())
    }

    /// Pop a free chunk and unmark it as free, if it exists.
    pub fn pop(&self) -> Option<Chunk> {
        let top = unsafe { self.0.as_ref() };
        let popped = top.take();

        if let Some(popped) = popped {
            assert!(popped.is_free(), "corrupt free list");

            let next_free = popped.next_free.take();
            top.set(next_free);
            popped.toggle_free();
        }

        popped
    }

    /// Push a chunk and mark it as free.
    /// Additionally, reset the bump pointer.
    pub fn push(&self, chunk: Chunk) -> Result<(), FreeError> {
        Self::can_push(chunk)?;

        let top = unsafe { self.0.as_ref() };
        let next_free = top.take();

        // Set the next free value and mark it as free.
        chunk.next_free.set(next_free);
        chunk.toggle_free();

        // Push it
        top.set(Some(chunk));

        println!("Freed {}-{}", chunk.size, chunk.index);

        Ok(())
    }

    pub unsafe fn drop(self) {
        drop(Box::from_raw(self.0.as_ptr()))
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub enum FreeError {
    #[error("cannot free a chunk that is already free")]
    AlreadyFree,
    #[error("cannot free the current chunk")]
    IsCurrent,
    #[error("chunk has {0} references when it needs to be zero")]
    RefCount(u64),
}
