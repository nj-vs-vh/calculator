use crate::{
    parser::parse,
    runtime::eval,
    tokenizer::{tokenize, untokenize},
};

mod errors;
mod parser;
mod runtime;
mod tokenizer;
mod values;

fn main() {
    let code = String::from(" a= 1 + 1 + (3 * 5)^2 * 10 + 6; b = a + 5; b = a + b; c = d = a");
    // let code = String::from("1 + 2 + (a ^ b) - log(3+ 5);\n\n3 + 5;\n 4 + f - foo(bar) + 4 (3)");

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

    let eval_result = eval(&expressions);
    let results = match eval_result {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(vs) => vs,
    };

    println!("{:?}", results);
}
