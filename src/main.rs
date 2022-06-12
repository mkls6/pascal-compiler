mod analyzer;
mod error;
mod io;
mod lexer;
mod parser;
mod scope;
mod syntax;
mod token;

use io::CharReader;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: pascal-compiler source.pas");
        exit(1);
    }

    let filename = &args[1];

    let char_reader = CharReader::new(String::from(filename));
    match char_reader {
        Ok(reader) => {
            let lexer = Lexer::new(reader);
            let mut parser = Parser::new(lexer);

            let res = parser.parse();
            match res {
                Ok(r) => {
                    println!("Parsed program!");
                    println!("Errors:");

                    for e in parser.errors {
                        println!("{}", e);
                    }

                    println!("{:#?}", r)
                }
                Err(e) => println!("{}", e),
            }
        }
        Err(e) => {
            eprintln!("Failed to open file {}: {}", filename, e);
        }
    }
}
