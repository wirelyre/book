#![allow(unreachable_code)]
#![allow(dead_code)]

extern crate pest;
#[macro_use]
extern crate pest_derive;
// extern crate regex;

mod ast;
mod runtime;

use std::env;
use std::fs;
use std::io;

use runtime::Environment;

#[derive(Debug)]
enum InputFile {
    Stdin,
    NamedFile(String),
}

fn main() {
    // Parse command-line arguments

    let mut args = env::args().skip(1);

    let program_text = match args.next() {
        Some(text) => {
            if text == "-f" {
                fs::read_to_string(args.next().expect("program file required")).unwrap()
            } else {
                text
            }
        }
        None => return,
    };

    let mut files: Vec<_> =
        args.map(|filename| {
            if filename == "-" {
                InputFile::Stdin
            } else {
                InputFile::NamedFile(filename)
            }
        }).collect();
    if files.is_empty() {
        files.push(InputFile::Stdin);
    }

    // Initialize and run program

    let program = ast::parse(&program_text);
    let mut env = Environment::default();

    env.run_begin(&program);

    for file in files {
        match file {
            InputFile::Stdin => {
                let stdin = io::stdin();
                let mut b = stdin.lock();
                env.run_file(&program, &mut b);
            }
            InputFile::NamedFile(name) => {
                let file = fs::File::open(name).expect("file must exist");
                let mut b = io::BufReader::new(file);
                env.run_file(&program, &mut b);
            }
        }
    }

    env.run_end(&program);
}
