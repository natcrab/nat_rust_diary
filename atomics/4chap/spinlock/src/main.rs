use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) {
        while self.locked.swap(true, Acquire)
        //while current value is true
        {
            std::hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Release);
    }
}

fn main() {
    println!("Hello, world!");
}

//In most cases, it will be used to protect mutations to a shared variable
//Meaning the API still requires unsafe and unchecked code -> not very good
