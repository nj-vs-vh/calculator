use crate::{
    bracket::{Bracket, BracketSide, BracketType},
    errors::TokenizerError,
};

use super::errors;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Number,
    Plus,
    Minus,
    Star,
    Slash,
    Bracket(Bracket),
    ExprEnd,
    Caret,
    Equals,
    Identifier,
    StringLiteral,
    BoolLiteral,
    If,
    Else,
    LeftAngle,
    RightAngle,
    DoubleEquals,
    Return,
    Bang,
    While,
    Func,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Token<'a> {
    pub t: TokenType,
    pub lexeme: &'a str,
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" ({:?})", self.lexeme, self.t)
    }
}

pub fn tokenize<'a>(code: &'a str) -> Result<Vec<Token<'a>>, errors::TokenizerError> {
    let mut tokens = Vec::new();

    if code.len() == 0 {
        return Ok(tokens);
    }

    let mut code_chars = code.char_indices();
    let mut current_char: Option<char> = None;

    while let Some((lookahead_idx, lookahead_char)) = code_chars.next() {
        if !lookahead_char.is_ascii() {
            return Err(errors::TokenizerError {
                code: code,
                errmsg: "non-ASCII character".into(),
                error_char_idx: lookahead_idx,
            });
        }

        // matching singe-char tokens, possibly left over from prev iteration / long token matching
        if let Some(current_char) = current_char {
            match match_char(current_char) {
                CharMatch::Token(token_type) => tokens.push(Token {
                    t: token_type,
                    lexeme: &code[lookahead_idx - 1..lookahead_idx],
                }),
                CharMatch::Whitespace => {}
                // CharMatch::CommentStart =>
                CharMatch::Unexpected => {
                    return Err(errors::TokenizerError {
                        code: &code,
                        errmsg: String::from("unexpected character"),
                        error_char_idx: lookahead_idx - 1,
                    })
                }
            };
        }

        // lookahead matching of "long" tokens with subiteration
        let maybe_long_token = match lookahead_char {
            numeric if is_numeric_char(numeric) => {
                let end_idx: usize;
                (end_idx, current_char) = iter_while_predicate(&mut code_chars, is_numeric_char)
                    .unwrap_or((code.len(), None));
                Some(Token {
                    t: TokenType::Number,
                    lexeme: &code[lookahead_idx..end_idx],
                })
            }
            letter if letter.is_ascii_alphabetic() => {
                let end_idx: usize;
                (end_idx, current_char) = iter_while_predicate(&mut code_chars, |ch| {
                    ch.is_ascii_alphanumeric() || ch == '_'
                })
                .unwrap_or((code.len(), None));
                let lexeme = &code[lookahead_idx..end_idx];
                if let Some(keyword) = match_keyword(lexeme) {
                    Some(Token { t: keyword, lexeme })
                } else {
                    Some(Token {
                        t: TokenType::Identifier,
                        lexeme,
                    })
                }
            }
            '=' => {
                let end_idx: usize;
                (end_idx, current_char) = iter_while_predicate(&mut code_chars, |ch| ch == '=')
                    .unwrap_or((code.len(), None));
                let lexeme = &code[lookahead_idx..end_idx];
                let token_type = match lexeme.len() {
                    1 => TokenType::Equals,
                    2 => TokenType::DoubleEquals,
                    _ => {
                        return Err(TokenizerError {
                            code: code,
                            errmsg: "too much equal signs".into(),
                            error_char_idx: end_idx - 1,
                        })
                    }
                };
                Some(Token {
                    t: token_type,
                    lexeme,
                })
            }
            '"' => {
                let (end_idx, _) = iter_while_predicate(&mut code_chars, |ch| ch != '"').ok_or(
                    TokenizerError {
                        code: &code,
                        errmsg: "unterminated string literal".into(),
                        error_char_idx: code.len() - 1,
                    },
                )?;
                // code_chars.next(); // consuming closing quote
                current_char = None;
                Some(Token {
                    t: TokenType::StringLiteral,
                    lexeme: &code[lookahead_idx..=end_idx],
                })
            }
            _ => None,
        };

        match maybe_long_token {
            None => {
                current_char = Some(lookahead_char);
            }
            Some(token) => {
                tokens.push(token);
            }
        }
    }

    // matching the last leftover character, if exists
    if let Some(last_char) = current_char {
        match match_char(last_char) {
            CharMatch::Token(tt) => tokens.push(Token {
                t: tt,
                lexeme: &code[code.len() - 1..code.len()],
            }),
            CharMatch::Whitespace => {}
            CharMatch::Unexpected => {
                return Err(errors::TokenizerError {
                    code: &code,
                    errmsg: String::from("unexpected character"),
                    error_char_idx: code.len() - 1,
                })
            }
        };
    }

    // inserting an implied expression end token, if not present
    if tokens[tokens.len() - 1].t != TokenType::ExprEnd {
        tokens.push(Token {
            t: TokenType::ExprEnd,
            lexeme: ";",
        })
    }

    return Ok(tokens);
}

