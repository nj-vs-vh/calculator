use crate::{
    bracket::{Bracket, BracketSide, BracketStack, BracketType},
    errors::ParserError,
    tokenizer::{Token, TokenType},
    values::Value,
};
use std::cmp::min;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Assign,
    FunctionCall,
}

#[derive(Debug, Clone)]
pub struct BinaryOperation {
    pub op: BinaryOp,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
}

#[derive(Debug, Clone)]
pub struct UnaryOperation {
    pub op: UnaryOp,
    pub operand: Box<Expression>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Op {
    Unary(UnaryOp),
    Binary(BinaryOp),
}

const ORDER_OF_PRECEDENCE: [Op; 8] = [
    Op::Binary(BinaryOp::Assign),
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

impl PartialOrd for Op {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.precedence().partial_cmp(&other.precedence())
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(Box<Value>),
    Variable(String),
    Bin(BinaryOperation),
    Un(UnaryOperation),
    Scope(Vec<Expression>),
}

pub fn parse<'a>(tokens: &'a [Token<'a>]) -> Result<Expression, ParserError<'a>> {
    let mut exprs: Vec<Expression> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let expr: Expression;
        (expr, i) = consume_expression(tokens, i, None)?;
        i += 1; // skipping expression end
        exprs.push(expr);
    }
    return Ok(Expression::Scope(exprs));
}

fn consume_expression<'a>(
    tokens: &'a [Token<'a>],
    i: usize,
    prev_op: Option<Op>,
) -> Result<(Expression, usize), ParserError<'a>> {
    let mut result: Option<Expression> = None;
    let mut left: Option<Expression>;
    let mut i = i;
    loop {
        (left, i) = if *&result.is_none() {
            consume_operand(tokens, i)?
        } else {
            (result, i)
        };

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
                TokenType::Bracket(Bracket {
                    type_: BracketType::Round,
                    side: BracketSide::Open,
                }) => BinaryOp::FunctionCall,
                _ => {
                    return Err(ParserError {
                        tokens: tokens,
                        errmsg: "binary operator expected".into(),
                        error_token_idx: i,
                    });
                }
            };
            let next_op_token_count: usize = if next_binary_op == BinaryOp::FunctionCall {
                0
            } else {
                1
            };
            let next_op = Op::Binary(next_binary_op);
            if let Some(prev_op) = prev_op {
                if next_op < prev_op || (next_op == prev_op && !next_op.is_rtl()) {
                    return Ok((left, i));
                }
            }
            let right: Expression;
            (right, i) = consume_expression(tokens, i + next_op_token_count, Some(next_op))?;
            result = Some(Expression::Bin(BinaryOperation {
                op: next_binary_op,
                left: Box::new(left),
                right: Box::new(right),
            }));
        } else {
            let next_unary_op = match tokens[i].t {
                TokenType::Minus => UnaryOp::Neg,
                _ => {
                    return Err(ParserError {
                        tokens: tokens,
                        errmsg: "expected unary operator or an operand here".into(),
                        error_token_idx: i,
                    })
                }
            };
            let operand: Expression;
            (operand, i) = consume_expression(tokens, i + 1, Some(Op::Unary(next_unary_op)))?;
            result = Some(Expression::Un(UnaryOperation {
                op: next_unary_op,
                operand: Box::new(operand),
            }))
        }
    }
}

fn consume_operand<'a>(
    tokens: &'a [Token<'a>],
    i: usize,
) -> Result<(Option<Expression>, usize), ParserError<'a>> {
    if i > tokens.len() {
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
            return Ok((Some(Expression::Value(Box::new(value))), i + 1));
        }
        TokenType::StringLiteral => Ok((
            Some(Expression::Value(Box::new(Value::String(
                next.lexeme[1..next.lexeme.len() - 1].into(),
            )))),
            i + 1,
        )),
        TokenType::BoolLiteral => Ok((
            Some(Expression::Value(Box::new(Value::Bool(
                next.lexeme.to_lowercase() == "true",
            )))),
            i + 1,
        )),
        TokenType::Identifier => Ok((Some(Expression::Variable(next.lexeme.to_owned())), i + 1)),
        TokenType::Bracket(Bracket {
            type_: bracket_type,
            side: BracketSide::Open,
        }) => {
            let mut bracket_stack = BracketStack::new();
            bracket_stack
                .update(Bracket {
                    type_: bracket_type,
                    side: BracketSide::Open,
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
                return Err(ParserError {
                    tokens: tokens,
                    errmsg: "empty brackets".into(),
                    error_token_idx: i,
                });
            }

            let bracketed_expr = match bracket_type {
                BracketType::Round => {
                    let (expr, last_expr_token_offset_idx) =
                        consume_expression(bracketed_tokens, 0, None)?;
                    if last_expr_token_offset_idx < bracketed_tokens.len() - 1 {
                        return Err(ParserError {
                            tokens: bracketed_tokens,
                            errmsg: "round brackets must contain only one expression".into(),
                            error_token_idx: last_expr_token_offset_idx,
                        });
                    }
                    expr
                }
                BracketType::Curly => parse(bracketed_tokens)?,
            };
            return Ok((Some(bracketed_expr), j));
        }
        _ => Ok((None, i)),
    }
}
