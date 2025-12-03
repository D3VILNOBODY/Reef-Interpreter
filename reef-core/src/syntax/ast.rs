#![allow(unused)]

/*
   This file stores all the necessary types needed for the parser.
   It constructs parse nodes from a vector of tokens.
*/

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExprOperator(pub char);

#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    LessThan,
    GreaterThan,
    EqualTo,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    CompoundStatement(Vec<Stmt>),
    LogStatement(Vec<Expr>),
    ReturnStatement(Expr),
    ForLoop {
        condition: Expr,
        body: Box<Stmt>,
    },
    VariableDeclaration {
        name: String,
        value: Option<Expr>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<FunctionParameter>,
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    NumberLiteral(f64),
    StringLiteral(String),
    NilLiteral,
    ComparisonExpression {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        operator: ComparisonOperator,
    },
    BinaryExpression {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        operator: BinaryExprOperator,
    },
    FunctionCall {
        name: String,
        arguments: Vec<FunctionArgument>,
    },
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    name: String,
}

#[derive(Debug, Clone)]
pub struct FunctionArgument {
    value: Expr,
}
