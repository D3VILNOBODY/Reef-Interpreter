#![allow(unused)]

use crate::syntax::token::{Token, TokenType};
use crate::ReefDebuggable;
use std::str::Chars;
use std::thread::current;
use std::{collections::HashMap, fs::write, path::Path};

const SCANNER_DEBUG_FILE_NAME: &str = "scanner.log";

/// Scanner is responsible for converting text input into a stream of tokens
/// which represent the smallest components of a program. It is a struct so
/// it can keep track of its state and so that the state is shared between
/// the methods.
#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    tokens: Vec<Token>,
    text: Vec<u8>,
    current: usize,
    line: i32,
    keywords: HashMap<&'a str, &'a str>,
    debug: u8,
}

impl<'a> Scanner<'a> {
    /// Construct a new Scanner, taking the text to scan as the only argument.
    pub fn new(text: &'a str) -> Self {
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
            text: String::from(text).into_bytes(),
            tokens: Vec::new(),
            current: 0,
            line: 1,
            keywords: keyword_map,
            debug: 0,
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
            current_char = self.text.get(self.current);

            let c = current_char.unwrap();
            match char::from(*c) {
                '\n' => {
                    self.line += 1;
                    self.current += 1;
                }
                '+' | '=' | '<' | '>' | '*' | '/' => {
                    let (t, v) = self.scan_operator();
                    self.add_token(t, v);
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let (t, v) = self.scan_ident();
                    self.add_token(t, v);
                }
                '0'..='9' => {
                    let (t, v) = self.scan_number();
                    self.add_token(t, v);
                }
                '-' => self.handle_hyphen(),
                '"' => {
                    let (t, v) = self.scan_string();
                    self.add_token(t, v);
                    self.advance();
                }
                ':' => {
                    self.add_token(TokenType::Colon, String::from(":"));
                    self.advance();
                }
                ';' => {
                    self.add_token(TokenType::Semicolon, String::from(";"));
                    self.advance();
                }
                '(' => {
                    self.add_token(TokenType::LeftParen, String::from("("));
                    self.advance();
                }
                ')' => {
                    self.add_token(TokenType::RightParen, String::from(")"));
                    self.advance();
                }
                '{' => {
                    self.add_token(TokenType::LeftBrace, String::from("{"));
                    self.advance();
                }
                '}' => {
                    self.add_token(TokenType::RightBrace, String::from("}"));
                    self.advance();
                }
                ',' => {
                    self.add_token(TokenType::Comma, String::from(","));
                    self.advance();
                }
                '.' => {
                    self.add_token(TokenType::Dot, String::from("."));
                    self.advance();
                }
                c if c.is_whitespace() => {
                    self.advance();
                }
                _ => {
                    panic!("Panic: Unrecognised character {}", c);
                }
            };
        }

        self.add_token(TokenType::EndOfFile, String::new());

        if self.debug > 0 {
            self.debug_write_to_file(SCANNER_DEBUG_FILE_NAME);
        }
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    /// Add a token to the list of tokens stored in the scanner state.
    fn add_token(&mut self, kind: TokenType, value: String) {
        let token = Token::new(kind, value);
        self.tokens.push(token);
    }

    /// Check an identifier against the built-in hashmap of keywords, and returns true if it matches a keyword, else returns false.
    fn is_keyword(&self, ident: &str) -> bool {
        self.keywords.contains_key(ident)
    }

    /// Peek one character ahead.
    fn peek(&self) -> Option<&u8> {
        let next_char = self.text.get(self.current + 1);
        next_char
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

        // get the next character
        let next_char = char::from(*self.text.get(self.current).unwrap());

        match next_char {
            '-' => {
                let (t, v) = self.scan_comment();
                self.add_token(t, v);
            }
            _ => {
                self.add_token(TokenType::Operator, String::from("-"))
            }
        }
    }

    /// Scan characters that make up an int/float and convert it into a 64 bit
    /// floating point number. This method can panic if there are multiple "."
    fn scan_number(&mut self) -> (TokenType, String) {
        let mut buf = String::new();
        let first_char = self.text.get(self.current);
        if first_char.is_some() {
            // Add the first character of the identifier
            buf.push(char::from(*first_char.unwrap()));
            self.advance();

            loop {
                let mut current_char = self.text.get(self.current);
                if current_char.is_none() {
                    break;
                }
                let c = char::from(*current_char.unwrap());

                if c.is_ascii_digit() || c == '.' {
                    self.advance();
                    buf.push(c);
                    continue;
                } else if c == '_' {
                    self.advance();
                    continue;
                } else {
                    break;
                }
            }
        }

        (TokenType::Number, buf)
    }

    /// Constructs an operator token, starting with one of a select few
    /// characters.
    fn scan_operator(&mut self) -> (TokenType, String) {
        let mut buf = String::new();
        let current_char = char::from(*self.text.get(self.current).unwrap());

        match current_char {
            '=' => {
                buf.push(current_char);
                self.advance();
            }
            '+' | '/' | '*' | '<' | '>' => {
                buf.push(current_char);
                self.advance();
            }
            _ => {}
        };

        (TokenType::Operator, buf)
    }

    /// Save the contents of a comment as a string for potential use in the parser.
    fn scan_comment(&mut self) -> (TokenType, String) {
        let mut buf = String::new();
        let mut current_char = self.text.get(self.current);
        if current_char.is_some() {
            buf.push(char::from(*current_char.unwrap()));

            loop {
                current_char = self.text.get(self.current);
                if current_char.is_none() || char::from(*current_char.unwrap()) == '\n' {
                    break;
                }

                buf.push(char::from(*current_char.unwrap()));
                self.advance();
            }
        }

        (TokenType::Comment, buf)
    }

    /// Scans user defined identifiers, or if the identifier matches the name
    /// of a keyword, return a keyword token instead.
    fn scan_ident(&mut self) -> (TokenType, String) {
        let mut buf = String::new();
        let mut current_char = self.text.get(self.current);
        if current_char.is_some() {
            // Add the first character of the identifier and skip past it
            buf.push(char::from(*current_char.unwrap()));
            self.advance();

            loop {
                current_char = self.text.get(self.current);
                if current_char.is_none() {
                    break;
                }

                let c = char::from(*current_char.unwrap());
                if c.is_ascii_alphanumeric() {
                    self.advance();
                    buf.push(c);
                    continue;
                } else if c == '_' {
                    self.advance();
                    continue;
                } else {
                    break;
                }
            }
        }

        if self.is_keyword(&buf) {
            (TokenType::Keyword, buf)
        } else {
            (TokenType::Identifier, buf)
        }
    }

    /// Scans a string. A string starts and ends with a double quote, with the
    /// text in between them.
    fn scan_string(&mut self) -> (TokenType, String) {
        // Consume the first double quote
        self.advance();

        let mut buf = String::new();

        loop {
            // Get the character at index ptr
            let current_char = self.text.get(self.current);
            if current_char.is_none() {
                break;
            }

            let c = char::from(*current_char.unwrap());
            if c == '"' {
                // self.advance();
                break;
            } else {
                buf.push(c);
                self.advance();
            }
        }

        (TokenType::String, buf)
    }
}

impl ReefDebuggable for Scanner<'_> {
    fn debug_write_to_file(&self, file_path: &str) {
        let mut buf = String::new();
        for token in &self.tokens {
            let token_as_str = token.to_string();
            let token_as_chars = token_as_str.chars();

            for char in token_as_chars {
                buf.push(char);
            }

            buf.push('\n');
        }

        write(Path::new(file_path), buf);
    }

    fn debug(&self) {}
}
