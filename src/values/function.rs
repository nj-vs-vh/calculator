use crate::parser::Expression;
use crate::values::builtins::BuiltinFunction;

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
