use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

fn main() {
    static STOP: AtomicBool = AtomicBool::new(false);

    let bg_thread = thread::spawn(|| {
        while !STOP.load(Relaxed) {
            println!("stuff");
            thread::sleep(Duration::from_secs(1));
        }
    });

    for line in std::io::stdin().lines() {
        //does not exit unless break
        match line.unwrap().as_str() {
            "stop" => break,
            cmd => println!("unknown command: {cmd:?}"),
        }
    }
    STOP.store(true, Relaxed); //communicate condition to background thread
    bg_thread.join().unwrap();
}
