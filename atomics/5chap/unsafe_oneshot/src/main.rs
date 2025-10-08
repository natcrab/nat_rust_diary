use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
    in_use: AtomicBool, //prevent multiple send calls from accessing the cell at the same time
}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
            in_use: AtomicBool::new(false),
        }
    }

    //after setting ready flag, receiver may read message at any point, so this can only be
    //called once; otherwise, daya race: receiver reading while sender makes a new message or
    //concurrent writing
    pub fn send(&self, message: T) {
        if self.in_use.swap(true, Relaxed) {
            panic!("can't send more than one message!");
        }
        unsafe { (*self.message.get()).write(message) };
        self.ready.store(true, Release);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Relaxed)
    }

    //unsafely assumes it has already been initialized
    pub fn receive(&self) -> T {
        if !self.ready.swap(false, Acquire) {
            panic!("no message")
        }

        unsafe { (*self.message.get()).assume_init_read() }
    }
    //If a message is sent but never received, it is never dropped
}

//Atomic is not needed since dropping is only possible if the object is fully owned by the thread
impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

unsafe impl<T> Sync for Channel<T> where T: Send {}
fn main() {
    let channel = Channel::new();
    let t = thread::current();
    thread::scope(|s| {
        s.spawn(|| {
            channel.send("stuff");
            t.unpark();
        });
        while !channel.is_ready() {
            thread::park();
        }
        assert_eq!(channel.receive(), "stuff");
    });
}
