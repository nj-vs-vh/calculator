use crate::{
    errors::ParserError,
    tokenizer::{Token, TokenType},
};

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
    op: BinaryOp,
    left: Box<Expression>,
    right: Box<Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
}

#[derive(Debug, Clone)]
pub struct UnaryOperation {
    op: UnaryOp,
    operand: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(f32), // value modelling for data types TBD
    Variable(String),
    Bin(BinaryOperation),
    Un(UnaryOperation),
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

pub fn parse<'a>(tokens: &'a [Token<'a>]) -> Result<Vec<Expression>, ParserError<'a>> {
    let mut result: Vec<Expression> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let expr: Expression;
        (expr, i) = consume_expression(tokens, i, None, false)?;
        i += 1;
        result.push(expr);
    }
    return Ok(result);
}

fn consume_expression<'a>(
    tokens: &'a [Token<'a>],
    i: usize,
    prev_op: Option<Op>,
    allow_unterminated: bool,
) -> Result<(Expression, usize), ParserError<'a>> {
    let mut result: Option<Expression> = None;
    let mut left: Option<Expression>;
    let mut i = i;
    loop {
        (left, i) = if *&result.is_none() {
            consume_operand(tokens, i)?
        } else {
            (result.clone(), i)
        };

        if i >= tokens.len() {
            if allow_unterminated {
                if let Some(left) = left {
                    return Ok((left, tokens.len()));
                }
            }
            return Err(ParserError {
                tokens: tokens,
                errmsg: "unterminated expression".into(),
                error_token_idx: tokens.len() - 1,
            });
        }

        if let Some(left) = left {
            if tokens[i].t == TokenType::ExprEnd {
                return Ok((left, i));
            }
            let next_binary_op = match tokens[i].t {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Sub,
                TokenType::Star => BinaryOp::Mul,
                TokenType::Slash => BinaryOp::Div,
                TokenType::Caret => BinaryOp::Pow,
                TokenType::Equals => BinaryOp::Assign,
                TokenType::RoundBracketOpen => BinaryOp::FunctionCall,
                _ => {
                    return Err(ParserError {
                        tokens: tokens,
                        errmsg: "binary operator expected here".into(),
                        error_token_idx: i,
                    })
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
            (right, i) = consume_expression(
                tokens,
                i + next_op_token_count,
                Some(next_op),
                allow_unterminated,
            )?;
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
                        errmsg: "unary operator expected".into(),
                        error_token_idx: i,
                    })
                }
            };
            let operand: Expression;
            (operand, i) = consume_expression(
                tokens,
                i,
                Some(Op::Unary(next_unary_op)),
                allow_unterminated,
            )?;
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
    let first = &tokens[i];
    if first.t == TokenType::Number {
        return match first.lexeme.parse::<f32>() {
            Ok(f) => Ok((Some(Expression::Value(f)), i + 1)),
            Err(_) => Err(ParserError {
                tokens: tokens,
                errmsg: "not a valid floating point number".into(),
                error_token_idx: i,
            }),
        };
    } else if first.t == TokenType::Identifier {
        return Ok((Some(Expression::Variable(first.lexeme.to_owned())), i + 1));
    } else if first.t == TokenType::RoundBracketOpen {
        let mut open_bracket_count = 1;
        let mut j = i + 1;
        while j < tokens.len() && open_bracket_count > 0 {
            let tt = &tokens[j].t;
            if *tt == TokenType::ExprEnd {
                return Err(ParserError {
                    tokens: tokens,
                    errmsg: "expression terminated inside brackets".into(),
                    error_token_idx: j,
                });
            }
            if *tt == TokenType::RoundBracketOpen {
                open_bracket_count += 1;
            } else if *tt == TokenType::RoundBracketClose {
                open_bracket_count -= 1;
            }
            j += 1;
        }
        let bracketed_tokens = &tokens[i + 1..j - 1];
        if open_bracket_count > 0 {
            return Err(ParserError {
                tokens: tokens,
                errmsg: "unclosed bracket".into(),
                error_token_idx: i,
            });
        }
        if bracketed_tokens.len() == 0 {
            return Err(ParserError {
                tokens: tokens,
                errmsg: "empty brackets".into(),
                error_token_idx: i,
            });
        }
        let (bracketed_expr, _) = consume_expression(bracketed_tokens, 0, None, true)?;
        return Ok((Some(bracketed_expr), j));
    } else {
        return Ok((None, i));
    }
}
