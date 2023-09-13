use crate::tokenizer::{tokenize, untokenize};

mod errors;
mod tokenizer;
fn main() {
    let code = String::from("1 + 2 + (a ^ b) - log(3)");
    let tokenizer_result = tokenize(&code);
    match tokenizer_result {
        Err(e) => {
            println!("{}", e);
        }
        Ok(tokens) => {
            println!("{:?}", tokens);
            println!("{}", untokenize(&tokens))
        }
    }
}
