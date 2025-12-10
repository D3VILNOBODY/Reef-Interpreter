/*
    This is the main file for the reef interpreter. It is a standalone
    executable project that includes the reef-core as an external library.
*/

#![allow(unused)]

use clap::Parser as ClapParser;
use reef_core::lex;
use reef_core::parse;
use reef_syntax::token::{Token, TokenDisplay};
use std::path::Path;
use std::{env, fmt::Display, fs, path};

use crate::evaluator::Evaluator;

mod evaluator;

const LEXER_DEBUG_FILE: &str = "reef_lexer.log";
const PARSER_DEBUG_FILE: &str = "reef_parser.log";

fn main() {
    let args = Args::parse();
    let source_code = fs::read_to_string(&args.path).expect("Failed to read file");

    let mut scanner = lex::Scanner::new(&source_code, args.debug);
    scanner.scan();

    write_to_debug_file(
        Path::new(LEXER_DEBUG_FILE),
        format!("{}", TokenDisplay(&scanner.tokens)),
    );

    let mut parser = parse::Parser::new(scanner.tokens, args.debug);
    let parse_result = parser.parse();

    if parse_result.is_err() {
        use parse::ParserError::*;

        match parse_result.unwrap_err() {
            SyntaxError { position, message } => {
                println!("Syntax error: at {}, {}", position, message)
            }
            CurrentIndexOutOfBounds(position) => {
                println!("Attempt to index out of bounds. Index at {}", position)
            }
            UnknownToken { position } => {
                println!("Encountered an unknown token at position {}", position)
            }
        }
    }

    write_to_debug_file(
        Path::new(PARSER_DEBUG_FILE),
        format!("{:#?}", parser.program),
    );

    // let mut evaluator = Evaluator::new(parser.program);
    // evaluator.evaluate_program();
}

/// Argument struct that stores data collected from command line arguments.
/// Uses clap to parse the data I need out, such as debug lvl and path.
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

fn write_to_debug_file<T: Display>(path: &Path, data: T) -> Result<(), String> {
    let res = fs::write(path, format!("{}", data));

    match res {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{e}")),
    }
}
