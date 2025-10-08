//#[repr(align(64))]
//struct Aligned(AtomicU64);
//Using an aligned struct as such will cause padding until it aligns with 64 bytes  => exchange
//some storage cost to prevent sharing cacheline between elements, making it faster
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::thread;
use std::time::Instant;
static A: [AtomicU64; 3] = [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];

fn main() {
    black_box(&A);
    thread::spawn(|| {
        loop {
            A[0].store(0, Relaxed);
            A[2].store(0, Relaxed);
        } //At least one shares a cache line with A[1] -> is slowed down regardless, even
          //if unrelated
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A[1].load(Relaxed)); //Should be slower + much higher variance from thread being marked as exclusive
                                       //and removing the shared caches between background process
    }
    println!("{:?}", start.elapsed());
}
