use std::cell::UnsafeCell; //does not implement Sync by default
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> &mut T {
        //both contains a reference, assume lifetime is identical
        while self.locked.swap(true, Acquire) {
            std::hint::spin_loop();
        }
        unsafe { &mut *self.value.get() }
    }
    //Ideally, the lifetime of the mut reference should end when the next call to unlock happen
    //or if self is dropped, but there is not a safe way to do it with this approach

    pub unsafe fn unlock(&self) {
        self.locked.store(false, Release)
    }
    //Only used once &mut T from lock is dropped
}
fn main() {
    println!("Hello, world!");
}
