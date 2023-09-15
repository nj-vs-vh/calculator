use std::collections::HashMap;

use super::Value;
use crate::values::builtins::BuiltinFunction;
use crate::{parser::Expression, runtime::eval};

#[derive(Debug, Clone, PartialEq)]
pub struct UserDefinedFunction {
    pub name: String,
    pub arg_name: String,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Builtin(BuiltinFunction),
    UserDefined(UserDefinedFunction),
}

impl Function {
    pub fn call(
        &self,
        arg: Box<Value>,
        global_vars: &HashMap<String, Box<Value>>,
    ) -> Result<Box<Value>, String> {
        match self {
            Function::Builtin(builtin_func) => builtin_func(&arg).map(|v| Box::new(v)),
            Function::UserDefined(func) => {
                let mut local_vars = global_vars.clone();
                local_vars.insert(func.arg_name.clone(), arg);
                eval(&func.body, &mut local_vars).map_err(|e| e.errmsg)
            }
        }
    }
}
