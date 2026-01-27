use reef_syntax::{ast::*, common::*, token::Token};
use std::{backtrace::Backtrace, mem};

/// The parser is responsible for taking a vector of tokens
/// and producing a tree-like representation of the program
/// which is fed to the evaluator.
#[derive(Clone)]
pub struct Parser<'a> {
    pub program: Vec<Stmt>,
    tokens: Vec<Token<'a>>,
    current: usize,
    debug: u8,
}

#[derive(Debug)]
pub enum ParserError {
    SyntaxError { position: usize, message: String },
    UnknownToken { position: usize },
    CurrentIndexOutOfBounds(usize),
}

impl<'a> Parser<'a> {
    /// Constructs a new parser, taking a vector of tokens
    /// produced by the scanner.
    pub fn new(tokens: Vec<Token<'a>>, debug: u8) -> Self {
        Self {
            tokens,
            debug,
            current: 0,
            program: vec![],
        }
    }

    /// Top level function for parsing every token.
    pub fn parse_all(&mut self) -> Result<(), ParserError> {
        while self.current < self.tokens.len() {
            let n = self.next_statement()?;

            self.add_statement(n.unwrap());
        }

        Ok(())
    }

    fn next_statement(&mut self) -> Result<Option<Stmt>, ParserError> {
        match self.get_current_token() {
            // Statements
            Some(Token::Keyword("var")) => Ok(Some(self.variable_declaration()?)),
            Some(Token::Keyword("log")) => Ok(Some(self.log_statement()?)),
            Some(Token::Keyword("if")) => Ok(Some(self.if_statement()?)),
            Some(Token::Delimiter('{')) => Ok(Some(self.block_statement()?)),

            // Expression statements
            Some(Token::Keyword("true"))
            | Some(Token::Keyword("false"))
            | Some(Token::String(_))
            | Some(Token::Number(_))
            | Some(Token::BinaryOperator('-'))
            | Some(Token::Delimiter('(')) => Ok(Some(self.expression_statement()?)),

            Some(Token::Identifier(_)) => {
                let next = self.lookahead(1);

                match next {
                    Some(Token::Equals) => Ok(Some(self.variable_reassignment()?)),
                    _ => Ok(Some(self.expression_statement()?)),
                }
            }

            Some(Token::Delimiter(';')) => {
                self.advance();
                Ok(Some(Stmt::EmptyStatement))
            }

            _t => {
                println!("UNKNOWN TOKEN: {:?}", _t);
                Err(ParserError::UnknownToken {
                    position: self.current,
                })
            }
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        let condition: Expr;
        let body: Stmt;

        self.expect(Token::Delimiter('('))?;
        self.advance();

        condition = self.expression()?;

        self.expect(Token::Delimiter(')'))?;
        self.expect(Token::Keyword("then"))?;
        self.advance();

        body = self.block_statement()?;

        Ok(Stmt::IfStatement {
            condition: condition,
            body: Box::new(body),
        })
    }

    fn variable_reassignment(&mut self) -> Result<Stmt, ParserError> {
        let name = match self.get_current_token() {
            Some(Token::Identifier(i)) => String::from(i),
            _ => {
                return Err(ParserError::SyntaxError {
                    position: self.current,
                    message: "Help".to_string(),
                })
            }
        };

        self.expect(Token::Equals)?;
        self.advance();

        let value = self.expression()?;

        self.expect(Token::Delimiter(';'))?;

        Ok(Stmt::VariableReassignment {
            name: String::from(name),
            value: value,
        })
    }

    /// The base method for parsing any kind of expression.
    fn expression(&mut self) -> Result<Expr, ParserError> {
        match self.get_current_token() {
            Some(Token::Keyword("true")) => Ok(Expr::Boolean(Boolean::True)),
            Some(Token::Keyword("false")) => Ok(Expr::Boolean(Boolean::False)),
            Some(Token::Keyword("nil")) => Ok(Expr::NilLiteral),
            Some(Token::Delimiter('(')) => Ok(self.group_expression()?),
            Some(Token::String(s)) => Ok(create_string_literal(s)),
            Some(Token::BinaryOperator('-')) => {
                // Skip past the '-'. May cause issues down the line but idc.
                self.advance();

                match self.get_current_token() {
                    Some(Token::Number(_))
                    | Some(Token::Identifier(_))
                    | Some(Token::Delimiter('(')) => Ok(Expr::UnaryExpression(
                        UnaryOperation::Minus,
                        Box::new(self.expression()?),
                    )),
                    _ => Err(ParserError::SyntaxError {
                        position: self.current,
                        message: format!("Wrong kind of argument after a unary operater bro!"),
                    }),
                }
            }
            Some(Token::Number(n)) => {
                let next = self.lookahead(1);

                match next {
                    Some(Token::BinaryOperator(_)) => Ok(self.binary_expression()?),
                    Some(Token::ComparisonOperator(_)) => Ok(self.comparison_expression()?),
                    _ => Ok(create_number_literal(n)),
                }
            }
            Some(Token::Identifier(ident)) => {
                // TODO: abstract this to a different function
                let next = self.lookahead(1);

                match next {
                    Some(Token::BinaryOperator(_)) => Ok(self.binary_expression()?),
                    _ => Ok(Expr::Identifier(String::from(ident))),
                }
            }
            _token => panic!("[!] {:?}", _token),
        }
    }

    fn log_statement(&mut self) -> Result<Stmt, ParserError> {
        // log expr1, expr2, expr3;
        // log expr1;
        // log;

        // Skip past the "log" keyword.
        self.advance();

        let expressions = self.parse_call_site_arguments()?;
        // let expressions = vec![self.expression()?];

        self.expect(Token::Delimiter(';'))?;

        Ok(Stmt::LogStatement(expressions))
    }

    fn block_statement(&mut self) -> Result<Stmt, ParserError> {
        // Skip the '{'.
        self.advance();
        println!("{:?}", self.get_current_token());

        let mut statements: Vec<Stmt> = vec![];

        while self.current < self.tokens.len() && self.get_current_token() != None {
            let s = self.next_statement()?;
            statements.push(s.unwrap());

            match self.get_current_token() {
                Some(Token::Delimiter('}')) => {
                    self.advance();
                    break;
                }
                None => panic!("Expected '}}' to close a compound statement."),
                _ => continue,
            }
        }

        Ok(Stmt::BlockStatement(statements))
    }

    /// Collects a list of arguments (expressions) separated by commas.
    fn parse_call_site_arguments(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut collected: Vec<Expr> = vec![];

        // Im not sure why this doesnt work if i replace it all with self.expression(),
        // so im just going to leave it and pray it keeps working!
        while let Some(token) = self.get_current_token() {
            let expr = match token {
                Token::String(_)
                | Token::Number(_)
                | Token::Identifier(_)
                | Token::Delimiter('(')
                | Token::BinaryOperator('-')
                | Token::Keyword("true")
                | Token::Keyword("false") => self.expression()?,

                _ => break,
            };
            collected.push(expr);

            let next = self.lookahead(1);

            match next {
                Some(Token::Delimiter(',')) => {
                    // Really janky but the first advance skips the expression,
                    // the second one skips the comma. Im a lil stupid so just
                    // let it slide.
                    self.advance();
                    // dbg!(self.get_current_token());
                    self.advance();
                    // dbg!(self.get_current_token());
                    continue;
                }
                _ => break,
            }
        }

        Ok(collected)
    }

    /// Generates an expression statement. An expression statement is simply an expression
    /// but as a statement. For example, `10 + 5;` is an expression statement.
    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.expect(Token::Delimiter(';'))?;

        Ok(Stmt::ExpressionStatement(expr))
    }

