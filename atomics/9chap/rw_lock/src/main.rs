use libc;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::Instant;
use std::{cell::UnsafeCell, ops::Deref, ops::DerefMut, sync::atomic::AtomicU32};
pub struct RwLock<T> {
    //The number of read locks x2 + 1 if there is a writer waiting
    //Reader may acquire lock when state is even, but block when odd
    //This is to prevent writer starvation
    state: AtomicU32, //number of readers, u32::MAX indicates write-locked
    value: UnsafeCell<T>,
    writer_wake_counter: AtomicU32, //fixes problem of low state changing between compare and
                                    //exchange and subsequent waits
}

unsafe impl<T> Sync for RwLock<T> where T: Send + Sync {}

impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
            writer_wake_counter: AtomicU32::new(0),
        }
    }

    pub fn read(&self) -> ReadGuard<'_, T> {
        let mut s = self.state.load(Relaxed);
        loop {
            if s % 2 == 0 {
                assert!(s < u32::MAX - 2, "too many readers");
                match self.state.compare_exchange_weak(s, s + 2, Acquire, Relaxed) {
                    Ok(_) => return ReadGuard { rwlock: self },
                    Err(e) => s = e,
                }
            }
            if s % 2 == 1 {
                wait(&self.state, s);
                s = self.state.load(Relaxed);
            }
        }
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        /*while let Err(s) = self.state.compare_exchange(0, u32::MAX, Acquire, Relaxed) {
            wait(&self.state, s);
        }
        WriteGuard { rwlock: self }*/
        /*while self
            .state
            .compare_exchange(0, u32::MAX, Acquire, Relaxed)
            .is_err()
        {
            let w = self.writer_wake_counter.load(Acquire);
            if self.state.load(Relaxed) != 0 {
                //wait if rwlock is still locked but no wake signals since checked
                wait(&self.writer_wake_counter, w);
            }
        }
        WriteGuard { rwlock: self }*/
        let mut s = self.state.load(Relaxed);
        loop {
            //try locking if unlock
            if s <= 1 {
                match self.state.compare_exchange(s, u32::MAX, Acquire, Relaxed) {
                    Ok(_) => return WriteGuard { rwlock: self },
                    Err(e) => {
                        s = e;
                        continue;
                    }
                }
            }
            //block new readers by making the state odd
            if s % 2 == 0 {
                match self.state.compare_exchange(s, s + 1, Relaxed, Relaxed) {
                    Ok(_) => {}
                    Err(e) => {
                        s = e;
                        continue;
                    }
                }
            }
            let w = self.writer_wake_counter.load(Acquire);
            s = self.state.load(Relaxed);
            if s >= 2 {
                wait(&self.writer_wake_counter, w);
                s = self.state.load(Relaxed);
            }
        }
    }
}

pub struct ReadGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

pub struct WriteGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.value.get() }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.rwlock.value.get() }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        if self.rwlock.state.fetch_sub(2, Release) == 3 {
            self.rwlock.writer_wake_counter.fetch_add(1, Release);
            wake_one(&self.rwlock.writer_wake_counter); //no reader would wait on a readlocked lock
        }
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.rwlock.state.store(0, Release);
        self.rwlock.writer_wake_counter.fetch_add(1, Release);
        wake_one(&self.rwlock.writer_wake_counter);
        wake_all(&self.rwlock.state); //wake all waiting readers and writers
        //could be skipped if wake_one indicate if it woke up a thread
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

pub fn wake_all(a: &AtomicU32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex,
            a as *const AtomicU32,
            libc::FUTEX_WAKE,
            libc::INT_MAX, // num of
                           // threads to wake up
        );
    }
}

fn main() {
    println!("Hello, world!");
}
