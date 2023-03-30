use std::{
    fmt::{Debug, Display},
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
};

use super::{Ptr, Ref, RefMut};

#[repr(transparent)]
pub struct Boxed<'chunk, T: ?Sized>(RefMut<'chunk, T>);

impl<'chunk, T: ?Sized> Boxed<'chunk, T> {
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: Ptr<T>) -> Self {
        Self(RefMut::from_ptr(ptr))
    }

    #[inline]
    pub(crate) unsafe fn new(ptr: Ptr<T>) -> Self {
        Self(RefMut::new(ptr))
    }

    #[inline]
    pub const fn from_mut(value: RefMut<'chunk, T>) -> Self {
        Self(value)
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> Ptr<T> {
        self.0.as_ptr()
    }

    #[inline]
    pub(crate) fn into_ptr(self) -> Ptr<T> {
        ManuallyDrop::new(self).as_ptr()
    }

    #[inline]
    pub fn as_raw(&self) -> *mut T {
        self.0.as_raw()
    }

    #[inline]
    pub fn into_mut(self) -> RefMut<'chunk, T> {
        unsafe { RefMut::from_ptr(self.into_ptr()) }
    }

    #[inline]
    pub fn into_ref(self) -> Ref<'chunk, T> {
        unsafe { Ref::from_ptr(self.into_ptr()) }
    }

    /// See [`Ref<T>::leak()`] for details.
    #[inline]
    pub fn leak(self) -> &'chunk mut T {
        self.into_mut().leak()
    }
}

impl<'chunk, T> Boxed<'chunk, MaybeUninit<T>> {
    #[inline]
    pub fn init_with(self, value: T) -> Boxed<'chunk, T> {
        Boxed::from_mut(self.into_mut().init_with(value))
    }

    #[inline]
    pub unsafe fn assume_init(self) -> Boxed<'chunk, T> {
        Boxed::from_mut(self.into_mut().assume_init())
    }
}

impl<'chunk, T: ?Sized> Deref for Boxed<'chunk, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'chunk, T: ?Sized> DerefMut for Boxed<'chunk, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<'chunk, T: ?Sized> AsRef<T> for Boxed<'chunk, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'chunk, T: ?Sized> AsMut<T> for Boxed<'chunk, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<'chunk, T: ?Sized + Debug> Debug for Boxed<'chunk, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<'chunk, T: ?Sized + Display> Display for Boxed<'chunk, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<'chunk, T: ?Sized> Drop for Boxed<'chunk, T> {
    #[inline]
    fn drop(&mut self) {
        // Call the drop function for the value
        unsafe { core::ptr::drop_in_place(self.deref_mut()) }
    }
}