    /// Generates a group expression, which is any expression inside of brackets.
    fn group_expression(&mut self) -> Result<Expr, ParserError> {
        // Starts with a '(', should also end with a ')'.

        // Skip the opening bracket
        self.advance();

        let inner = self.expression()?;

        self.expect(Token::Delimiter(')'))?;

        Ok(Expr::GroupExpression(Box::new(inner)))
    }

    /// Generates a binary expression, returning Ok if it was successful.
    fn binary_expression(&mut self) -> Result<Expr, ParserError> {
        let lhs: Expr;
        let rhs: Expr;
        let operator: BinaryExprOperator;

        // The left hand side of the binary expression. Creates a number from a Number token,
        // a string from a String token, and keeps track of identifiers. If the current token
        // isn't a valid type, it simply is turned into Nil.
        lhs = match self.get_current_token() {
            Some(Token::Keyword("true")) => Expr::Boolean(Boolean::True),
            Some(Token::Keyword("false")) => Expr::Boolean(Boolean::False),
            Some(Token::Delimiter('(')) => self.group_expression()?,
            Some(Token::String(s)) => create_string_literal(s),
            Some(Token::BinaryOperator('-')) => {
                // Skip past the '-'. May cause issues down the line but idc.
                self.advance();
                match self.get_current_token() {
                    Some(Token::Number(n)) => create_number_literal(&*format!("-{}", n)),
                    _ => {
                        return Err(ParserError::SyntaxError {
                            position: self.current,
                            message: String::new(),
                        })
                    }
                }
            }
            Some(Token::Number(n)) => {
                let next = self.lookahead(1);

                match next {
                    _ => create_number_literal(n),
                }
            }
            Some(Token::Identifier(ident)) => {
                // TODO: abstract this to a different function
                let next = self.lookahead(1);

                match next {
                    _ => Expr::Identifier(String::from(ident)),
                }
            }
            _ => Expr::NilLiteral,
        };

        // Creates a BinaryExprOperator containing the operator used in the binary expression.
        // Panics if the token isn't a binary operator.
        operator = match self.expect(Token::BinaryOperator(' '))? {
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
                return Err(ParserError::SyntaxError {
                    position: self.current,
                    message: String::new(),
                })
            }
        };

