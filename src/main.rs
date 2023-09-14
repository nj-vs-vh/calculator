use std::collections::HashMap;

use crate::{
    parser::parse,
    runtime::eval,
    tokenizer::{tokenize, untokenize},
};

mod bracket;
mod errors;
mod parser;
mod runtime;
mod tokenizer;
mod values;

fn main() {
    let code = "a = 1; { a = 2; b = { c = log(3); exp(c)}; a + b } + { a + 5 };";
    // let code = "log(1); exp(3); a = exp; a(3)";
    // let code = "\"hello world\" + \"!!!\";";
    // let code = " a= 1 + 1 + (3 * 5)^2 * 10 + 6; b = a + 5; b = a + b; c = d = a";
    // let code = "1 + 2 + (a ^ b) - log(3+ 5);\n\n3 + 5;\n 4 + f - foo(bar) + 4 (3)";

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

    let eval_result = eval(&expressions, &mut HashMap::new());
    let results = match eval_result {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(vs) => vs,
    };

    println!("{:?}", results);
}
