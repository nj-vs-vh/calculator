use crate::{
    parser::parse,
    tokenizer::{tokenize, untokenize},
};

mod errors;
mod parser;
mod tokenizer;

fn main() {
    let code = String::from("1 + 2 + (a ^ b) - log(3+ 5);\n\n3 + 5;\n 4 + f - foo(bar) + 4 (3)");

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

    let parser_result = parse(&tokens);
    let expressions = match parser_result {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(exprs) => exprs,
    };
    println!("{:?}", expressions);
}
