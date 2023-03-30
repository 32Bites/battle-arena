use super::{Boxed, Ptr, Ref};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
};

#[repr(transparent)]
pub struct RefMut<'chunk, T: ?Sized> {
    ptr: Ptr<T>,
    _marker: PhantomData<&'chunk mut T>,
}

impl<'chunk, T: ?Sized> RefMut<'chunk, T> {
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: Ptr<T>) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    pub(crate) unsafe fn new(ptr: Ptr<T>) -> Self {
        ptr.add_ref();

        Self::from_ptr(ptr)
    }

    #[inline]
    pub fn into_ref(self) -> Ref<'chunk, T> {
        unsafe { Ref::from_ptr(self.into_ptr()) }
    }

    #[inline]
    pub fn into_box(self) -> Boxed<'chunk, T> {
        Boxed::from_mut(self)
    }

    #[inline]
    pub fn from_box(value: Boxed<'chunk, T>) -> Self {
        value.into_mut()
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> Ptr<T> {
        self.ptr
    }

    #[inline]
    pub fn as_raw(&self) -> *mut T {
        self.ptr.as_raw()
    }

    /// See [`Ref<T>::into_ptr()`] for details.
    #[inline]
    pub(crate) fn into_ptr(self) -> Ptr<T> {
        ManuallyDrop::new(self).as_ptr()
    }

    /// See [`Ref<T>::leak()`] for details.
    #[inline]
    pub fn leak(self) -> &'chunk mut T {
        unsafe { &mut *ManuallyDrop::new(self).as_raw() }
    }
}

impl<'chunk, T> RefMut<'chunk, MaybeUninit<T>> {
    /// Overwrites the current value.
    ///
    /// Note that if the current value is indeed not uninitialized, it will not be dropped.
    pub fn init_with(mut self, value: T) -> RefMut<'chunk, T> {
        self.deref_mut().write(value);

        // SAFETY: Since we know we just initialized the value
        //         it is safe to cast the pointer.
        unsafe { self.assume_init() }
    }

    /// Unsafely assume that the value is initialized.
    #[inline]
    pub unsafe fn assume_init(self) -> RefMut<'chunk, T> {
        RefMut::from_ptr(self.into_ptr().cast())
    }
}

impl<'chunk, T: ?Sized> Deref for RefMut<'chunk, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.deref() }
    }
}

impl<'chunk, T: ?Sized> DerefMut for RefMut<'chunk, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.deref_mut() }
    }
}

impl<'chunk, T: ?Sized> AsRef<T> for RefMut<'chunk, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'chunk, T: ?Sized> AsMut<T> for RefMut<'chunk, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<'chunk, T: ?Sized> Drop for RefMut<'chunk, T> {
    fn drop(&mut self) {
        unsafe {
            self.ptr.remove_ref();
        }
    }
}

impl<'chunk, T: ?Sized + Debug> Debug for RefMut<'chunk, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<'chunk, T: ?Sized + Display> Display for RefMut<'chunk, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}
