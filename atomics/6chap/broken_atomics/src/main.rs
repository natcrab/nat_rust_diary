use std::sync::atomic::{
    compiler_fence, AtomicBool, AtomicUsize,
    Ordering::{Acquire, Relaxed, Release},
};
use std::thread;

fn main() {
    let locked = AtomicBool::new(false);
    let counter = AtomicUsize::new(0);

    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                for _ in 0..1_000_000 {
                    while locked.swap(true, Relaxed) {}
                    compiler_fence(Acquire); // trick compiler into thinking the
                                             // operations is Acquire/Release without informing
                                             // processor -> atomicity
                                             // is not guaranteed on counter between locks
                    let old = counter.load(Relaxed);
                    let new = old + 1;
                    counter.store(new, Relaxed);
                    compiler_fence(Release);
                    locked.store(false, Relaxed);
                    //if works properly, this should return 4 million, should
                    //function on most CISC architecture with strong ordering
                }
            });
        }
    });
    println!("{}", counter.into_inner());
}
