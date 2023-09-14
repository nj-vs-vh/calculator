use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Float(v) => write!(f, "{}", v),
            _ => write!(f, "{:?}", self),
        }
    }
}
