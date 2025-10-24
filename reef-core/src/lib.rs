use std::fmt::{Display, Formatter, Result};

// pub mod parser;
pub mod scanner;
pub mod syntax;

pub trait ReefDebuggable {
    fn debug_write_to_file(&self, file_path: &str);
    fn debug(&self);
}
