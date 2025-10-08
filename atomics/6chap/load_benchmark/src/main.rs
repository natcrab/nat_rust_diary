use std::thread;
use std::{
    hint::black_box,
    sync::atomic::{AtomicU64, Ordering::Relaxed},
    time::Instant,
};
static A: AtomicU64 = AtomicU64::new(0);

fn main() {
    black_box(&A);
    thread::spawn(|| loop {
        black_box(A.load(Relaxed));
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A.load(Relaxed));
    }
    println!("{:?}", start.elapsed());
    //Due to caching, background thread can each run on a separate processor core, but the
    //caches retain a copy of A -> background thread has little effect on main thread
}
