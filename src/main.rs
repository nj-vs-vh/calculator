use std::{collections::HashMap, fs, path::PathBuf};

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

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "calculator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    filename: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    Fmt,
}

fn main() {
    let args = Cli::parse();

    let code = fs::read_to_string(&args.filename).expect("Failed to read input file");

    // let code = "a = 1; { a = 2; b = { c = log(3); exp(c)}; a + b } + { a + 5 };";
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
    if args.verbose > 0 {
        println!("Tokens:\n{:?}", &tokens);
    }

    if let Some(Commands::Fmt) = args.command {
        let formatted = untokenize(&tokens, false);
        fs::write(&args.filename, formatted).expect("Failed to write formatted code to file");
        return;
    }

    let parser_result = parse(&tokens);
    let expressions = match parser_result {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(exprs) => exprs,
    };
    if args.verbose > 0 {
        println!("AST:\n{:?}", expressions);
    }

    let eval_result = eval(&expressions, &mut HashMap::new());
    let result = match eval_result {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(vs) => vs,
    };

    if args.verbose > 0 {
        println!("Resulting value:\n{:?}", result);
    }
}
