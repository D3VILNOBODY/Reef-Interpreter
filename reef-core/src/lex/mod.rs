#![allow(unused)]

use reef_syntax::token::Token;
use std::str::Chars;
use std::{collections::HashMap, path::Path};

/// Scanner is responsible for converting text input into a stream of tokens
/// which represent the smallest components of a program. It is a struct so
/// it can keep track of its state and so that the state is shared between
/// the methods.
#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    pub tokens: Vec<Token<'a>>,
    text: &'a str,
    current: usize,
    line: i32,
    keywords: HashMap<&'a str, &'a str>,
    debug: u8,
}

impl<'a> Scanner<'a> {
    /// Construct a new Scanner, taking the text to scan as the only argument.
    pub fn new(text: &'a str, debug: u8) -> Self {
        let mut keyword_map: HashMap<&str, &str> = HashMap::new();

        // Populate the keyword list with the language keywords.
        keyword_map.insert("continue", "continue");
        keyword_map.insert("struct", "struct");
        keyword_map.insert("elseif", "elseif");
        keyword_map.insert("return", "return");
        keyword_map.insert("typeof", "typeof");
        keyword_map.insert("false", "false");
        keyword_map.insert("break", "break");
        keyword_map.insert("true", "true");
        keyword_map.insert("else", "else");
        keyword_map.insert("then", "then");
        keyword_map.insert("type", "type");
        keyword_map.insert("for", "for");
        keyword_map.insert("fun", "fun");
        keyword_map.insert("nil", "nil");
        keyword_map.insert("not", "not");
        keyword_map.insert("and", "and");
        keyword_map.insert("var", "var");
        keyword_map.insert("log", "log");
        keyword_map.insert("do", "do");
        keyword_map.insert("if", "if");
        keyword_map.insert("or", "or");

        Self {
            text,
            tokens: vec![],
            current: 0,
            line: 1,
            keywords: keyword_map,
            debug,
        }
    }

    /// Debug mode on the scanner makes it create an output file and put the tokens it generated in there
    pub fn set_debug_lvl(&mut self, debug_lvl: u8) {
        self.debug = debug_lvl;
    }

    /// Scan the input text and break it down into the smallest components.
    /// Token definitions can be found in ./lib.rs
    pub fn scan(&mut self) {
        let mut current_char: Option<&u8>;
        while self.current < self.text.len() {
            self.next_token();
        }
    }

    fn next_token(&mut self) {
        match self.get_current_char() {
            Some(c) => match c {
                '\n' => {
                    self.line += 1;
                    self.current += 1;
                }
                'a'..='z' | 'A'..='Z' | '_' => self.scan_ident(),
                '0'..='9' => self.scan_number(),
                '"' => self.scan_string(),
                '-' => self.handle_hyphen(),
                '+' | '*' | '/' => {
                    self.tokens.push(Token::BinaryOperator(c));
                    self.advance();
                }
                '<' | '>' => {
                    self.tokens.push(Token::ComparisonOperator(c));
                    self.advance();
                }
                '=' => {
                    self.tokens.push(Token::Equals);
                    self.advance();
                }
                '.' | ',' | ';' | ':' | '{' | '}' | '(' | ')' => {
                    self.tokens.push(Token::Delimiter(c));
                    self.advance();
                }
                c if c.is_whitespace() => {
                    self.advance();
                }
                _ => {
                    panic!("Panic: Unrecognised character {}", c);
                }
            },
            None => self.tokens.push(Token::EndOfFile),
        }
    }

    /// Check an identifier against the built-in hashmap of keywords, and returns true if it matches a keyword, else returns false.
    fn is_keyword(&self, ident: &str) -> bool {
        self.keywords.contains_key(ident)
    }

    /// Peek one character ahead.
    fn peek(&self) -> Option<char> {
        self.text.chars().nth(self.current + 1)
    }

    fn get_current_char(&self) -> Option<char> {
        self.text.chars().nth(self.current)
    }

    /// Increment the current char pointer and return the new value.
    fn advance(&mut self) -> usize {
        self.current += 1;
        self.current
    }

    /// Since a hyphen can be the start of multiple things, this function figures out
    /// which type of token it is supposed to be and calls the correct function to scan
    /// it fully.
    fn handle_hyphen(&mut self) {
        // consume the hyphen
        self.advance();

        match self.get_current_char() {
            Some(c) => match c {
                '-' => self.scan_comment(),
                _ => self.tokens.push(Token::BinaryOperator('-')),
            },
            None => self.tokens.push(Token::BinaryOperator('-')),
        }
    }

    /// Scan characters that make up an int/float and convert it into a 64 bit
    /// floating point number. This method can panic if there are multiple "."
    fn scan_number(&mut self) {
        let start = self.current;

        while let Some(c) = self.get_current_char() {
            match c {
                c if c.is_ascii_digit() => self.advance(),
                '_' | '.' => self.advance(),
                _ => break,
            };
        }

        let sym = &self.text[start..self.current];

        self.tokens.push(Token::Number(sym));
    }

    /// Save the contents of a comment as a string for potential use in the parser.
    fn scan_comment(&mut self) {
        // Capture both hyphens at the start
        let start = self.current - 1;

        while let Some(c) = self.get_current_char() {
            match c {
                '\n' => break,
                _ => self.advance(),
            };
        }

        // Removed for sake of simplicity in the parser. Might add this back later :3
        // let sym = &self.text[start..self.current];
        // self.tokens.push(Token::Comment(sym));
    }

    /// Scans user defined identifiers, or if the identifier matches the name
    /// of a keyword, return a keyword token instead.
    fn scan_ident(&mut self) {
        let start = self.current;

        while let Some(c) = self.get_current_char() {
            match c {
                c if c.is_ascii_alphanumeric() => self.advance(),
                '_' => self.advance(),
                _ => break,
            };
        }

        let sym = &self.text[start..self.current];

        if self.is_keyword(sym) {
            self.tokens.push(Token::Keyword(sym))
        } else {
            self.tokens.push(Token::Identifier(sym))
        }
    }

    /// Scans a string. A string starts and ends with a double quote, with the
    /// text in between them.
    fn scan_string(&mut self) {
        // Consume the first double quote
        self.advance();
        let start = self.current;

        while let Some(c) = self.get_current_char() {
            match c {
                '"' => break,
                _ => self.advance(),
            };
        }

        // Consume the ending double quote
        self.advance();
        let sym = &self.text[start..self.current - 1];

        self.tokens.push(Token::String(sym));
    }
}
