use libc;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::Instant;
use std::{cell::UnsafeCell, ops::Deref, ops::DerefMut, sync::atomic::AtomicU32};

//lock_api crate
pub struct Mutex<T> {
    state: AtomicU32, //0: unlocked, 1: locked, 2: other threads waiting
    value: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

unsafe impl<T> Send for MutexGuard<'_, T> where T: Send {}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Sync {}
unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T; //associated type Target for deref
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Mutex<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        /*while self.state.swap(1, Acquire) == 1 {
            wait(&self.state, 1); //wait until unlock
        }
        MutexGuard { mutex: self }*/
        //Avoid syscalls wait and wake whenever possible
        if self.state.compare_exchange(0, 1, Acquire, Relaxed).is_err() {
            /*while self.state.swap(2, Acquire) != 0 {
                wait(&self.state, 2);
            }*/
            lock_contended(&self.state);
        }
        MutexGuard { mutex: self }
    }
}

#[cold]
fn lock_contended(state: &AtomicU32) {
    //avoids compare and exchange as it may cause exclusive access to cache lines
    let mut spin_count = 0;
    while state.load(Relaxed) == 1 && spin_count < 100 {
        //Rust 1.66 uses a spin count of 100
        spin_count += 1;
        std::hint::spin_loop();
    }

    if state.compare_exchange(0, 1, Acquire, Relaxed).is_ok() {
        return;
    }
    while state.swap(2, Acquire) != 0 {
        wait(state, 2);
    }
}

pub fn wait(a: &AtomicU32, expected: u32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex,                    //futex syscall
            a as *const AtomicU32,              //futex word (atomic to operate on)
            libc::FUTEX_WAIT,                   //operation
            expected,                           //expected value
            std::ptr::null::<libc::timespec>(), //no timeout
        );
    }
}

pub fn wake_one(a: &AtomicU32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex,
            a as *const AtomicU32,
            libc::FUTEX_WAKE,
            1, // num of
               // threads to wake up
        );
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        if self.mutex.state.swap(0, Release) == 2 {
            wake_one(&self.mutex.state); // thread woken up responsible for setting
            // state back to 2
        } //notify any mutex waiting
    }
}

//wait may spuriously wake up => cannot make assumption about when it returns
fn main() {
    /*let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    for _ in 0..5_000_000 {
        *m.lock() += 1;
    }
    let duration = start.elapsed();
    println!("locked {} times in {:?}", *m.lock(), duration);
    //about 90ms
    */
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                for _ in 0..5_000_000 {
                    *m.lock() += 1;
                }
            });
        }
    });
    let duration = start.elapsed();
    println!("locked {} times in {:?}", *m.lock(), duration);
}
