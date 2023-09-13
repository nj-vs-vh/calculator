use std::{cmp::min, error::Error, fmt::Display};

#[derive(Debug)]
pub struct TokenizerError<'a> {
    pub code: &'a str,
    pub errmsg: String,
    pub error_idx: usize,
}

impl Error for TokenizerError<'_> {}

const DISPLAYED_CODE_CONTEXT_LEN: usize = 15;

impl Display for TokenizerError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start_idx = self.error_idx.saturating_sub(DISPLAYED_CODE_CONTEXT_LEN);
        let end_idx = min(self.error_idx + DISPLAYED_CODE_CONTEXT_LEN, self.code.len());

        let mut code_context = if start_idx > 0 {
            String::from("...")
        } else {
            String::new()
        };
        code_context.push_str(&self.code[start_idx..end_idx]);
        if end_idx < self.code.len() {
            code_context.push_str("...")
        }

        let mut pointing_arrow = " ".repeat(self.error_idx - start_idx);
        pointing_arrow.push_str("^");

        write!(
            f,
            "Tokenizer error: {}\n{}\n{}",
            self.errmsg, code_context, pointing_arrow
        )
    }
}

// pub struct ParserError
