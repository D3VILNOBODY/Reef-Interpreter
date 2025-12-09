#![allow(unused)]

use std::fmt;

/// Different types of tokens which can be returned by the scanner.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token<'a> {
    Comment(&'a str),
    String(&'a str),
    Keyword(&'a str),
    Number(&'a str),
    Identifier(&'a str),
    Delimiter(char),          // (, ), [, ], {, }, ;, :
    BinaryOperator(char),     // +, -, /, *
    ComparisonOperator(char), // <, >
    Illegal(char),
    Equals,
    EndOfFile,
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Wrapper type for Vec<Token> which allows it to be displayed.
/// Used to print out token vectors and write them to files.
pub struct TokenDisplay<'a>(pub &'a [Token<'a>]);

impl<'a> fmt::Display for TokenDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[")?;
        for item in self.0 {
            writeln!(f, "\t{},", item)?;
        }
        writeln!(f, "]")?;

        Ok(())
    }
}
