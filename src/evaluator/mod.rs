use reef_syntax::ast::*;
use std::backtrace::Backtrace;

/// The evaluator is the part of the interpreter that actually
/// runs (evaluates) the code. It takes an input of statements
/// and evaluates each thing as it reads it.
pub struct Evaluator {
    pub program: Vec<Stmt>,
    ptr: usize,
}

#[derive(Debug)]
enum Boolean {
    True,
    False,
}

#[derive(Debug)]
enum RuntimeType {
    Number(f64),
    String(String),
    Boolean(Boolean),
}

impl Evaluator {
    pub fn new(program: Vec<Stmt>) -> Self {
        Self { program, ptr: 0 }
    }

    pub fn evaluate_program(&mut self) {
        while self.ptr < self.program.len() {
            match self.get_current_statement() {
                Some(Stmt::ExpressionStatement(expr)) => self.evaluate_expression_statement(expr),
                Some(_stmt) => {
                    panic!("Unhandled statement {:?}", _stmt);
                }
                None => break,
            }
        }
    }

    fn evaluate_expression_statement(&mut self, expr: Expr) {
        let x = self.evaluate_expression(expr);
        println!("{}", Backtrace::capture());
        println!("///////////////////////////////////////////////////");
    }

    fn evaluate_expression(&mut self, expr: Expr) -> RuntimeType {
        match expr {
            Expr::BinaryExpression {
                left_side,
                right_side,
                operator,
            } => self.evaluate_binary_expression(left_side, right_side, operator),
            Expr::NumberLiteral(n) => RuntimeType::Number(n),
            Expr::StringLiteral(s) => RuntimeType::String(s),
            _ => panic!("Can only do binary expressions rn sorry bud"),
        }
    }

    fn evaluate_binary_expression(
        &mut self,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: BinaryExprOperator,
    ) -> RuntimeType {
        let lhs = self.evaluate_expression(*lhs);
        let rhs = self.evaluate_expression(*rhs);

        // Undefined because the actual values are set later.
        let mut lhs_n: f64;
        let mut rhs_n: f64;
        let mut final_num: f64;

        match lhs {
            RuntimeType::Number(n) => lhs_n = n,
            _ => panic!("CANT DO A BINARY OPERATION ON ANYTHING BUT A NUMBER"),
        };

        match rhs {
            RuntimeType::Number(n) => rhs_n = n,
            _ => panic!("CANT DO A BINARY OPERATION ON ANYTHING BUT A NUMBER"),
        };

        match op {
            BinaryExprOperator::Plus => final_num = lhs_n + rhs_n,
            BinaryExprOperator::Minus => final_num = lhs_n - rhs_n,
            BinaryExprOperator::Multiply => final_num = lhs_n * rhs_n,
            BinaryExprOperator::Divide => final_num = lhs_n / rhs_n,
            BinaryExprOperator::Modulus => final_num = lhs_n % rhs_n,
        };

        self.advance();

        RuntimeType::Number(final_num)
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
