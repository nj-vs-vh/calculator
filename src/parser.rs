use crate::{
    bracket::{Bracket, BracketSide, BracketStack, BracketType},
    errors::ParserError,
    tokenizer::{Token, TokenType},
    values::{
        function::{Function, UserDefinedFunction},
        Value,
    },
};
use std::{cmp::min, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Assign,
    IsEq,
    IsGt,
    IsLt,
    FunctionCall,
    FormTuple,
    AppendToTuple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Return,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Op {
    Unary(UnaryOp),
    Binary(BinaryOp),
}

const ORDER_OF_PRECEDENCE: [Op; 13] = [
    Op::Unary(UnaryOp::Return),
    Op::Binary(BinaryOp::Assign),
    Op::Binary(BinaryOp::FormTuple),
    Op::Binary(BinaryOp::IsEq),
    Op::Binary(BinaryOp::IsLt),
    Op::Binary(BinaryOp::IsGt),
    Op::Binary(BinaryOp::Add),
    Op::Binary(BinaryOp::Sub),
    Op::Binary(BinaryOp::Mul),
    Op::Binary(BinaryOp::Div),
    Op::Unary(UnaryOp::Neg),
    Op::Binary(BinaryOp::Pow),
    Op::Binary(BinaryOp::FunctionCall),
];

impl Op {
    fn precedence(&self) -> usize {
        ORDER_OF_PRECEDENCE
            .iter()
            .enumerate()
            .find(|(_, op)| *op == self)
            .map(|(idx, _)| idx)
            .unwrap_or(usize::MAX)
    }

    fn is_rtl(&self) -> bool {
        *self == Op::Binary(BinaryOp::Assign) || *self == Op::Binary(BinaryOp::FunctionCall)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Value(Rc<Value>),
    Variable(String),
    BinaryOperation {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOperation {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    Scope {
        body: Vec<Expression>,
        is_returnable: bool, // = can be returned from
    },
    If {
        condition: Box<Expression>,
        if_true: Box<Expression>,
        if_false: Option<Box<Expression>>,
    },
    While {
        condition: Box<Expression>,
        body: Box<Expression>,
        if_completed: Option<Box<Expression>>,
    },
}

pub fn parse<'a>(tokens: &'a [Token<'a>]) -> Result<Expression, ParserError<'a>> {
    parse_scope(tokens, true)
}

pub fn parse_scope<'a>(
    tokens: &'a [Token<'a>],
    is_returnable: bool,
) -> Result<Expression, ParserError<'a>> {
    let mut body: Vec<Expression> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let expr: Expression;
        (expr, i) = consume_expression(tokens, i, None, false)?;
        i += 1; // skipping expression end
        body.push(expr);
    }
    return Ok(Expression::Scope {
        body,
        is_returnable,
    });
}

fn consume_expression<'a>(
    tokens: &'a [Token<'a>],
    i: usize,
    outer_op: Option<Op>,
    terminate_on_unexpected_token: bool,
) -> Result<(Expression, usize), ParserError<'a>> {
    let mut result: Option<Expression> = None;
    let mut left: Option<Expression>;
    let mut prev_op: Option<Op> = None;
    let mut i = i;
    loop {
        i = skip_comments(tokens, i);
        (left, i) = if result.is_none() {
            consume_operand(tokens, i)?
        } else {
            (result, i)
        };

        i = skip_comments(tokens, i);
        if let Some(left) = left {
            if i >= tokens.len() || tokens[i].t == TokenType::ExprEnd {
                return Ok((left, min(i, tokens.len())));
            }
            let next_binary_op = match tokens[i].t {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Sub,
                TokenType::Star => BinaryOp::Mul,
                TokenType::Slash => BinaryOp::Div,
                TokenType::Caret => BinaryOp::Pow,
                TokenType::Equals => BinaryOp::Assign,
                TokenType::DoubleEquals => BinaryOp::IsEq,
                TokenType::LeftAngle => BinaryOp::IsLt,
                TokenType::RightAngle => BinaryOp::IsGt,
                TokenType::Comma => {
                    let mut repeating_comma_op = None;
                    if let Some(prev_op) = prev_op {
                        if prev_op == Op::Binary(BinaryOp::FormTuple)
                            || prev_op == Op::Binary(BinaryOp::AppendToTuple)
                        {
                            repeating_comma_op = Some(BinaryOp::AppendToTuple);
                        }
                    }
                    repeating_comma_op.unwrap_or(BinaryOp::FormTuple)
                }
                TokenType::Bracket(Bracket {
                    type_: BracketType::Round,
                    side: BracketSide::Opening,
                }) => BinaryOp::FunctionCall,
                _ => {
                    if terminate_on_unexpected_token {
                        return Ok((left, i));
                    }
                    return Err(ParserError {
                        tokens: tokens,
                        errmsg: "expression end or binary operator expected here".into(),
                        error_token_idx: i,
                    });
                }
            };
            let next_op_token_count: usize = if next_binary_op == BinaryOp::FunctionCall {
                0
            } else {
                1
            };
            let op = Op::Binary(next_binary_op);
            if let Some(prev_op) = outer_op {
                if op.precedence() < prev_op.precedence() || (op == prev_op && !op.is_rtl()) {
                    return Ok((left, i));
                }
            }
            prev_op = Some(op);
            let right: Expression;
            (right, i) = consume_expression(
                tokens,
                i + next_op_token_count,
                Some(op),
                terminate_on_unexpected_token,
            )?;
            result = Some(Expression::BinaryOperation {
                op: next_binary_op,
                left: Box::new(left),
                right: Box::new(right),
            });
        } else {
            if i >= tokens.len() || tokens[i].t == TokenType::ExprEnd {
                return Ok((Expression::Value(Rc::new(Value::Nothing)), i));
            }
            let next_unary_op = match tokens[i].t {
                TokenType::Minus => UnaryOp::Neg,
                TokenType::Bang => UnaryOp::Neg,
                TokenType::Return => UnaryOp::Return,
                _ => {
                    return Err(ParserError {
                        tokens: tokens,
                        errmsg: "operand or unary operator expected here".into(),
                        error_token_idx: i,
                    })
                }
            };
            let operand: Expression;
            (operand, i) = consume_expression(
                tokens,
                i + 1,
                Some(Op::Unary(next_unary_op)),
                terminate_on_unexpected_token,
            )?;
            result = Some(Expression::UnaryOperation {
                op: next_unary_op,
                operand: Box::new(operand),
            })
        }
    }
}

