use colored::Colorize;
use reef_syntax::{
    ast::*,
    common::{Boolean, ComparisonOperator},
};
use std::fmt::{Display, Formatter, Result as FmtRes};

use super::types::*;

/// The evaluator is the part of the interpreter that actually
/// runs (evaluates) the code. It takes an input of statements
/// and evaluates each statement as it reads it.
#[derive(Debug)]
pub struct Evaluator<'a> {
    pub program: Vec<Stmt>,
    scope: Scope<'a>,
    ptr: usize,
    _debug: u8,
}

impl<'a> Display for Evaluator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtRes {
        write!(
            f,
            "Evaluator<scope: {}, ptr: {}, _debug: {}, program: {:?}>",
            self.scope, self.ptr, self._debug, self.program
        )
    }
}

impl<'a> Evaluator<'a> {
    pub fn new(program: Vec<Stmt>, debug: u8) -> Self {
        Self {
            program,
            scope: Scope::new(None),
            _debug: debug,
            ptr: 0,
        }
    }

    #[allow(unused)]
    pub fn get_main_scope(&self) -> &Scope<'a> {
        &self.scope
    }

    pub fn evaluate_program(&mut self) {
        while self.ptr < self.program.len() {
            self.evaluate_statement(self.get_current_statement());
        }
    }

    fn evaluate_statement(&mut self, statement: Option<Stmt>) {
        match statement {
            Some(Stmt::ExpressionStatement(expr)) => self.evaluate_expression_statement(expr),
            Some(Stmt::LogStatement(args)) => self.evaluate_log_statement(args),
            Some(Stmt::IfStatement { condition, body }) => {
                self.evaluate_if_statement(condition, body)
            }
            Some(Stmt::VariableDeclaration { name, value }) => {
                self.evaluate_variable_declaration(name, value)
            }
            Some(Stmt::VariableReassignment { name, value }) => {
                self.evaluate_variable_reassignment(name, value)
            }
            Some(Stmt::BlockStatement(statements)) => self.evaluate_block_statement(statements),
            Some(Stmt::EmptyStatement) => self.evaluate_empty_statement(),
            Some(_stmt) => {
                self.error(&format!("Unhandled statement {:?}", _stmt));
            }
            None => RuntimeType::None,
        };
    }

    fn evaluate_empty_statement(&mut self) -> RuntimeType {
        self.advance();
        RuntimeType::None
    }

    fn evaluate_expression_statement(&mut self, expr: Expr) -> RuntimeType {
        let v = self.evaluate_expression(expr);
        self.log("expr_stmt", v);
        self.advance();
        RuntimeType::None
    }

    fn evaluate_expression(&mut self, expr: Expr) -> RuntimeType {
        match expr {
            Expr::BinaryExpression {
                left_side,
                right_side,
                operator,
            } => self.evaluate_binary_expression(left_side, right_side, operator),
            Expr::ComparisonExpression { lhs, rhs, operator } => {
                self.evaluate_comparison_expression(lhs, rhs, operator)
            }
            Expr::UnaryExpression(_operation, expression) => {
                let ret = self.evaluate_expression(*expression);

                match ret {
                    RuntimeType::Number(num) => RuntimeType::Number(-num),
                    _ => self.error(&format!("Cant perform a unary operation on {:?}", ret)),
                }
            }
            Expr::GroupExpression(expression) => self.evaluate_expression(*expression),
            Expr::Boolean(boolean) => RuntimeType::Boolean(boolean),
            Expr::NumberLiteral(n) => RuntimeType::Number(n),
            Expr::StringLiteral(s) => RuntimeType::String(s),
            Expr::Identifier(ident) => self.scope.get_variable(&ident),
            _ => self.error(&format!("Unable to evaluate expression {:?}", expr)),
        }
    }

    /// Runs a variable declaration statement and adds the variable to the global
    /// `self.variables` field.
    fn evaluate_variable_declaration(&mut self, name: String, value: Expr) -> RuntimeType {
        let value = self.evaluate_expression(value);
        self.scope.set_variable(&name, value);
        self.advance();
        RuntimeType::None
    }

    /// Runs a variable declaration statement and adds the variable to the global
    /// `self.variables` field.
    fn evaluate_variable_reassignment(&mut self, name: String, value: Expr) -> RuntimeType {
        let value = self.evaluate_expression(value);
        self.scope.reassign_variable(&name, value);
        self.advance();
        RuntimeType::None
    }

    fn evaluate_if_statement(&mut self, condition: Expr, body: Box<Stmt>) -> RuntimeType {
        let c = self.evaluate_expression(condition);

        match c {
            RuntimeType::Boolean(b) => match b {
                Boolean::True => {
                    self.evaluate_block_statement(match *body {
                        Stmt::BlockStatement(statements) => statements,
                        _ => self
                            .error("Expected a block statement following if statement condition"),
                    });
                }
                Boolean::False => self.advance(),
            },
            _ => self.error("If statement condition didnt evaluate to a boolean"),
        }

        RuntimeType::None
    }

    fn evaluate_comparison_expression(
        &mut self,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        operator: ComparisonOperator,
    ) -> RuntimeType {
        let lhs = self.evaluate_expression(*lhs);
        let rhs = self.evaluate_expression(*rhs);

        match operator {
            ComparisonOperator::And => {
                let lhs_v = match lhs {
                    RuntimeType::Boolean(b) => b,
                    _ => self.error(
                        "Expected both sides of comparison expression to evaluate to a boolean",
                    ),
                };

                let rhs_v = match rhs {
                    RuntimeType::Boolean(b) => b,
                    _ => self.error(
                        "Expected both sides of comparison expression to evaluate to a boolean",
                    ),
                };

                if lhs_v == Boolean::True && rhs_v == Boolean::True {
                    RuntimeType::Boolean(Boolean::True)
                } else {
                    RuntimeType::Boolean(Boolean::False)
                }
            }
            ComparisonOperator::Or => {
                let lhs_v = match lhs {
                    RuntimeType::Boolean(b) => b,
                    _ => self.error(
                        "Expected both sides of comparison expression to evaluate to a boolean",
                    ),
                };

                let rhs_v = match rhs {
                    RuntimeType::Boolean(b) => b,
                    _ => self.error(
                        "Expected both sides of comparison expression to evaluate to a boolean",
                    ),
                };

                if lhs_v == Boolean::True || rhs_v == Boolean::True {
                    RuntimeType::Boolean(Boolean::True)
                } else {
                    RuntimeType::Boolean(Boolean::False)
                }
            }
            ComparisonOperator::EqualTo => RuntimeType::Boolean(match lhs == rhs {
                true => Boolean::True,
                false => Boolean::False,
            }),
            ComparisonOperator::NotEqualTo => RuntimeType::Boolean(match lhs != rhs {
                true => Boolean::True,
                false => Boolean::False,
            }),
            ComparisonOperator::GreaterThan => self.error("Greater than is not implemented"),
            ComparisonOperator::LessThan => self.error("Less than is not implemented"),
            ComparisonOperator::LessThanOrEqualTo => {
                self.error("Less than/equal to is not implemented")
            }
            ComparisonOperator::GreaterThanOrEqualTo => {
                self.error("Greater than/equal to is not implemented")
            }
        }
    }

    fn evaluate_block_statement(&mut self, statements: Vec<Stmt>) -> RuntimeType {
        for statement in statements {
            self.evaluate_statement(Some(statement));
        }

        RuntimeType::None
    }

    /// Runs a log statement, printing all of its arguments one after another in
    /// one string.
    fn evaluate_log_statement(&mut self, args: Vec<Expr>) -> RuntimeType {
        let mut val_to_print = String::new();

        let mut ptr = 0;
        while ptr < args.len() {
            let expr = self.evaluate_expression(args.get(ptr).unwrap().clone());

            if ptr == args.len() - 1 {
                val_to_print.push_str(&format!("{}", expr));
            } else {
                val_to_print.push_str(&format!("{} ", expr));
            }

            ptr += 1;
        }

        println!("{}", val_to_print);

        self.advance();

        RuntimeType::None
    }

    /// Evaluates the value of a binary expression. For example 1 + 2 will
    /// evaluate to the runtime value of Number(3).
    fn evaluate_binary_expression(
        &mut self,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        operator: BinaryExprOperator,
    ) -> RuntimeType {
        let lhs = self.evaluate_expression(*lhs);
        let rhs = self.evaluate_expression(*rhs);

        // Undefined because the actual values are set later.
        let lhs_n: f64;
        let rhs_n: f64;
        let final_num: f64;

        match lhs {
            RuntimeType::Number(n) => lhs_n = n,
            _ => self.error("Cannot perform binary operations on anything that isnt a number"),
        };

        match rhs {
            RuntimeType::Number(n) => rhs_n = n,
            _ => self.error("Cannot perform binary operations on anything that isnt a number"),
        };

        match operator {
            BinaryExprOperator::Plus => final_num = lhs_n + rhs_n,
            BinaryExprOperator::Minus => final_num = lhs_n - rhs_n,
            BinaryExprOperator::Multiply => final_num = lhs_n * rhs_n,
            BinaryExprOperator::Divide => final_num = lhs_n / rhs_n,
            BinaryExprOperator::Modulus => final_num = lhs_n % rhs_n,
        };

        RuntimeType::Number(final_num)
    }

    fn log(&self, source: &str, value: RuntimeType) {
        println!("{}", format!("[{}] {}", source, value).bright_green());
    }

    fn error(&self, value: &str) -> ! {
        panic!("{}", format!("[error] {}", value).bright_red());
    }

    fn get_current_statement(&self) -> Option<Stmt> {
        if self.ptr >= self.program.len() {
            return None;
        }

        Some(self.program[self.ptr].clone())
    }

    fn advance(&mut self) {
        self.ptr += 1;
    }
}

/// Converts an expression to a boolean value. Useful for
/// comparison expressions which require both sides to be
/// booleans.
fn expr_to_boolean(expr: &Expr) -> Boolean {
    match *expr {
        Expr::NilLiteral | Expr::Boolean(Boolean::False) => Boolean::False,
        _ => Boolean::True,
    }
}
