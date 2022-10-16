#![allow(unsafe_code, unreachable_pub)]

use core::{hint::unreachable_unchecked, marker::PhantomData};

#[allow(clippy::inline_always)]
#[inline(always)]
unsafe fn as_mut_unchecked<V>(opt: &mut Option<V>) -> &mut V {
    match opt {
        Some(ref mut x) => x,
        // FIXME: evaluate whether `unreachable_unchecked` is necessary here
        None => unreachable_unchecked(),
    }
}

/// Abstraction for an `Option` that's known to be `Some`
pub struct SomeBucket<'a, V> {
    opt: *mut Option<V>,
    inner: *mut V,
    _life: PhantomData<&'a mut Option<V>>,
}
impl<'a, V> SomeBucket<'a, V> {
    pub fn as_ref(&self) -> &V {
        unsafe { &(*self.inner) }
    }

    pub fn as_mut(&mut self) -> &mut V {
        unsafe { &mut (*self.inner) }
    }

    pub fn into_mut(self) -> &'a mut V {
        unsafe { &mut (*self.inner) }
    }

    pub fn replace(&mut self, value: V) -> V {
        core::mem::replace(self.as_mut(), value)
    }

    pub fn take(self) -> V {
        unsafe {
            let value = self.inner.read();
            self.opt.write(None);
            value
        }
    }
}

/// Abstraction for an `Option` that's known to be `None`
pub struct NoneBucket<'a, V> {
    opt: &'a mut Option<V>,
}
impl<'a, V> NoneBucket<'a, V> {
    pub fn insert(self, value: V) -> &'a mut V {
        unsafe {
            let opt_ptr: *mut Option<V> = self.opt;
            opt_ptr.write(Some(value));
        }

        unsafe { as_mut_unchecked(self.opt) }
    }
}

pub enum OptionBucket<'a, V> {
    Some(SomeBucket<'a, V>),
    None(NoneBucket<'a, V>),
}
impl<'a, V> OptionBucket<'a, V> {
    pub fn new(opt: &'a mut Option<V>) -> Self {
        if opt.is_some() {
            let opt_ptr: *mut Option<V> = opt;
            let inner: *mut V = unsafe { as_mut_unchecked(opt) };

            OptionBucket::Some(SomeBucket {
                opt: opt_ptr,
                inner,
                _life: PhantomData,
            })
        } else {
            OptionBucket::None(NoneBucket { opt })
        }
    }
}
