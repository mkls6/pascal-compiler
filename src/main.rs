mod io;
mod lexer;
mod token;

use io::CharReader;
use lexer::Lexer;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: pascal-compiler source.pas");
    }

    let mut reader = BufReader::new(File::open(&args[1]).expect("Failed to open source file"));
    let mut text = String::new();

    // TODO: read file line by line internally
    let _n = reader.read_to_string(&mut text);

    let char_reader = CharReader::new(&text);
    let lexer = Lexer::new(char_reader);

    println!("Parsing tokens");

    for token in lexer {
        println!("Parsed token {}", token);
    }
}