fn iter_while_predicate<Predicate>(
    it: &mut impl Iterator<Item = (usize, char)>,
    predicate: Predicate,
) -> Option<(usize, Option<char>)>
where
    Predicate: Fn(char) -> bool,
{
    while let Some((idx, ch)) = it.next() {
        if !predicate(ch) {
            return Some((idx, Some(ch)));
        }
    }
    return None;
}

fn is_numeric_char(ch: char) -> bool {
    ch.is_ascii_digit() || ch == '.'
}

enum CharMatch {
    Token(TokenType),
    Whitespace,
    // CommentStart,
    Unexpected,
}

fn match_char(ch: char) -> CharMatch {
    match ch {
        '+' => CharMatch::Token(TokenType::Plus),
        '-' => CharMatch::Token(TokenType::Minus),
        '!' => CharMatch::Token(TokenType::Bang),
        '*' => CharMatch::Token(TokenType::Star),
        '/' => CharMatch::Token(TokenType::Slash),
        '(' => CharMatch::Token(TokenType::Bracket(Bracket {
            type_: BracketType::Round,
            side: BracketSide::Open,
        })),
        ')' => CharMatch::Token(TokenType::Bracket(Bracket {
            type_: BracketType::Round,
            side: BracketSide::Close,
        })),
        ';' => CharMatch::Token(TokenType::ExprEnd),
        '=' => CharMatch::Token(TokenType::Equals),
        '^' => CharMatch::Token(TokenType::Caret),
        '<' => CharMatch::Token(TokenType::LeftAngle),
        '>' => CharMatch::Token(TokenType::RightAngle),
        '{' => CharMatch::Token(TokenType::Bracket(Bracket {
            type_: BracketType::Curly,
            side: BracketSide::Open,
        })),
        '}' => CharMatch::Token(TokenType::Bracket(Bracket {
            type_: BracketType::Curly,
            side: BracketSide::Close,
        })),
        // '#' => CharMatch::CommentStart,
        ws if ws.is_whitespace() => CharMatch::Whitespace,
        _ => CharMatch::Unexpected,
    }
}

fn match_keyword(lexeme: &str) -> Option<TokenType> {
    match lexeme {
        "if" => Some(TokenType::If),
        "else" => Some(TokenType::Else),
        any_true if any_true.to_lowercase() == "true" => Some(TokenType::BoolLiteral),
        any_false if any_false.to_lowercase() == "false" => Some(TokenType::BoolLiteral),
        "return" => Some(TokenType::Return),
        "while" => Some(TokenType::While),
        "func" => Some(TokenType::Func),
        _ => None,
    }
}

