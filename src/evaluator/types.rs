use reef_syntax::common::*;
use std::fmt::{write, Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
pub enum RuntimeType {
    Number(f64),
    String(String),
    Boolean(Boolean),
    None,
}

impl Display for RuntimeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::None => write!(f, "None")?,
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
