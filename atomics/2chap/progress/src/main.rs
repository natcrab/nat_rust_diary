use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;
fn main() {
    let main_thread = thread::current();
    let num_done = AtomicUsize::new(0);
    thread::scope(|s| {
        s.spawn(|| {
            for i in 0..100 {
                let b = process_item(i);
                //print!("{b}");
                num_done.store(i + 1, Relaxed);
                main_thread.unpark();
            }
        });
        loop {
            let n = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("{n} % done");
            thread::park_timeout(Duration::from_secs(1));
        }
    });
    println!("Done!");
}

fn process_item(i: usize) -> usize {
    let i = i * 1000000;
    let mut sum = 0;
    for n in 0..i {
        sum += n;
    }
    sum
}
