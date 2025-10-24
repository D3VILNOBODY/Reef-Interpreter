use std::fmt::{Display, Formatter};

/// Different types of tokens which can be returned by the scanner.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Operator,
    Keyword,
    Identifier,
    String,
    Number,
    Comment,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Colon,
    Comma,
    Dot,
    EndOfFile,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub value: String,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Token {
    pub fn new(kind: TokenType, value: String) -> Self {
        Self { kind, value }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(type: {}, value: '{}')", self.kind, self.value)
    }
}
