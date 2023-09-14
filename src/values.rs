use std::fmt::Display;

use crate::values::functions::Function;
pub mod functions;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Float(f32),
    String(String),
    Function(Function),
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Function(f) => match f {
                Function::Builtin(_) => "built-in function",
            },
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Float(v) => write!(f, "{}", v),
            Value::String(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{:?}", self),
        }
    }
}
