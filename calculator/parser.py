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
        filler_whitespace = untokenize(
            [
                Token(
                    type=TokenType.EXPR_END,
                    lexeme=" " * len(t.lexeme),
                )
                for t in parsed_tokens
            ]
        )
        return "\n".join([f"Parser error: {self.errmsg}", untokenize(self.tokens), filler_whitespace + "^"])


class BinaryOperator(PrintableEnum):
    ADD = enum.auto()
    SUB = enum.auto()
    MUL = enum.auto()
    DIV = enum.auto()


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


Expression = float | BinaryOperation | UnaryOperation


def parse(tokens: list[Token]) -> list[Expression]:
    result: list[Expression] = []
    i = 0
    while i < len(tokens):
        expr, i = _consume_expression(tokens, i, min_op_precedence=None)
        if i >= len(tokens) or tokens[i].type is not TokenType.EXPR_END:
            raise ParserError("Internal error", tokens=tokens, error_token_idx=i)
        i += 1  # skipping expr end
        result.append(expr)
    return result


def get_op_precedence(op: BinaryOperator | UnaryOperator) -> int:
    return [
        BinaryOperator.ADD,
        BinaryOperator.SUB,
        BinaryOperator.MUL,
        BinaryOperator.DIV,
        UnaryOperator.NEG,
        UnaryOperator.POS,
    ].index(op)


def _consume_expression(tokens: list[Token], i: int, min_op_precedence: Optional[int]) -> tuple[Expression, int]:
    result: Expression | None = None
    should_exit = False
    while not should_exit:
        if result is not None:
            left: Expression | None = result
        else:
            left, i = _consume_operand(tokens, i)

        if i >= len(tokens):
            raise ParserError("Unterminated expression", tokens=tokens, error_token_idx=len(tokens))

        if left is not None:
            if tokens[i].type is TokenType.EXPR_END:
                result = left
                should_exit = True
            else:
                operator_token = tokens[i]
                operator = {
                    TokenType.PLUS: BinaryOperator.ADD,
                    TokenType.MINUS: BinaryOperator.SUB,
                    TokenType.STAR: BinaryOperator.MUL,
                    TokenType.SLASH: BinaryOperator.DIV,
                }.get(operator_token.type)
                if operator is None:
                    raise ParserError(
                        f"Binary operator expected, found {operator_token.type}",
                        tokens=tokens,
                        error_token_idx=i,
                    )
                if min_op_precedence is not None and get_op_precedence(operator) <= min_op_precedence:
                    result = left
                    should_exit = True
                else:
                    right, i = _consume_expression(
                        tokens,
                        i + 1,
                        min_op_precedence=get_op_precedence(operator),
                    )
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
            operand, i = _consume_expression(
                tokens,
                i + 1,
                min_op_precedence=get_op_precedence(unary_operator),
            )
            result = UnaryOperation(operator=unary_operator, operand=operand)

    if result is None:
        raise ParserError("Internal error, no expression parsed", tokens=tokens, error_token_idx=i)

    return result, i


def _consume_operand(tokens: list[Token], i: int) -> tuple[Optional[Expression], int]:
    """Mutates passed tokens list"""
    if i >= len(tokens):
        return None, i
    first = tokens[i]
    if first.type == TokenType.NUMBER:
        return float(first.lexeme), i + 1
    elif first.type == TokenType.BRACKET_OPEN:
        bracket_count = 1
        j = i + 1
        while j < len(tokens) and bracket_count > 0:
            if tokens[j].type == TokenType.BRACKET_OPEN:
                bracket_count += 1
            elif tokens[j].type == TokenType.BRACKET_CLOSE:
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
