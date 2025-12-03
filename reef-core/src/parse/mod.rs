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
    pub fn parse(&mut self) -> Result<(), String> {
        match self.get_current_token()? {
            // Statements
            Token::Keyword("var") => self.variable_declr(),
            // Token::Keyword("log") => self.log(),

            // Expression statements
            Token::Identifier(_) => self.expression_statement(),
            Token::String(_) => self.expression_statement(),
            Token::Number(_) => self.expression_statement(),
            _ => Err(format!("Unhandled! {:?}", self.get_current_token())),
        }?;

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
        Expr::StringLiteral(String::from(s))
    }

    /// Generates an expression statement. An expression statement is simply an expression
    /// but as a statement. For example, `10 + 5;` is an expression statement.
    fn expression_statement(&mut self) -> Result<(), String> {
        let expr = self.expr()?;
        self.expect(Token::Delimiter(';'))?;

        self.add_node(Stmt::ExpressionStatement(expr));

        Ok(())
    }

    /// Generates a group expression, which is any expression inside of brackets.
    fn group_expr(&mut self) -> Expr {
        todo!();
    }

    /// Generates a binary expression, returning Ok if it was successful.
    fn binary_expr(&mut self) -> Result<Expr, String> {
        // The left hand side of the binary expression. Creates a number from a Number token,
        // a string from a String token, and keeps track of identifiers. If the current token
        // isn't a valid type, it simply is turned into Nil.
        let lhs = match self.get_current_token()? {
            Token::Number(n) => Box::new(self.number_literal(n)),
            Token::String(s) => Box::new(self.string_literal(s)),
            Token::Identifier(i) => Box::new(Expr::Identifier(String::from(i))),
            _ => Box::new(Expr::NilLiteral),
        };

        // Creates a BinaryExprOperator containing the operator used in the binary expression.
        // Panics if the token isn't a binary operator.
        let operator = match self.expect(Token::BinaryOperator(' '))? {
            Token::BinaryOperator(op) => BinaryExprOperator(op),
            _t => return Err(format!("[!] Expected a binary operator, got {:?}", _t)),
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
    fn expr(&mut self) -> Result<Expr, String> {
        match self.get_current_token() {
            Ok(token) => match token {
                Token::Delimiter('(') => Ok(self.group_expr()),
                Token::Number(n) => {
                    let next = self.lookahead(1)?;

                    match next {
                        Token::BinaryOperator(op) => Ok(self.binary_expr()?),
                        _ => Ok(self.number_literal(n)),
                    }
                }
                Token::Identifier(ident) => {
                    // TODO: abstract this to a different function
                    let next = self.lookahead(1)?;

                    match next {
                        Token::BinaryOperator(op) => Ok(self.binary_expr()?),
                        _ => Ok(Expr::Identifier(String::from(ident))),
                    }
                }
                Token::String(s) => Ok(self.string_literal(s)),
                _ => panic!("[!] {:?}", token),
            },
            Err(e) => panic!("{}", e),
        }
    }

    /// Creates a variable declaration with a name (identifier) and a value (expression).
    fn variable_declr(&mut self) -> Result<(), String> {
        let name = match self.expect(Token::Identifier(""))? {
            Token::Identifier(i) => String::from(i),
            _ => return Err("[!] Expected an identifier after keyword `var`".to_string()),
        };

        self.expect(Token::BinaryOperator('='))?;

        // Skip '='
        self.advance()?;

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
    fn lookahead(&self, distance: usize) -> Result<Token<'_>, String> {
        if self.current + distance >= self.tokens.len() {
            return Err(format!(
                "Index is out of bounds. Tokens len: {}, index: {}",
                self.tokens.len(),
                self.current + distance
            ));
        }

        let token = self.tokens[self.current + distance];

        Ok(token)
    }

    /// Returns the token at index `current`.
    fn get_current_token(&self) -> Result<Token<'_>, String> {
        if self.current >= self.tokens.len() {
            return Err(format!(
                "Index is out of bounds. Tokens len: {}, index: {}",
                self.tokens.len(),
                self.current
            ));
        }

        Ok(self.tokens[self.current])
    }

    /// Increments the `current` pointer and returns the next token.
    fn advance(&mut self) -> Result<Token<'_>, String> {
        self.current += 1;

        self.get_current_token()
    }

    /// Compares the next token to an expected token. Generates an error if the token doesn't
    /// match the expected one.
    fn expect(&'_ mut self, expected: Token) -> Result<Token<'_>, String> {
        self.current += 1;

        let token = self.get_current_token()?;

        // Using mem::discriminant takes the variant of the enum at face value,
        // ignoring the value stored inside.
        if token == expected
            || mem::discriminant(&expected) == mem::discriminant(&Token::Identifier(""))
            || mem::discriminant(&expected) == mem::discriminant(&Token::BinaryOperator(' '))
            || mem::discriminant(&expected) == mem::discriminant(&Token::ComparisonOperator(' '))
        {
            // println!("[?] {:?} == {:?}", token, expected);
            Ok(token)
        } else {
            // println!("[?] {:?} != {:?}", token, expected);
            Err(format!("{:?} doesn't match {:?}", token, expected))
        }
    }
}
