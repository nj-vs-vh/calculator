use std::fmt::Display;

use crate::values::functions::Function;
pub mod functions;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nothing,
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Function(Function),
    Tuple(Vec<Box<Value>>),
    // service values for control flow
    Returned(Box<Value>),
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Nothing => "nothing",
            Value::Returned(_) => "returned value",
            Value::Int(_) => "integer",
            Value::Float(_) => "floating point number",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Tuple(_) => "tuple",
            Value::Function(f) => match f {
                Function::Builtin(_) => "built-in function",
                Function::UserDefined(_) => "function",
            },
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Returned(v) => write!(f, "returned {}", v),
            Value::Nothing => write!(f, "nothing"),
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(v) => write!(f, "{}", if *v { "True" } else { "False" }),
            Value::Tuple(vec) => {
                write!(f, "(")?;
                for (idx, elem) in vec.iter().enumerate() {
                    write!(f, "{}", elem)?;
                    if idx < vec.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
            _ => write!(f, "{:?}", self),
        }
    }
}
