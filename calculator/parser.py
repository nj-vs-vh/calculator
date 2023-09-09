import enum
from dataclasses import dataclass
from typing import Optional

from calculator.tokenizer import Token, TokenType, untokenize
from calculator.utils import PrintableEnum


@dataclass
class ParserError(Exception):
    errmsg: str
    tokens: list[Token]
    error_token_idx: int

    def __str__(self) -> str:
        parsed_tokens = self.tokens[: self.error_token_idx]
        filler_whitespace = " " * len(untokenize(parsed_tokens)) + " "
        return "\n".join([f"Parser error: {self.errmsg}", untokenize(self.tokens), filler_whitespace + "^"])


class BinaryOperator(PrintableEnum):
    ADD = enum.auto()
    SUB = enum.auto()
    MUL = enum.auto()
    DIV = enum.auto()
    POW = enum.auto()
    ASSIGN = enum.auto()


@dataclass
class BinaryOperation:
    operator: BinaryOperator
    left: "Expression"
    right: "Expression"


class UnaryOperator(PrintableEnum):
    NEG = enum.auto()
    POS = enum.auto()


@dataclass
class UnaryOperation:
    operator: UnaryOperator
    operand: "Expression"


@dataclass
class Variable:
    name: str


Operator = BinaryOperator | UnaryOperator
Expression = float | Variable | BinaryOperation | UnaryOperation


def parse(tokens: list[Token]) -> list[Expression]:
    result: list[Expression] = []
    i = 0
    while i < len(tokens):
        expr, i = _consume_expression(tokens, i, prev_operator=None)
        if i >= len(tokens) or tokens[i].type is not TokenType.EXPR_END:
            raise ParserError("Internal error", tokens=tokens, error_token_idx=i)
        i += 1  # skipping expr end
        result.append(expr)
    return result


def get_op_precedence(op: Operator) -> int:
    return [
        BinaryOperator.ASSIGN,
        BinaryOperator.ADD,
        BinaryOperator.SUB,
        BinaryOperator.MUL,
        BinaryOperator.DIV,
        UnaryOperator.NEG,
        UnaryOperator.POS,
        BinaryOperator.POW,
    ].index(op)


def is_rtl_op(op: Operator) -> bool:
    return op is BinaryOperator.ASSIGN


def _consume_expression(tokens: list[Token], i: int, prev_operator: Optional[Operator]) -> tuple[Expression, int]:
    result: Expression | None = None
    while True:
        if result is not None:
            left: Expression | None = result
        else:
            left, i = _consume_operand(tokens, i)

        if i >= len(tokens):
            raise ParserError("Unterminated expression", tokens=tokens, error_token_idx=len(tokens))

        if left is not None:
            if tokens[i].type is TokenType.EXPR_END:
                result = left
                break
            else:
                operator_token = tokens[i]
                operator = {
                    TokenType.PLUS: BinaryOperator.ADD,
                    TokenType.MINUS: BinaryOperator.SUB,
                    TokenType.STAR: BinaryOperator.MUL,
                    TokenType.SLASH: BinaryOperator.DIV,
                    TokenType.CARET: BinaryOperator.POW,
                    TokenType.EQUAL: BinaryOperator.ASSIGN,
                }.get(operator_token.type)
                if operator is None:
                    raise ParserError(
                        f"Binary operator expected, found {operator_token.type}",
                        tokens=tokens,
                        error_token_idx=i,
                    )
                if prev_operator is not None:
                    curr_precedence = get_op_precedence(operator)
                    prev_precedence = get_op_precedence(prev_operator)
                    if curr_precedence < prev_precedence or (
                        curr_precedence == prev_precedence and not is_rtl_op(operator)
                    ):
                        result = left
                        break

                right, i = _consume_expression(tokens, i + 1, prev_operator=operator)
                if right is None:
                    raise ParserError(f"Right operand expected", tokens=tokens, error_token_idx=i)
                result = BinaryOperation(operator=operator, left=left, right=right)
        else:
            unary_operator_token = tokens[i]
            unary_operator = {
                TokenType.MINUS: UnaryOperator.NEG,
                TokenType.PLUS: UnaryOperator.POS,
            }.get(unary_operator_token.type)
            if unary_operator is None:
                raise ParserError(
                    f"Unary operator expected, found {unary_operator_token.type}", tokens=tokens, error_token_idx=i
                )
            operand, i = _consume_expression(tokens, i + 1, prev_operator=unary_operator)
            result = UnaryOperation(operator=unary_operator, operand=operand)

    if result is None:
        raise ParserError("Internal error, no expression parsed", tokens=tokens, error_token_idx=i)

    return result, i


def _consume_operand(tokens: list[Token], i: int) -> tuple[Optional[Expression], int]:
    """Mutates passed tokens list"""
    if i >= len(tokens):
        return None, i
    first = tokens[i]
    if first.type is TokenType.NUMBER:
        return float(first.lexeme), i + 1
    elif first.type is TokenType.IDENTIFIER:
        return Variable(first.lexeme), i + 1
    elif first.type is TokenType.BRACKET_OPEN:
        bracket_count = 1
        j = i + 1
        while j < len(tokens) and bracket_count > 0:
            if tokens[j].type is TokenType.BRACKET_OPEN:
                bracket_count += 1
            elif tokens[j].type is TokenType.BRACKET_CLOSE:
                bracket_count -= 1
            j += 1
        bracketed_tokens = tokens[i + 1 : j - 1]
        if bracket_count:
            raise ParserError(f"Unclosed bracket", tokens, error_token_idx=i + 1)
        if not bracketed_tokens:
            raise ParserError(f"Empty parenthesis", tokens=tokens, error_token_idx=i + 1)
        return parse(bracketed_tokens + [Token(type=TokenType.EXPR_END, lexeme="")])[0], j
    else:
        return None, i
