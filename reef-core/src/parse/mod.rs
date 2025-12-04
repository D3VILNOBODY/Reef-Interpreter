#![allow(unused)]

use crate::syntax::{ast::*, token::Token};
use std::{iter::Peekable, mem};

/// The parser is responsible for taking a vector of tokens
/// and producing a tree-like representation of the program
/// which is fed to the evaluator.
#[derive(Clone)]
pub struct Parser<'a> {
    pub program: Vec<Stmt>,
    tokens: Vec<Token<'a>>,
    current: usize,
}

#[derive(Debug)]
pub enum ParserError {
    SyntaxError(String),
    CurrentIndexOutOfBounds(String),
    UnknownToken { position: usize },
    UnexpectedToken { position: usize },
}

impl<'a> Parser<'a> {
    /// Constructs a new parser, taking a vector of tokens
    /// produced by the scanner.
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens,
            current: 0,
            program: vec![],
        }
    }

    /// Top level function for parsing every token.
    pub fn parse(&mut self) -> Result<(), ParserError> {
        while self.current < self.tokens.len() {
            match self.get_current_token() {
                // Statements
                Some(Token::Keyword("var")) => self.variable_declr(),
                // Token::Keyword("log") => self.log(),

                // Expression statements
                Some(Token::Identifier(_)) => self.expression_statement(),
                Some(Token::String(_)) => self.expression_statement(),
                Some(Token::Number(_)) => self.expression_statement(),
                Some(Token::Delimiter('(')) => self.expression_statement(),
                Some(Token::Delimiter(';')) => Ok(self.advance()),
                _ => Err(ParserError::UnknownToken {
                    position: self.current,
                }),
            }?;
        }

        Ok(())
    }

    /// Attempts to convert n into a number and returns a wrapper around n.
    fn number_literal(&self, n: &str) -> Expr {
        let p = n.parse::<f64>();

        match p {
            Ok(v) => Expr::NumberLiteral(v),
            Err(e) => {
                panic!("[!] Error unwrapping {}: {:?}", n, e);
            }
        }
    }

    /// Creates a string literal wrapper which contains the string s.
    fn string_literal(&self, s: &str) -> Expr {
        println!("{}", self.current);
        Expr::StringLiteral(String::from(s))
    }

    /// Generates an expression statement. An expression statement is simply an expression
    /// but as a statement. For example, `10 + 5;` is an expression statement.
    fn expression_statement(&mut self) -> Result<(), ParserError> {
        let expr = self.expr()?;
        self.expect(Token::Delimiter(';'))?;

        self.add_node(Stmt::ExpressionStatement(expr));

        Ok(())
    }

    /// Generates a group expression, which is any expression inside of brackets.
    fn group_expr(&mut self) -> Result<Expr, ParserError> {
        // Starts with a '(', should also end with a ')'.

        // Skip the opening bracket
        self.advance();

        let inner = self.expr()?;

        self.expect(Token::Delimiter(')'))?;

        Ok(Expr::GroupExpression(Box::new(inner)))
    }

    /// Generates a binary expression, returning Ok if it was successful.
    fn binary_expr(&mut self) -> Result<Expr, ParserError> {
        // The left hand side of the binary expression. Creates a number from a Number token,
        // a string from a String token, and keeps track of identifiers. If the current token
        // isn't a valid type, it simply is turned into Nil.
        let lhs = match self.get_current_token() {
            Some(Token::Number(n)) => Box::new(self.number_literal(n)),
            Some(Token::String(s)) => Box::new(self.string_literal(s)),
            Some(Token::Identifier(i)) => Box::new(Expr::Identifier(String::from(i))),
            _ => Box::new(Expr::NilLiteral),
        };

        // Creates a BinaryExprOperator containing the operator used in the binary expression.
        // Panics if the token isn't a binary operator.
        let operator = match self.expect(Token::BinaryOperator(' '))? {
            Token::BinaryOperator(op) => match op {
                '+' => BinaryExprOperator::Plus,
                '-' => BinaryExprOperator::Minus,
                '*' => BinaryExprOperator::Multiply,
                '/' => BinaryExprOperator::Divide,
                '%' => BinaryExprOperator::Modulus,
                _ => {
                    return Err(ParserError::UnknownToken {
                        position: self.current,
                    })
                }
            },
            _t => {
                return Err(ParserError::UnexpectedToken {
                    position: self.current,
                })
            }
        };

        // Pass the operator.
        self.advance();

        // The right hand side of the expression. Could be any expression, so the base expression
        // method is used.
        let rhs = Box::new(self.expr()?);

        Ok(Expr::BinaryExpression {
            left_side: lhs,
            right_side: rhs,
            operator,
        })
    }

    /// The base method for parsing any kind of expression.
    fn expr(&mut self) -> Result<Expr, ParserError> {
        match self.get_current_token() {
            Some(token) => match token {
                Token::Delimiter('(') => Ok(self.group_expr()?),
                Token::Number(n) => {
                    let next = self.lookahead(1);

                    match next {
                        Some(Token::BinaryOperator(op)) => Ok(self.binary_expr()?),
                        _ => Ok(self.number_literal(n)),
                    }
                }
                Token::Identifier(ident) => {
                    // TODO: abstract this to a different function
                    let next = self.lookahead(1);

                    match next {
                        Some(Token::BinaryOperator(op)) => Ok(self.binary_expr()?),
                        _ => Ok(Expr::Identifier(String::from(ident))),
                    }
                }
                Token::String(s) => Ok(self.string_literal(s)),
                _ => panic!("[!] {:?}", token),
            },
            None => panic!("FUCK!"),
        }
    }

    /// Creates a variable declaration with a name (identifier) and a value (expression).
    fn variable_declr(&mut self) -> Result<(), ParserError> {
        let name = match self.expect(Token::Identifier(""))? {
            Token::Identifier(i) => String::from(i),
            _ => {
                return Err(ParserError::SyntaxError(
                    "Expected an identifier after keyword `var`".to_string(),
                ))
            }
        };

        self.expect(Token::BinaryOperator('='))?;

        // Skip '='
        self.advance();

        let value = self.expr()?;

        self.expect(Token::Delimiter(';'))?;

        self.add_node(Stmt::VariableDeclaration { name, value });

        Ok(())
    }

    /// Pushes `node` to `self.program`.
    fn add_node(&mut self, node: Stmt) {
        println!("[?] Adding node {:?}", node);
        self.program.push(node);
    }

    /// Gets the token at `current + distance`.
    fn lookahead(&self, distance: usize) -> Option<Token<'_>> {
        if self.current + distance >= self.tokens.len() {
            return None;
        }

        let token = self.tokens[self.current + distance];

        Some(token)
    }

    /// Returns the token at index `current`.
    fn get_current_token(&self) -> Option<Token<'_>> {
        if self.current >= self.tokens.len() {
            // return Err(ParserError::CurrentIndexOutOfBounds(format!(
            //     "Index {} is out of bounds.",
            //     self.current
            // )));

            return None;
        }

        Some(self.tokens[self.current])
    }

    /// Increments the `current` pointer and returns the next token.
    fn advance(&mut self) {
        self.current += 1;
    }

    /// Compares the next token to an expected token. Generates an error if the token doesn't
    /// match the expected one.
    fn expect(&'_ mut self, expected: Token) -> Result<Token<'_>, ParserError> {
        self.current += 1;

        // Because the error is propagated, I can't give an error message if the expected
        // symbol is supposed to be at the end of the file.
        // TODO: do something about this!
        let token = self.get_current_token();

        if token.is_none() {
            return Err(ParserError::CurrentIndexOutOfBounds(format!(
                "Index {} is out of bounds.",
                self.current
            )));
        }

        // Using mem::discriminant takes the variant of the enum at face value,
        // ignoring the value stored inside.
        if token.unwrap() == expected
            || mem::discriminant(&expected) == mem::discriminant(&Token::Identifier(""))
            || mem::discriminant(&expected) == mem::discriminant(&Token::BinaryOperator(' '))
            || mem::discriminant(&expected) == mem::discriminant(&Token::ComparisonOperator(' '))
        {
            // println!("[?] {:?} == {:?}", token, expected);
            Ok(token.unwrap())
        } else {
            // println!("[?] {:?} != {:?}", token, expected);
            Err(ParserError::SyntaxError(format!(
                "Expected {}, got {}",
                expected,
                token.unwrap()
            )))
        }
    }
}
