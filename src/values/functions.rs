use super::Value;

type BuiltinFunction = fn(&Value) -> Result<Value, String>;
use rand::Rng;

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
        Value::Int(v) => log(&Value::Float(*v as f32)),
        a => not_defined_for_arg("log", a),
    }
}
fn exp(arg: &Value) -> Result<Value, String> {
    match arg {
        Value::Float(v) => Ok(Value::Float(v.exp())),
        Value::Int(v) => exp(&Value::Float(*v as f32)),
        a => not_defined_for_arg("exp", a),
    }
}
fn print(arg: &Value) -> Result<Value, String> {
    println!("{}", arg);
    Ok(Value::Float(0.0))
}
fn length(arg: &Value) -> Result<Value, String> {
    match arg {
        Value::String(s) => Ok(Value::Int(s.len() as i32)),
        a => not_defined_for_arg("length", a),
    }
}
fn random(arg: &Value) -> Result<Value, String> {
    let mut rng = rand::thread_rng();
    if let Value::Nothing = arg {
        Ok(Value::Float(rng.gen::<f32>()))
    } else {
        Err("random accpets no arguments".into())
    }
}

pub fn builtin(name: &str) -> Option<Function> {
    match name {
        "log" => Some(Function::Builtin(log)),
        "exp" => Some(Function::Builtin(exp)),
        "print" => Some(Function::Builtin(print)),
        "length" => Some(Function::Builtin(length)),
        "random" => Some(Function::Builtin(random)),
        _ => None,
    }
}

fn not_defined_for_arg(func_name: &str, arg: &Value) -> Result<Value, String> {
    Err(format!(
        "{} is not defined for arg of type \"{}\"",
        func_name,
        arg.type_name()
    ))
}
