use reef_syntax::common::*;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtRes};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeType {
    Number(f64),
    String(String),
    Boolean(Boolean),
    None,
    Error(String),
}

#[derive(Debug)]
pub struct Scope<'a> {
    variables: HashMap<String, RuntimeType>,
    parent: Option<&'a mut Scope<'a>>,
}

impl<'a> Display for Scope<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtRes {
        write!(
            f,
            "Scope<parent: {:?}, variables: {:?}>",
            self.parent, self.variables
        )
    }
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a mut Scope<'a>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: parent,
        }
    }

    pub fn get_variable(&self, name: &str) -> RuntimeType {
        let v = self.variables.get(name);

        match v {
            Some(v) => v.clone(),
            None => match &self.parent {
                Some(parent) => parent.get_variable(name),
                None => panic!("No variable called {} exists", name),
            },
        }
    }

    pub fn set_variable(&mut self, name: &str, value: RuntimeType) -> RuntimeType {
        if self.variables.contains_key(name) {
            panic!("Variable named {name} already exists. Did you mean to reassign it?")
        } else {
            self.variables.insert(name.to_string(), value);
            RuntimeType::None
        }
    }

    pub fn reassign_variable(&mut self, name: &str, value: RuntimeType) -> RuntimeType {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            RuntimeType::None
        } else {
            panic!("Attempt to reassign variable \"{name}\" which doesn't exist.")
        }
    }
}

impl Display for RuntimeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtRes {
        match self {
            Self::None => write!(f, "None")?,
            Self::Error(msg) => write!(f, "Error: {msg}")?,
            Self::Number(number) => write!(f, "{}", number)?,
            Self::String(string) => write!(f, "{}", string)?,
            Self::Boolean(boolean) => write!(
                f,
                "{}",
                match boolean {
                    Boolean::True => "true",
                    Boolean::False => "false",
                }
            )?,
        }

        Ok(())
    }
}
