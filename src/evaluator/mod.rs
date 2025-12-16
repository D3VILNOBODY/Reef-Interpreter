use colored::Colorize;
use reef_syntax::ast::*;
use std::collections::HashMap;

mod types;
use types::*;

/// The evaluator is the part of the interpreter that actually
/// runs (evaluates) the code. It takes an input of statements
/// and evaluates each thing as it reads it.
#[derive(Debug)]
pub struct Evaluator {
    pub program: Vec<Stmt>,
    variables: HashMap<String, RuntimeType>,
    ptr: usize,
    debug: u8,
}

impl Evaluator {
    pub fn new(program: Vec<Stmt>, debug: u8) -> Self {
        Self {
            program,
            variables: HashMap::new(),
            debug,
            ptr: 0,
        }
    }

    pub fn evaluate_program(&mut self) {
        while self.ptr < self.program.len() {
            match self.get_current_statement() {
                Some(Stmt::ExpressionStatement(expr)) => self.evaluate_expression_statement(expr),
                Some(Stmt::LogStatement(args)) => self.evaluate_log_statement(args),
                Some(Stmt::VariableDeclaration { name, value }) => {
                    self.evaluate_variable_declaration(name, value)
                }
                Some(_stmt) => {
                    panic!("Unhandled statement {:?}", _stmt);
                }
                None => {
                    println!("Done");
                    break;
                }
            };
        }
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
            Expr::UnaryExpression(_operation, expression) => {
                let ret = self.evaluate_expression(*expression);

                match ret {
                    RuntimeType::Number(num) => RuntimeType::Number(-num),
                    _ => panic!("Cant perform a unary operation on {:?}", ret),
                }
            }
            Expr::GroupExpression(expression) => self.evaluate_expression(*expression),
            Expr::Boolean(boolean) => RuntimeType::Boolean(boolean),
            Expr::NumberLiteral(n) => RuntimeType::Number(n),
            Expr::StringLiteral(s) => RuntimeType::String(s),
            Expr::Identifier(ident) => self.get_variable(ident),
            _ => panic!("fn evaluate_expression: unhandled expression {:?}", expr),
        }
    }

    /// Runs a variable declaration statement and adds the variable to the global
    /// `self.variables` field.
    fn evaluate_variable_declaration(&mut self, name: String, value: Expr) -> RuntimeType {
        let value = self.evaluate_expression(value);

        self.set_variable(name, value);

        self.advance();

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
                val_to_print.push_str(&format!("{}, ", expr));
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
            _ => panic!("CANT DO A BINARY OPERATION ON ANYTHING BUT A NUMBER"),
        };

        match rhs {
            RuntimeType::Number(n) => rhs_n = n,
            _ => panic!("CANT DO A BINARY OPERATION ON ANYTHING BUT A NUMBER"),
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

    /// Gets a variable from the global `self.variables` field. Panics if the
    /// variable doesn't exist.
    fn get_variable(&self, name: String) -> RuntimeType {
        if self.debug >= 1 {
            println!("[log] Attempting to access variable named {name}...")
        }

        let var = self.variables.get(&name);

        if var.is_none() {
            panic!("Attempt to access undefined variable {name}");
        }

        var.unwrap().clone()
    }

    /// Sets a variable in the global `self.variables` field.
    fn set_variable(&mut self, name: String, value: RuntimeType) {
        if self.debug >= 1 {
            println!("[log] Creating variable {name} with value {value}...")
        }

        self.variables.insert(name, value);
    }

    fn log(&self, source: &str, value: RuntimeType) {
        println!("{}", format!("[{}] {}", source, value).bright_green());
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
