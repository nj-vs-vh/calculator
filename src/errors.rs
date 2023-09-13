use std::{cmp::min, error::Error, fmt::Display};

use itertools::Itertools;

use crate::tokenizer::Token;

#[derive(Debug)]
pub struct TokenizerError<'a> {
    pub code: &'a str,
    pub errmsg: String,
    pub error_char_idx: usize,
}

impl Error for TokenizerError<'_> {}

const DISPLAYED_CODE_CONTEXT_CHARS: usize = 30;

impl Display for TokenizerError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code_chars: Vec<char> = self.code.chars().collect();

        let mut start_idx = self
            .error_char_idx
            .saturating_sub(DISPLAYED_CODE_CONTEXT_CHARS);
        let mut is_ellipsis_prefix = start_idx > 0;
        if let Some((rel_newline_idx, _)) = code_chars[start_idx..self.error_char_idx]
            .iter()
            .enumerate()
            .find_or_first(|&(_, &ch)| ch == '\n')
        {
            start_idx += rel_newline_idx + 1;
            is_ellipsis_prefix = false;
        }

        let end_idx = min(
            self.error_char_idx + DISPLAYED_CODE_CONTEXT_CHARS,
            code_chars.len(),
        );

        let mut code_context_line = if is_ellipsis_prefix {
            String::from("...")
        } else {
            String::new()
        };

        let code_context = code_chars[start_idx..end_idx].iter().collect::<String>();
        code_context_line.push_str(&code_context);
        if end_idx < code_chars.len() {
            code_context_line.push_str("...")
        }

        let mut pointing_arrow_line =
            " ".repeat(self.error_char_idx - start_idx - (if is_ellipsis_prefix { 3 } else { 0 }));

        pointing_arrow_line.push_str("^");

        write!(
            f,
            "Tokenizer error\n> {}\n  {} {}",
            code_context_line, pointing_arrow_line, self.errmsg
        )
    }
}

#[derive(Debug)]
pub struct ParserError<'a> {
    tokens: &'a Vec<Token<'a>>,
    errmsg: String,
    error_token_idx: usize,
}

// impl Error for ParserError<'_> {}

// const DISPLAYED_CODE_CONTEXT_TOKENS: usize = 10;

// impl Display for ParserError<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "Parser error\n> {}\n  {} {}",
//             code_context_line, pointing_arrow_line, self.errmsg
//         )
//     }
// }
