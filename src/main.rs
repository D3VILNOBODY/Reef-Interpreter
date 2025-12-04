/*
    This is the main file for the reef interpreter. It is a standalone
    executable project that includes the reef-core as an external library.
*/

#![allow(unused)]

use clap::Parser as ClapParser;
use reef_core::lex;
use reef_core::parse;
use reef_core::syntax::token::{Token, TokenDisplay};
use std::path::Path;
use std::{env, fmt, fs, path};

const DEBUG_FILE: &str = "REEF_LOG.log";

fn main() {
    let args = Args::parse();
    let source_code = fs::read_to_string(&args.path).expect("Failed to read file");

    let mut scanner = lex::Scanner::new(&source_code);
    scanner.scan();

    let mut parser = parse::Parser::new(scanner.tokens);
    let res = parser.parse();

    if res.is_err() {
        println!("{:?}", res.unwrap_err());
    }

    write_to_debug_file(format!("{:#?}", parser.program));
}

/*
   Argument struct that stores data collected from command line arguments.
   Uses clap to parse the data I need out, such as debug lvl and path.
*/
#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "The file with the source code inside")]
    path: path::PathBuf,

    #[arg(
        short = 'd',
        long = "debug",
        default_value_t = 0,
        help = "Activates debug features"
    )]
    debug: u8,
}

fn write_to_debug_file<T: fmt::Display>(d: T) {
    let path = Path::new(DEBUG_FILE);
    let res = fs::write(path, format!("{}", d));
    match res {
        Ok(()) => println!("[*] Successfully wrote to {DEBUG_FILE}"),
        Err(e) => println!("[!] An error has occurred: {e}"),
    }
}
