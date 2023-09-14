use super::Value;

type BuiltinFunction = fn(&Value) -> Result<Value, String>;

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Builtin(BuiltinFunction),
}

impl Function {
    pub fn call(&self, arg: &Value) -> Result<Value, String> {
        match self {
            Function::Builtin(builtin_func) => builtin_func(arg),
        }
    }
}

fn log(arg: &Value) -> Result<Value, String> {
    match arg {
        Value::Float(v) => Ok(Value::Float(v.ln())),
        _ => Err("log is only defined for float arg".into()),
    }
}
fn exp(arg: &Value) -> Result<Value, String> {
    match arg {
        Value::Float(v) => Ok(Value::Float(v.exp())),
        _ => Err("exp is only defined for float arg".into()),
    }
}
fn print(arg: &Value) -> Result<Value, String> {
    println!("{}", arg);
    Ok(Value::Float(0.0))
}

pub fn builtin(name: &str) -> Option<Function> {
    match name {
        "log" => Some(Function::Builtin(log)),
        "exp" => Some(Function::Builtin(exp)),
        "print" => Some(Function::Builtin(print)),
        _ => None,
    }
}