fn consume_operand<'a>(
    tokens: &'a [Token<'a>],
    i: usize,
) -> Result<(Option<Expression>, usize), ParserError<'a>> {
    let advance_if_type = |idx: usize, t: TokenType| {
        if idx < tokens.len() && tokens[idx].t == t {
            return idx + 1;
        } else {
            idx
        }
    };

    if i >= tokens.len() {
        return Ok((None, i));
    }
    let next = &tokens[i];
    match next.t {
        TokenType::ExprEnd => Ok((None, i)),
        TokenType::Number => {
            let includes_dot = next.lexeme.chars().find(|&ch| ch == '.').is_some();
            let value = if includes_dot {
                if let Ok(f) = next.lexeme.parse::<f32>() {
                    Value::Float(f)
                } else {
                    return Err(ParserError {
                        tokens: tokens,
                        errmsg: "not a valid floating point number".into(),
                        error_token_idx: i,
                    });
                }
            } else {
                if let Ok(i) = next.lexeme.parse::<i32>() {
                    Value::Int(i)
                } else {
                    return Err(ParserError {
                        tokens: tokens,
                        errmsg: "not a valid integer".into(),
                        error_token_idx: i,
                    });
                }
            };
            return Ok((Some(Expression::Value(Rc::new(value))), i + 1));
        }
        TokenType::StringLiteral => Ok((
            Some(Expression::Value(Rc::new(Value::String(
                next.lexeme[1..next.lexeme.len() - 1].into(),
            )))),
            i + 1,
        )),
        TokenType::BoolLiteral => Ok((
            Some(Expression::Value(Rc::new(Value::Bool(
                next.lexeme.to_lowercase() == "true",
            )))),
            i + 1,
        )),
        TokenType::Identifier => Ok((Some(Expression::Variable(next.lexeme.to_owned())), i + 1)),
        TokenType::Bracket(Bracket {
            type_: bracket_type,
            side: BracketSide::Opening,
        }) => {
            let mut bracket_stack = BracketStack::new();
            bracket_stack
                .update(Bracket {
                    type_: bracket_type,
                    side: BracketSide::Opening,
                })
                .unwrap();
            let mut j = i + 1;
            while j < tokens.len() && !bracket_stack.is_empty() {
                let tt = &tokens[j].t;
                if let TokenType::Bracket(b) = tt {
                    if let Err(update_errmsg) = bracket_stack.update(*b) {
                        return Err(ParserError {
                            tokens: tokens,
                            errmsg: update_errmsg,
                            error_token_idx: j,
                        });
                    }
                }
                j += 1;
            }
            if !bracket_stack.is_empty() {
                return Err(ParserError {
                    tokens: tokens,
                    errmsg: "unclosed bracket".into(),
                    error_token_idx: i,
                });
            }

            let bracketed_tokens = &tokens[i + 1..j - 1];
            if bracketed_tokens.len() == 0 {
                return Ok((Some(Expression::Value(Rc::new(Value::Nothing))), j));
            }

            let bracketed_expr = match bracket_type {
                BracketType::Round => {
                    let (expr, last_expr_token_offset_idx) =
                        consume_expression(bracketed_tokens, 0, None, false)?;
                    if last_expr_token_offset_idx < bracketed_tokens.len() - 1 {
                        return Err(ParserError {
                            tokens: bracketed_tokens,
                            errmsg: "round brackets must contain only one expression".into(),
                            error_token_idx: last_expr_token_offset_idx,
                        });
                    }
                    expr
                }
                BracketType::Curly => parse_scope(bracketed_tokens, false)?,
            };
            return Ok((Some(bracketed_expr), j));
        }
        t if t == TokenType::If || t == TokenType::While => {
            let mut j = i + 1;
            let condition: Expression;
            (condition, j) = consume_expression(tokens, j, None, true)?;
            if tokens[j].t == TokenType::ExprEnd {
                j += 1;
            }
            let body: Expression;
            (body, j) = consume_expression(tokens, j, None, true)?;

            let possible_else_idx = advance_if_type(j, TokenType::ExprEnd);
            let possible_else_body_start_idx = advance_if_type(possible_else_idx, TokenType::Else);
            let body_after_else = if possible_else_body_start_idx > possible_else_idx {
                let expr: Expression;
                (expr, j) = consume_expression(tokens, possible_else_body_start_idx, None, false)?;
                Some(Box::new(expr))
            } else {
                None
            };
            let res = if t == TokenType::If {
                Expression::If {
                    condition: Box::new(condition),
                    if_true: Box::new(body),
                    if_false: body_after_else,
                }
            } else {
                Expression::While {
                    condition: Box::new(condition),
                    body: Box::new(body),
                    if_completed: body_after_else,
                }
            };
            Ok((Some(res), j))
        }
        TokenType::Func => {
            let mut j = i + 1;
            let func_declaration_expr: Expression;
            (func_declaration_expr, j) = consume_expression(tokens, j, None, true)?;
            let (func_name, func_params) = if let Expression::BinaryOperation {
                op: BinaryOp::FunctionCall,
                left,
                right,
            } = func_declaration_expr
            {
                if let Expression::Variable(func_name) = left.clone().as_ref() {
                    (func_name.clone(), *right.clone())
                } else {
                    return Err(ParserError {
                        tokens: tokens,
                        errmsg: "functon name expected here".into(),
                        error_token_idx: i + 1,
                    });
                }
            } else {
                return Err(ParserError {
                    tokens,
                    errmsg: "function declaration expected here".into(),
                    error_token_idx: i + 1,
                });
            };

            j = advance_if_type(j, TokenType::ExprEnd);

            let mut func_body: Expression;
            (func_body, j) = consume_expression(tokens, j, None, false)?;
            func_body = match func_body {
                Expression::Scope {
                    body,
                    is_returnable: _,
                } => Expression::Scope {
                    body: body,
                    is_returnable: true,
                },
                other => other,
            };
            return Ok((
                Some(Expression::BinaryOperation {
                    op: BinaryOp::Assign,
                    left: Box::new(Expression::Variable(func_name.clone())),
                    right: Box::new(Expression::Value(Rc::new(Value::Function(
                        Function::UserDefined(UserDefinedFunction {
                            name: func_name,
                            params: func_params.clone(),
                            body: func_body,
                        }),
                    )))),
                }),
                j,
            ));
        }
        _ => Ok((None, i)),
    }
}

fn skip_comments(tokens: &[Token], i: usize) -> usize {
    let mut i = i;
    while i < tokens.len() && tokens[i].t == TokenType::Comment {
        i += 1
    }
    i
}
