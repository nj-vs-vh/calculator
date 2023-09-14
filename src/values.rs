use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
    String(String),
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Float(_) => "float",
            Value::String(_) => "string",
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
