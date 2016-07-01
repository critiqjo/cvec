#![feature(alloc)]

extern crate alloc;

use alloc::raw_vec::RawVec;

use std::cell::UnsafeCell;
use std::ptr;
use std::slice;
use std::sync::atomic::{self, AtomicUsize, Ordering};
use std::sync::Arc;

struct CVecRaw<T> {
    buf: UnsafeCell<RawVec<T>>,
    len: AtomicUsize,
}

impl<T> CVecRaw<T> {
    pub fn with_capacity(cap: usize) -> Self {
        CVecRaw {
            buf: UnsafeCell::new(RawVec::with_capacity(cap)),
            len: AtomicUsize::new(0),
        }
    }

    fn buf(&self) -> &RawVec<T> {
        unsafe { &*self.buf.get() }
    }

    unsafe fn buf_mut(&self) -> &mut RawVec<T> {
        &mut *self.buf.get()
    }

    fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}

impl<T> Drop for CVecRaw<T> {
    fn drop(&mut self) {
        let buf = self.buf();
        let len = self.len();
        atomic::fence(Ordering::Acquire);
        unsafe {
            for p in slice::from_raw_parts_mut(buf.ptr(), len) {
                ptr::drop_in_place(p as *mut T);
            }
        }
    }
}

pub struct CVec<T> {
    inner: Arc<CVecRaw<T>>,
}

impl<T> CVec<T> {
    pub fn with_capacity(cap: usize) -> Self {
        CVec { inner: Arc::new(CVecRaw::with_capacity(cap)) }
    }

    pub fn capacity(&self) -> usize {
        self.inner.buf().cap()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        let len = self.len();
        if len == self.capacity() {
            if unsafe { !self.inner.buf_mut().double_in_place() } {
                return Err(value);
            }
        }
        unsafe {
            let end = self.inner.buf().ptr().offset(len as isize);
            ptr::write(end, value);
        }
        self.inner.len.store(len + 1, Ordering::Release); // no need to cas
        Ok(())
    }

    pub fn try_reserve(&mut self, additional: usize) -> bool {
        unsafe { self.inner.buf_mut().reserve_in_place(self.len(), additional) }
    }

    pub fn view(&self) -> CVecView<T> {
        CVecView { inner: self.inner.clone() }
    }
}

pub struct CVecView<T> {
    inner: Arc<CVecRaw<T>>,
}

impl<T> CVecView<T> {
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let p = self.inner.buf().ptr();
            let len = self.len();
            atomic::fence(Ordering::Acquire);
            slice::from_raw_parts(p, len)
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T> Clone for CVecView<T> {
    fn clone(&self) -> Self {
        CVecView { inner: self.inner.clone() }
    }
}

unsafe impl<T> Send for CVecView<T> where T: Send + Sync {}
unsafe impl<T> Sync for CVecView<T> where T: Send + Sync {}

#[cfg(test)]
mod tests {
    use super::CVec;
    use std::thread;
    use std::sync::{Arc, Barrier};

    #[test]
    fn basic() {
        let mut x: CVec<usize> = CVec::with_capacity(2);
        let xv = x.view();
        x.push(300).unwrap();
        x.push(200).unwrap();
        assert_eq!(xv.as_slice(), [300, 200]);
    }

    #[test]
    fn threaded() {
        let mut x: CVec<usize> = CVec::with_capacity(2);
        let xv = x.view();

        let b1 = Arc::new(Barrier::new(2));
        let b2 = b1.clone();
        let h = thread::spawn(move || {
            b2.wait();
            while xv.len() < 1 {}
            assert_eq!(xv.as_slice()[0], 300);
            while xv.len() < 2 {}
            assert_eq!(xv.as_slice(), [300, 200]);
        });
        b1.wait();
        for _ in 0..1000 {}
        x.push(300).unwrap();
        for _ in 0..1000 {}
        x.push(200).unwrap();
        h.join().unwrap();
    }

    #[test]
    fn resize() {
        let mut x: CVec<usize> = CVec::with_capacity(2);
        let xv = x.view();
        x.push(300).unwrap();
        x.push(200).unwrap();
        if x.push(100).is_ok() {
            // never observed to reach here
            assert_eq!(xv.as_slice(), [300, 200, 100]);
        } else {
            assert_eq!(xv.as_slice(), [300, 200]);
        }
    }
}
