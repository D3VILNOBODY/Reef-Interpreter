/*
    This is the main file for the reef interpreter. It is a standalone
    executable project that includes the reef-core as an external library.
*/

#![allow(unused)]

use reef_core::{scanner::Scanner, ReefDebuggable};
use reef_core::syntax::token::Token;
use std::{env, fs, path};
use clap::Parser as ClapParser;

/*
    Argument struct that stores data collected from command line arguments.
    Uses clap to parse the data i need out, such as debug lvl and path.
 */
#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "The file with the source code inside")]
    path: path::PathBuf,

    #[arg(short='d', long = "debug", default_value_t = 0, help = "Activates debug features")]
    debug: u8,
}

fn main() {
    let args = Args::parse();
    let source_code = fs::read_to_string(&args.path).expect("Failed to read file");

    let mut scanner = Scanner::new(&source_code);
    if args.debug > 0 {
        scanner.set_debug_lvl(args.debug);
    }

    scanner.scan();
}
