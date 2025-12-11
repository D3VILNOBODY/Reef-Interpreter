/*
    This is the main file for the reef interpreter. It is a standalone
    executable project that includes the reef-core as an external library.
*/

use clap::Parser as ClapParser;
use reef_core::lex;
use reef_core::parse;
use reef_syntax::token::TokenDisplay;
use std::io::Write;
use std::{fmt::Display, fs, io, path};

mod evaluator;

const LEXER_DEBUG_FILE: &str = "reef_lexer.log";
const PARSER_DEBUG_FILE: &str = "reef_parser.log";

fn main() {
    let args = Args::parse();

    match &args.path {
        Some(path) => evaluate_file(&args, path.clone()),
        None => repl(&args),
    }
}

/// Argument struct that stores data collected from command line arguments.
/// Uses clap to parse the data I need out, such as debug lvl and path.
#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'f', long = "file", default_value = None, help = "The file with the source code inside")]
    path: Option<path::PathBuf>,

    #[arg(
        short = 'd',
        long = "debug",
        default_value_t = 0,
        help = "Activates debug features"
    )]
    debug: u8,
}

fn repl(args: &Args) {
    println!("/// You are in repl mode. Type 'EXIT' to exit. \\\\\\");
    loop {
        print!("-> ");
        io::stdout().flush().expect("Stdout flush failed");

        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read from stdin");

        match buf.as_str().trim() {
            "EXIT" => panic!("Quit program"),
            _ => run(&buf, args.debug),
        }
    }
}

fn evaluate_file(args: &Args, path: path::PathBuf) {
    let source_code = fs::read_to_string(path).expect("Failed to read source code from file.");

    run(&source_code, args.debug)
}

fn run(source_code: &str, debug: u8) {
    let mut scanner: lex::Scanner;
    let mut parser: parse::Parser;
    let mut evaluator: evaluator::Evaluator;

    scanner = lex::Scanner::new(source_code, debug);
    scanner.scan();

    if debug >= 1 {
        let _ = write_to_debug_file(
            path::Path::new(LEXER_DEBUG_FILE),
            format!("{}", TokenDisplay(&scanner.tokens)),
        );
        println!("Wrote parser output to {}", LEXER_DEBUG_FILE)
    }

    parser = parse::Parser::new(scanner.tokens, debug);
    let parse_result = parser.parse_all();
    match parse_result {
        Ok(_) => {
            if debug >= 1 {
                let _ = write_to_debug_file(
                    path::Path::new(PARSER_DEBUG_FILE),
                    format!("{:?}", parser.program),
                );
                println!("Wrote parser output to {}", PARSER_DEBUG_FILE)
            }
        }
        Err(err) => match err {
            parse::ParserError::SyntaxError { position, message } => {
                println!("Syntax error: at {}, {}", position, message)
            }
            parse::ParserError::CurrentIndexOutOfBounds(position) => {
                println!("Attempt to index out of bounds. Index at {}", position)
            }
            parse::ParserError::UnknownToken { position } => {
                println!("Encountered an unknown token at position {}", position)
            }
        },
    };

    evaluator = evaluator::Evaluator::new(parser.program, debug);
    evaluator.evaluate_program();
}

fn write_to_debug_file<T: Display>(path: &path::Path, data: T) -> Result<(), String> {
    let res = fs::write(path, format!("{}", data));

    match res {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{e}")),
    }
}
