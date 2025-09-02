use std::collections::BTreeMap;
use std::io;

// may improve the typing loop with winit later, but because terminal is not an "app", this is
// currently too complicated for my liking
fn main() -> io::Result<()> {
    let table: BTreeMap<u8, u8> = setup();
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut key = "example".to_string();
    let mut word = String::new();
    loop {
        buffer.clear();
        println!("The current key is {key}");
        println!("w for a new word\n k for a new key\n r to redo previous translation (with current key)\n q to quit");
        stdin.read_line(&mut buffer)?;
        match &*buffer.trim() {
            "w" => {
                word.clear();
                println!("Type your word");
                stdin.read_line(&mut word)?;
                println!(
                    "\nThe ciphertext is: \n{result}",
                    result = translate(&*key, &*word, &table)
                );
            }
            "k" => {
                key.clear();
                println!("Type your new cipherkey");
                loop {
                    stdin.read_line(&mut key)?;
                    let key_chars: Vec<_> = key.trim().split_whitespace().collect();
                    key = key_chars.join("");
                    if key != "" && key.chars().all(char::is_alphabetic) {
                        break;
                    }
                    println!("Keys should only contains letters");
                    key.clear();
                }
            }
            "q" => return Ok(()),
            "r" => println!(
                "\nThe cipertext is: \n{result}",
                result = translate(&*key, &*word, &table)
            ),
            _ => {}
        }
    }
}

fn translate(key: &str, word: &str, table: &BTreeMap<u8, u8>) -> String {
    let mut result = String::new();
    let keylen = key.len();
    let mut i = 0;
    for byte in word.bytes() {
        if byte < 65 || byte > 122 || (byte > 90 && byte < 97) {
            result.push(char::from(byte));
            continue;
        }
        if i == keylen {
            i = 0;
        }
        let key_char = &key.as_bytes()[i];
        let mut result_char = 65;
        if byte > 96 {
            result_char = 97;
        }
        result_char += (table.get(&byte).unwrap() + table.get(&key_char).unwrap()) % 26;
        result.push(char::from(result_char));
        i += 1;
    }
    result
}
fn setup() -> BTreeMap<u8, u8> {
    let mut table: BTreeMap<u8, u8> = BTreeMap::new();
    // for (a, b, c) in (65..90, 97..122, 0..25) {
    for n in 0..=25 {
        table.insert(65 + n, n);
        table.insert(97 + n, n);
    }
    table
}
