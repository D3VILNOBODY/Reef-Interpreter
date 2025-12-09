use crate::common::*;

#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    LessThan,
    GreaterThan,
    EqualTo,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryExprOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    CompoundStatement(Vec<Stmt>), // { ...stmt }
    ExpressionStatement(Expr),    // any expr;
    LogStatement(Vec<Expr>),      // log ...expr;
    ReturnStatement(Expr),        // return expr;
    ForLoop {
        condition: Expr,
        body: Box<Stmt>,
    }, // for (condition) do { ...stmt }
    VariableDeclaration {
        name: String,
        value: Expr, // might change this to Option<Expr> to allow for uninitialised vars
    }, // var var_name = expr;
    FunctionDeclaration {
        name: String,
        parameters: Vec<FunctionParameter>,
        body: Box<Stmt>,
    }, // fun func_name(...params) { ...stmt }
}

#[derive(Debug, Clone)]
pub enum Expr {
    NumberLiteral(f64),
    StringLiteral(String),
    Identifier(String),
    GroupExpression(Box<Expr>),
    NegatedExpression(Box<Expr>),
    Boolean(Boolean),
    NilLiteral,

    // expr  > | < | <= | >= | == | != expr
    ComparisonExpression {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        operator: ComparisonOperator,
    },
    BinaryExpression {
        left_side: Box<Expr>,
        right_side: Box<Expr>,
        operator: BinaryExprOperator,
    },
    FunctionCall {
        func_name: String,
        arguments: Vec<FunctionArgument>,
    }, // func_name(...expr)
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct FunctionParameter {
    name: String,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct FunctionArgument {
    value: Expr,
}
