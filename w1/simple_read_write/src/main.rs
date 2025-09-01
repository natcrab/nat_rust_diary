use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::process;
use std::str::FromStr;

struct Reader {
    number: u8,
    files: Vec<File>,
}

fn main() {
    let reader: Reader = Reader::build(env::args()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });
    let re = Regex::new(r"\d+\s+\|\s(\w|-)+\s").unwrap();
    let mut pokes: Vec<String> = Vec::new();
    for mut file in reader.files {
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
            _ => {}
        }
        let mut matches = re.captures_iter(&*contents);
        for _ in 0..reader.number {
            pokes.push(matches.next().unwrap().extract::<1>().0.to_string());
        }
    }
    println!("{:?}", pokes);
}

impl Reader {
    fn build(mut args: impl Iterator<Item = String>) -> Result<Reader, std::io::Error> {
        args.next();

        let number = match args.next() {
            Some(arg) => arg,
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Please provide a number and a file name",
                ))
            }
        };
        let number = match u8::from_str(&*number) {
            Ok(num) => num,
            Err(_) => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "The first arguement has to be a number between 0-255",
                ))
            }
        };
        let mut files: Vec<File> = Vec::new();

        while let Some(arg) = args.next() {
            files.push(File::open(arg)?);
        }
        if files.len() == 0 {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Please provide at least one file name",
            ));
        }
        Ok(Reader { number, files })
    }
}
