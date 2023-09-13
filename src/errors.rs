use std::{error::Error, fmt::Display};

use crate::tokenizer::untokenize;
use crate::tokenizer::Token;
use crate::tokenizer::TokenType;

#[derive(Debug)]
pub struct TokenizerError<'a> {
    pub code: &'a str,
    pub errmsg: String,
    pub error_char_idx: usize,
}

impl Error for TokenizerError<'_> {}

impl Display for TokenizerError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &start_offset = &self.code[..self.error_char_idx]
            .chars()
            .rev()
            .enumerate()
            .find(|&(_, ch)| ch == '\n')
            .map(|(idx, _)| idx)
            .unwrap_or(self.error_char_idx);

        let &end_offset = &self.code[self.error_char_idx..]
            .chars()
            .enumerate()
            .find(|&(_, ch)| ch == '\n')
            .map(|(idx, _)| idx)
            .unwrap_or(self.code.len() - self.error_char_idx);

        let code_context_line =
            &self.code[self.error_char_idx - start_offset..self.error_char_idx + end_offset];

        let mut pointing_arrow_line = " ".repeat(start_offset);

        pointing_arrow_line.push_str("^");

        write!(
            f,
            "Tokenizer error\n> {}\n  {} {}",
            code_context_line, pointing_arrow_line, self.errmsg
        )
    }
}

#[cfg(test)]
mod tokenizer_error_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("abcdefg", 3, "Tokenizer error\n> abcdefg\n     ^ example error")]
    #[case("abcdefg", 0, "Tokenizer error\n> abcdefg\n  ^ example error")]
    #[case(
        "abcdefg\nsecond line ok\n third line",
        5,
        "Tokenizer error\n> abcdefg\n       ^ example error"
    )]
    #[case(
        "line 1\nline 2\nline 3\nline 4",
        15,
        "Tokenizer error\n> line 3\n   ^ example error"
    )]
    #[case(
        "line 1\nline 2\nline 3",
        15,
        "Tokenizer error\n> line 3\n   ^ example error"
    )]
    fn test_tokenizer_error_display(
        #[case] code: &str,
        #[case] error_char_idx: usize,
        #[case] expected_formatted_error: &str,
    ) {
        let e = TokenizerError {
            code: code,
            errmsg: "example error".into(),
            error_char_idx: error_char_idx,
        };
        assert_eq!(format!("{}", e), expected_formatted_error);
    }
}

#[derive(Debug)]
pub struct ParserError<'a> {
    pub tokens: &'a Vec<Token<'a>>,
    pub errmsg: String,
    pub error_token_idx: usize,
}

impl Error for ParserError<'_> {}

impl Display for ParserError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &start_offset = &self.tokens[..self.error_token_idx]
            .iter()
            .rev()
            .enumerate()
            .find(|&(_, tok)| tok.t == TokenType::ExprEnd)
            .map(|(idx, _)| idx)
            .unwrap_or(self.error_token_idx);

        let &end_offset = &self.tokens[self.error_token_idx..]
            .iter()
            .enumerate()
            .find(|&(_, tok)| tok.t == TokenType::ExprEnd)
            .map(|(idx, _)| idx)
            .unwrap_or(self.tokens.len() - self.error_token_idx);

        let code_context_tokens: Vec<Token<'_>> = self.tokens
            [self.error_token_idx - start_offset..self.error_token_idx + end_offset]
            .into();
        let code_context_line = untokenize(&code_context_tokens);

        let code_context_pre_err = untokenize(
            &self.tokens[self.error_token_idx - start_offset..=self.error_token_idx].into(),
        );
        let code_context_err = untokenize(&vec![self.tokens[self.error_token_idx].clone()]);
        let mut pointing_arrow_line =
            " ".repeat(code_context_pre_err.len() - code_context_err.len());

        pointing_arrow_line.push_str(&"^".repeat(code_context_err.len()));

        write!(
            f,
            "Parser error\n> {}\n  {} {}",
            code_context_line, pointing_arrow_line, self.errmsg
        )
    }
}
