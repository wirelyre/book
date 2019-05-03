#![allow(unreachable_code)]
#![allow(dead_code)]

#[macro_use]
extern crate clap;
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

fn main() -> Result<(), pest::error::Error<ast::Rule>> {
    // Parse command-line arguments

    let args = clap_app!(awk =>
        (version: "0.1")
        (about: "a small language for text processing")
        (@arg progfile: -f +takes_value +required "the AWK script to run")
        (@arg INPUT: +multiple "an input file, or - for stdin")
    )
    .get_matches();

    let program_file = args.value_of("progfile").unwrap();
    let program_text = fs::read_to_string(program_file).expect("could not read program file");

    let files = match args.values_of("INPUT") {
        Some(files) => files
            .map(|f| {
                if f == "-" {
                    InputFile::Stdin
                } else {
                    InputFile::NamedFile(f.to_string())
                }
            })
            .collect(),
        None => vec![InputFile::Stdin],
    };

    // Initialize and run program

    let program = ast::parse(&program_text)?;
    let mut env = Environment::default();

    env.run_begin(&program);

    for file in files {
        match file {
            InputFile::Stdin => {
                let stdin = io::stdin();
                let mut b = stdin.lock();
                // TODO: handle IO errors
                env.run_file(&program, &mut b).unwrap();
            }
            InputFile::NamedFile(name) => {
                let file = fs::File::open(name).expect("file must exist");
                let mut b = io::BufReader::new(file);
                // TODO: handle IO errors
                env.run_file(&program, &mut b).unwrap();
            }
        }
    }

    env.run_end(&program);

    Ok(())
}