        // Pass the operator.
        self.advance();

        // The right hand side of the expression. Could be any expression, so the base expression
        // method is used.
        rhs = self.expression()?;

        Ok(Expr::BinaryExpression {
            left_side: Box::new(lhs),
            right_side: Box::new(rhs),
            operator,
        })
    }

    fn comparison_expression(&mut self) -> Result<Expr, ParserError> {
        let lhs: Expr;
        let rhs: Expr;
        let operator: ComparisonOperator;

        lhs = match self.get_current_token() {
            Some(Token::Keyword("true")) => Expr::Boolean(Boolean::True),
            Some(Token::Keyword("false")) => Expr::Boolean(Boolean::False),
            Some(Token::Delimiter('(')) => self.group_expression()?,
            Some(Token::String(s)) => create_string_literal(s),
            Some(Token::BinaryOperator('-')) => {
                // Skip past the '-'. May cause issues down the line but idc.
                self.advance();
                match self.get_current_token() {
                    Some(Token::Number(n)) => create_number_literal(&*format!("-{}", n)),
                    _ => {
                        return Err(ParserError::SyntaxError {
                            position: self.current,
                            message: String::new(),
                        })
                    }
                }
            }
            Some(Token::Number(n)) => {
                let next = self.lookahead(1);

                match next {
                    _ => create_number_literal(n),
                }
            }
            Some(Token::Identifier(ident)) => {
                // TODO: abstract this to a different function
                let next = self.lookahead(1);

                match next {
                    _ => Expr::Identifier(String::from(ident)),
                }
            }
            _ => Expr::NilLiteral,
        };

        operator = match self.expect(Token::ComparisonOperator(ComparisonOperator::Or))? {
            Token::ComparisonOperator(op) => op,
            _t => {
                return Err(ParserError::SyntaxError {
                    position: self.current,
                    message: String::new(),
                })
            }
        };

        self.advance();

        rhs = self.expression()?;

        Ok(Expr::ComparisonExpression {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            operator,
        })
    }

    /// Creates a variable declaration with a name (identifier) and a value (expression).
    fn variable_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = match self.expect(Token::Identifier(""))? {
            Token::Identifier(i) => String::from(i),
            _ => {
                return Err(ParserError::SyntaxError {
                    position: self.current,
                    message: "Expected an identifier after keyword `var`".to_string(),
                })
            }
        };

        self.expect(Token::BinaryOperator('='))?;

        // Skip '='
        self.advance();

        let value = self.expression()?;

        self.expect(Token::Delimiter(';'))?;

        Ok(Stmt::VariableDeclaration { name, value })
    }

    /// Pushes `node` to `self.program`.
    fn add_statement(&mut self, node: Stmt) {
        if self.debug >= 1 {
            println!("[log] Adding statement {:?}...", node);
        }

        self.program.push(node);
    }

    /// Gets the token at `current + distance`.
    fn lookahead(&self, distance: usize) -> Option<Token<'_>> {
        if self.current + distance >= self.tokens.len() {
            if self.debug >= 1 {
                println!(
                    "[log] Looked ahead {} from {} and found no token.",
                    distance, self.current
                );
            }
            return None;
        }

        let token = self.tokens[self.current + distance];

        if self.debug >= 1 {
            println!(
                "[log] Looked ahead {} from {} and current token is {:?}",
                distance, self.current, token
            );
        }

        Some(token)
    }

    /// Returns the token at index `current`.
    fn get_current_token(&self) -> Option<Token<'_>> {
        if self.current >= self.tokens.len() {
            return None;
        }

        Some(self.tokens[self.current])
    }

    /// Increments the `current` pointer and returns the next token.
    fn advance(&mut self) {
        self.current += 1;

        if self.debug >= 1 {
            println!(
                "[log] Advanced. Index is {} and current token is {:?}",
                self.current,
                self.get_current_token()
            );
        }
    }

    /// Compares the next token to an expected token. Generates an error if the token doesn't
    /// match the expected one.
    fn expect(&'_ mut self, expected: Token) -> Result<Token<'_>, ParserError> {
        self.advance();

        let token = self.get_current_token();

        // At the end of the file.
        if token.is_none() {
            use ParserError::*;
            use Token::*;

            match expected {
                Delimiter(';') => SyntaxError {
                    position: self.current,
                    message: format!("Expected semicolon"),
                },
                Number(_) => SyntaxError {
                    position: self.current,
                    message: format!("Expected Number"),
                },
                String(_) => SyntaxError {
                    position: self.current,
                    message: format!("Expected String"),
                },
                BinaryOperator(op) => SyntaxError {
                    position: self.current,
                    message: format!("Expected {}", op),
                },
                _ => CurrentIndexOutOfBounds(self.current),
            };
        }

        // Using mem::discriminant takes the variant of the enum at face value,
        // ignoring the value stored inside.
        if token.is_some() && token.unwrap() == expected
            || mem::discriminant(&expected) == mem::discriminant(&Token::Identifier(""))
            || mem::discriminant(&expected) == mem::discriminant(&Token::BinaryOperator(' '))
            || mem::discriminant(&expected)
                == mem::discriminant(&Token::ComparisonOperator(ComparisonOperator::Or))
        {
            // println!("[?] {:?} == {:?}", token, expected);
            Ok(token.unwrap())
        } else {
            // println!("[?] {:?} != {:?}", token, expected);
            println!("{:?}", &self.tokens[0..self.current]);
            Err(ParserError::SyntaxError {
                position: self.current,
                message: format!(
                    "Expected {}, got {}. Backtrace: {}",
                    expected,
                    token.unwrap_or(Token::EndOfFile),
                    Backtrace::capture()
                ),
            })
        }
    }
}

/// Attempts to convert n into a number and returns a wrapper around n.
fn create_number_literal(n: &str) -> Expr {
    let p = n.parse::<f64>();

    match p {
        Ok(v) => Expr::NumberLiteral(v),
        Err(e) => {
            panic!("Error unwrapping {}: {:?}", n, e);
        }
    }
}

/// Creates a string literal wrapper which contains the string `s`.
fn create_string_literal(s: &str) -> Expr {
    Expr::StringLiteral(String::from(s))
}
