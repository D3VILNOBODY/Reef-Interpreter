use std::fmt::{Display, Formatter};

trait NodeBehaviour {
    fn add_child(&mut self, node: dyn NodeBehaviour);
}

pub enum Node {
    Program { children: Vec<Node> },

    Statement { expressions: Vec<Node> },

    MultiplicativeExpression { lhs: Box<Node>, rhs: Box<Node>, operator: String },
    AdditiveExpression { lhs: Box<Node>, rhs: Box<Node>, operator: String },

    NumberLiteral(f64),
    StringLiteral(String),
}

pub struct Statement {}
