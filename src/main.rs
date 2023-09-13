use errors::ParserError;

use crate::tokenizer::{tokenize, untokenize};

mod errors;
mod tokenizer;
fn main() {
    let code = String::from("1 + 2 + (a ^ b) - log(3);\n\n3 + 5;\n 4 + f - foo(bar)");
    let tokenizer_result = tokenize(&code);
    let tokens = match tokenizer_result {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(tokens) => tokens,
    };

    println!("{:?}", tokens);
    println!("{}", untokenize(&tokens));

    let e = ParserError {
        tokens: &tokens,
        errmsg: "example".into(),
        error_token_idx: 10,
    };
    println!("{}", e);
}