pub fn untokenize(tokens: &[Token], minified: bool) -> String {
    let mut res = String::new();

    let token_iter_1 = tokens.iter();
    let mut token_iter_2 = tokens.iter();
    token_iter_2.next();

    let mut current_indent: usize = 0;
    let indent_spaces: usize = if minified { 0 } else { 2 };

    let newline = if minified { " " } else { "\n" };

    for (token_l, token_r) in token_iter_1.zip(token_iter_2) {
        res.push_str(&format_token(token_l));
        let delimiter = match (token_l.t, token_r.t) {
            (
                TokenType::Bracket(Bracket {
                    type_: BracketType::Curly,
                    side: BracketSide::Open,
                }),
                _,
            ) => {
                current_indent += 1;
                newline.into()
            }
            (
                _,
                TokenType::Bracket(Bracket {
                    type_: BracketType::Curly,
                    side: BracketSide::Close,
                }),
            ) => {
                current_indent = current_indent.saturating_sub(1);
                newline.into()
            }
            (TokenType::Caret, _) => "",
            (_, TokenType::Caret) => "",
            (
                // function calls like log(10)
                TokenType::Identifier,
                TokenType::Bracket(Bracket {
                    type_: BracketType::Round,
                    side: BracketSide::Open,
                }),
            ) => "",
            (_, TokenType::ExprEnd) => "",
            (TokenType::ExprEnd, _) => newline.into(),

            (
                TokenType::Bracket(Bracket {
                    type_: _,
                    side: BracketSide::Open,
                }),
                _,
            ) => "",
            (
                _,
                TokenType::Bracket(Bracket {
                    type_: _,
                    side: BracketSide::Close,
                }),
            ) => "",
            _ => " ",
        };
        res.push_str(delimiter);
        if delimiter.ends_with("\n") {
            res.push_str(&" ".repeat(current_indent * indent_spaces))
        }
    }
    res.push_str(&format_token(&tokens[tokens.len() - 1]));
    return res;
}

fn format_token(token: &Token) -> String {
    match token.t {
        TokenType::BoolLiteral => token.lexeme.to_lowercase(),
        _ => token.lexeme.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("1", vec![Token{t: TokenType::Number, lexeme: "1"}, Token{t: TokenType::ExprEnd, lexeme: ";"}])]
    #[case("  1     ", vec![Token{t: TokenType::Number, lexeme: "1"}, Token{t: TokenType::ExprEnd, lexeme: ";"}])]
    #[case("1 1", vec![Token{t: TokenType::Number, lexeme: "1"}, Token{t: TokenType::Number, lexeme: "1"}, Token{t: TokenType::ExprEnd, lexeme: ";"}])]
    #[case("1 + 1", vec![
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::Plus, lexeme: "+"},
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::ExprEnd, lexeme: ";"}
    ])]
    #[case("1+1", vec![
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::Plus, lexeme: "+"},
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::ExprEnd, lexeme: ";"}
    ])]
    #[case("1  + 1", vec![
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::Plus, lexeme: "+"},
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::ExprEnd, lexeme: ";"}
    ])]
    #[case("1 +1", vec![
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::Plus, lexeme: "+"},
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::ExprEnd, lexeme: ";"}
    ])]
    #[case("1+ 1", vec![
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::Plus, lexeme: "+"},
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::ExprEnd, lexeme: ";"}
    ])]
    #[case("   1      + \n  1  ", vec![
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::Plus, lexeme: "+"},
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::ExprEnd, lexeme: ";"}
    ])]
    #[case("a", vec![Token{t: TokenType::Identifier, lexeme: "a"}, Token{t: TokenType::ExprEnd, lexeme: ";"}])]
    #[case("a^b", vec![
        Token{t: TokenType::Identifier, lexeme: "a"},
        Token{t: TokenType::Caret, lexeme: "^"},
        Token{t: TokenType::Identifier, lexeme: "b"},
        Token{t: TokenType::ExprEnd, lexeme: ";"},
    ])]
    #[case("1  /  abc123def            ", vec![
        Token{t: TokenType::Number, lexeme: "1"},
        Token{t: TokenType::Slash, lexeme: "/"},
        Token{t: TokenType::Identifier, lexeme: "abc123def"},
        Token{t: TokenType::ExprEnd, lexeme: ";"},
    ])]
    fn test_tokenizer(#[case] code: &str, #[case] expected_result: Vec<Token>) {
        let code_ = String::from(code);
        let tokens = tokenize(&code_).unwrap();
        assert_eq!(tokens, expected_result);
    }
}
