use super::{Ptr, RefMut, Boxed};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::Deref,
};

#[repr(transparent)]
pub struct Ref<'chunk, T: ?Sized> {
    ptr: Ptr<T>,
    _marker: PhantomData<&'chunk T>,
}

impl<'chunk, T: ?Sized> Ref<'chunk, T> {
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: Ptr<T>) -> Self {
        Self {
            ptr,
            _marker: PhantomData
        }
    }

    /// Create a reference from a pointer,
    /// incrementing the chunk reference count
    pub(crate) unsafe fn new(ptr: Ptr<T>) -> Self {
        ptr.add_ref();

        Self::from_ptr(ptr)
    }

    #[inline]
    pub fn from_mut(value: RefMut<'chunk, T>) -> Self {
        value.into_ref()
    }

    #[inline]
    pub fn from_box(value: Boxed<'chunk, T>) -> Self {
        value.into_ref()
    }

    /// Get the inner [`Ptr<T>`] for this reference
    #[inline]
    pub(crate) fn as_ptr(&self) -> Ptr<T> {
        self.ptr
    }

    /// Turn this reference into a [`Ptr<T>`]
    /// without decrementing the reference count.
    ///
    /// In effect, this will leak the stored value
    #[inline]
    pub(crate) fn into_ptr(self) -> Ptr<T> {
        ManuallyDrop::new(self).as_ptr()
    }

    /// Get the raw pointer for this reference.
    #[inline]
    pub fn as_raw(&self) -> *const T {
        self.ptr.as_raw()
    }

    /// Leak this value.
    ///
    /// The chunk cannot be freed once a value is freed within it.
    #[inline]
    pub fn leak(self) -> &'chunk T {
        unsafe { &*self.into_ptr().as_raw() }
    }
}

impl<'chunk, T: ?Sized> Deref for Ref<'chunk, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.deref() }
    }
}

impl<'chunk, T: ?Sized> AsRef<T> for Ref<'chunk, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'chunk, T: ?Sized + Debug> Debug for Ref<'chunk, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<'chunk, T: ?Sized + Display> Display for Ref<'chunk, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<'chunk, T: ?Sized> Clone for Ref<'chunk, T> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::new(self.ptr) }
    }
}

impl<'chunk, T: ?Sized> Drop for Ref<'chunk, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.ptr.remove_ref();
        }
    }
}
