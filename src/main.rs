use std::fs::File;
use std::io::{Read, Write};

use std::env;
use std::path::PathBuf;

use std::str;

fn main() {
    let mut path_buf = PathBuf::from(env::args().nth(1).unwrap());
    let mut fname = path_buf.to_str().unwrap().to_string();
    println!("{}", path_buf.to_str().unwrap().to_string());

    let mut rom = Vec::new();
    let mut file = File::open(fname).unwrap();
    file.read_to_end(&mut rom).unwrap();

    // println!("{:?}", rom);

    let mut title = Vec::new();
    for i in 0x0134..0x0143 {
        // println!("{:?}", rom[i]);
        title.push(rom[i])
        // println!("{}",  str::from_utf8(rom[i]));
    };
    println!("{}", str::from_utf8(&title).unwrap());
}
