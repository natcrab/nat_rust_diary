use std::boxed::Box;
use std::ptr;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering::{Acquire, Release};

#[derive(Debug)]
struct DATA {
    data1: u64,
    data2: u32,
    data3: String,
}

fn get_data() -> &'static DATA {
    static PTR: AtomicPtr<DATA> = AtomicPtr::new(std::ptr::null_mut());
    let mut p = PTR.load(Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(generate_data()));
        if let Err(e) = PTR.compare_exchange(ptr::null_mut(), p, Release, Acquire) {
            drop(unsafe { Box::from_raw(p) }); //always droppable because the box
                                               //always point to the previous p from above
            p = e;
        }
    }
    // p is always pointing to a properly initialized value
    unsafe { &*p }
}
fn main() {
    let x = get_data();
    println!("{:#?}", x);
}

fn generate_data() -> DATA {
    DATA {
        data1: 23,
        data2: 76,
        data3: "stuff".to_string(),
    }
}
