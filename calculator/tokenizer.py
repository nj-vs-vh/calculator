import enum
import re
from dataclasses import dataclass

from calculator.utils import PrintableEnum


@dataclass
class TokenizerError(Exception):
    errmsg: str
    error_char_idx: int

    def format(self, code: str) -> str:
        print_start_idx = max(0, self.error_char_idx - 10)
        print_ellipsis_pre = print_start_idx > 0
        print_end_idx = min(len(code), self.error_char_idx + 10)
        print_ellipsis_post = print_end_idx < len(code)
        return "\n".join(
            [
                f"[Tokenizer error] {self.errmsg}",
                (
                    ("..." if print_ellipsis_pre else "")
                    + f"{code[print_start_idx:print_end_idx]}"
                    + ("..." if print_ellipsis_post else "")
                ),
                " " * (self.error_char_idx - print_start_idx + (3 if print_ellipsis_pre else 0)) + "^",
            ]
        )


class TokenType(PrintableEnum):
    NUMBER = enum.auto()
    PLUS = enum.auto()
    MINUS = enum.auto()
    STAR = enum.auto()
    SLASH = enum.auto()
    BRACKET_OPEN = enum.auto()
    BRACKET_CLOSE = enum.auto()
    EXPR_END = enum.auto()
    CARET = enum.auto()
    EQUAL = enum.auto()
    IDENTIFIER = enum.auto()


@dataclass
class Token:
    type: TokenType
    lexeme: str

    def __str__(self) -> str:
        return f"<{self.type}>{self.lexeme}"


def _is_valid_in_number(s: str) -> bool:
    return s.isdigit() or s == "."


def _is_valid_in_identifier(s: str) -> bool:
    return s.isalnum()


SINGLE_CHAR_TOKENS = {
    "+": TokenType.PLUS,
    "-": TokenType.MINUS,
    "*": TokenType.STAR,
    "/": TokenType.SLASH,
    "(": TokenType.BRACKET_OPEN,
    ")": TokenType.BRACKET_CLOSE,
    ";": TokenType.EXPR_END,
    "\n": TokenType.EXPR_END,
    "=": TokenType.EQUAL,
    "^": TokenType.CARET,
}


def tokenize(s: str) -> list[Token]:
    i = 0
    tokens: list[Token] = []
    while i < len(s):
        if _is_valid_in_number(s[i]):
            number_end_idx = i + 1
            while number_end_idx < len(s) and _is_valid_in_number(s[number_end_idx]):
                number_end_idx += 1
            tokens.append(Token(type=TokenType.NUMBER, lexeme=s[i:number_end_idx]))
            i = number_end_idx - 1  # to account for += 1 later
        elif s[i].isalpha():
            ident_end_idx = i + 1
            while ident_end_idx < len(s) and s[ident_end_idx].isalnum():
                ident_end_idx += 1
            tokens.append(Token(type=TokenType.IDENTIFIER, lexeme=s[i:ident_end_idx]))
            i = ident_end_idx - 1  # to account for += 1 later
        elif s[i] in SINGLE_CHAR_TOKENS:
            tokens.append(Token(type=SINGLE_CHAR_TOKENS[s[i]], lexeme=s[i]))
        elif s[i].isspace():
            pass
        else:
            raise TokenizerError(f"Unexpected character: {s[i]!r}", error_char_idx=i)
        i += 1

    if tokens and tokens[-1].type is not TokenType.EXPR_END:
        tokens.append(Token(type=TokenType.EXPR_END, lexeme=""))

    return tokens


def untokenize(tokens: list[Token]) -> str:
    result = " ".join(t.lexeme for t in tokens)

    result = re.sub(r"\s+;", ";", result)

    # ( 1 + 2 ) => (1 + 2)
    result = re.sub(r"\(\s+", "(", result)
    result = re.sub(r"\s+\)", ")", result)

    # 4 ^ 5 => 4^5
    result = re.sub(r"\s+^\s+", "^", result)
    return result
