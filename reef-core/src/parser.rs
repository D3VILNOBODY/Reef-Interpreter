#![allow(unused)]

use std::thread::current;

use crate::ReefDebuggable;
use crate::syntax::node::Node;
use crate::syntax::token::{Token, TokenType};

/// The parser is responsible for reading a stream of tokens and outputting
/// an abstract syntax tree representation of the program.
pub struct Parser {
    ast: Node,
    current: usize,
    token_stream: Vec<Token>,
    debug: bool,
}

impl Parser {
    pub fn new(token_stream: Vec<Token>, debug: bool) -> Self {
        Self {
            token_stream,
            ast: Node::Program { children: vec![] },
            current: 0,
            debug,
        }
    }

    pub fn parse(&mut self) {
        while !self.is_at_end() {
        }
    }

    pub fn get_program_node(&self) -> &Node {
        &self.ast
    }

    fn is_at_end(&self) -> bool {
        self.peek().unwrap().kind == TokenType::EndOfFile
    }

    fn match_types(t: &Vec<Token>) {
        for token in t {

        }
    }

    fn check(&self, t: Token) -> bool {
        if self.is_at_end() {
            return false
        }

        let peeked = self.peek();
        if peeked.is_none() {
            return false
        }

        *peeked.unwrap() == t
    }

    fn peek(&self) -> Option<&Token> {
        self.token_stream.get(self.current)
    }

    fn stmt(&mut self) {
        let next_token = self.get_next_token();

        match next_token.kind {
            TokenType::Number => self.binary_expr(),
            _ => panic!("UNHANDLED TOKEN! SORRY CUH!"),
        }
    }

    fn binary_expr(&mut self) -> Node {
        let lhs = self.get_current_token();
        let operator = self.get_next_token();
        let rhs = self.expr();

        match operator.value.as_str() {
            "+" | "-" => Node::AdditiveExpression {
                lhs: Box::from(Node::NumberLiteral(lhs.value.parse::<f64>().unwrap())),
                rhs: Box::from(rhs),
                operator: operator.value.clone()
            },
            "*" | "/" | "%" => Node::MultiplicativeExpression {
                lhs: Box::from(Node::NumberLiteral(lhs.value.parse::<f64>().unwrap())),
                rhs: Box::from(rhs),
                operator: operator.value.clone()
            },
            _ => panic!("Nope"),
        }
    }

    /// Base method for parsing any type of expression. Gets the next token from the
    /// stream and matches it, calling the relevant function and appending the returned
    /// node to the return node.
    fn expr(&mut self) -> Node {
        let next = self.get_next_token();

        match *next {
            Token::Number(_) => {
                let next = self.peek_next_token();
                match *next {
                    Token::Operator(_) => self.binary_expr(),
                    _ => panic!("WHAT! YOU CANT GIVE ME {:?}", next),
                }
            }
            Token::String(_) => {}
            _ => {}
        }
    }

    // fn paren_expr(&mut self) -> ParseNode {
    //     self.get_next_token(); // Consume '('
    //
    //     let current_token = self.token_stream.get(self.current_token).unwrap();
    //     if *current_token != Token::RParen {
    //         panic!("FUCK ITS NOT A R PAREN NOOOOOOO");
    //     }
    //
    //     self.get_next_token(); // Consume ')'
    // }

    fn get_current_token(&self) -> &Token {
        let current_token = self.token_stream.get(self.current);
        current_token.unwrap()
    }

    fn get_next_token(&mut self) -> &Token {
        self.current += 1;

        let next_token = self.token_stream.get(self.current);
        next_token.unwrap()
    }

    fn peek_next_token(&self) -> &Token {
        let next_token = self.token_stream.get(self.current + 1);
        next_token.unwrap()
    }
}

impl ReefDebuggable for Parser {
    fn debug_write_to_file(&self, file_path: &str) {
        todo!()
    }

    fn debug(&self) {
        todo!();
    }
}
