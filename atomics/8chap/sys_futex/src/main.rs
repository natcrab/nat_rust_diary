use libc;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::thread;
use std::time::Duration;
#[cfg(not(target_os = "linux"))]
compile_error!("Linux only"); //Syscall is only consistent on Linux kernel interface

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

fn main() {
    let a = AtomicU32::new(0);

    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_secs(3));
            a.store(1, Relaxed);
            wake_one(&a); //wake up main thread in case its sleeping to react to the
                          //change variable
        });
        while a.load(Relaxed) == 0 {
            wait(&a, 0); //check whether a is still - before going to sleep, signal
                         //from spawn thread cannot be lost between condition check and sleeping
        }
        println!("Finished!");
    });
}

fn futex_wake_op() {
    let atomic1 = AtomicU32::new(1);
    let atomic2 = AtomicU32::new(2);
    let old = atomic2.fetch_update(Relaxed, Relaxed, stuff);
    wake(atomic1, 1);
    if old == Ok(2) {
        wake(atomic2, 3);
    }
}

fn stuff(n: u32) -> Option<u32> {
    Some(n + 1)
}
fn wake(a: AtomicU32, n: u32) {} //usually from some futex library
