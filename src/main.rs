use crate::tokenizer::tokenize;

mod errors;
mod tokenizer;
fn main() {
    let code = String::from("1 + 111111 +");
    let tokenizer_result = tokenize(&code);
    match tokenizer_result {
        Err(e) => {
            println!("{}", e);
        }
        Ok(tokens) => {
            println!("{:?}", tokens);
        }
    }
}
