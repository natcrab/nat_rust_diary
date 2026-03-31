use std::sync::atomic::AtomicI32;
use std::sync::atomic::{AtomicBool, Ordering::Acquire, Ordering::Relaxed, Ordering::Release};
static mut DATA: String = String::new();
static LOCKED: AtomicBool = AtomicBool::new(false);
use std::thread;

fn f(num: &i32) {
    let atom = AtomicI32::new(num)
    if LOCKED
        .compare_exchange(false, true, Acquire, Relaxed)
        .is_ok()
    {
        unsafe {
            DATA.push('!');
            print!("{DATA}");
        };
        LOCKED.store(false, Release);
    } //Compare exchange only fails if locked is set to true
}
fn main() {
    let num = AtomicI32(0);
    thread::scope(|s| {
        for _ in 0..100 {
            s.spawn(f);
        }
    });
}
