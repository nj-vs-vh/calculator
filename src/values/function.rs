use std::collections::HashMap;

use super::Value;
use crate::errors::RuntimeError;
use crate::runtime::eval_assignment;
use crate::values::builtins::BuiltinFunction;
use crate::{parser::Expression, runtime::eval};

#[derive(Debug, Clone, PartialEq)]
pub struct UserDefinedFunction {
    pub name: String,
    pub params: Expression, // must be assignable-to
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
        arg_expr: &Expression,
        global_vars: &HashMap<String, Box<Value>>,
    ) -> Result<Box<Value>, RuntimeError> {
        let mut local_vars = global_vars.clone();
        match self {
            Function::Builtin(builtin_func) => {
                let arg_value = eval(arg_expr, &mut local_vars)?;
                builtin_func(&arg_value)
                    .map(|v| Box::new(v))
                    .map_err(|e| RuntimeError { errmsg: e })
            }
            Function::UserDefined(func) => {
                eval_assignment(&func.params, arg_expr, &mut local_vars)?;
                eval(&func.body, &mut local_vars)
            }
        }
    }
}
