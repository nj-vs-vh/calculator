use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    debug::print_tree,
    parser::parse,
    runtime::eval,
    tokenizer::{tokenize, untokenize},
};

mod bracket;
mod debug;
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
    Fmt {
        #[arg(short, long)]
        minified: bool,
    },
}

fn main() {
    let args = Cli::parse();

    let code = fs::read_to_string(&args.filename).expect("Failed to read input file");

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

    if let Some(Commands::Fmt { minified }) = args.command {
        let formatted = untokenize(&tokens, minified);
        fs::write(&args.filename, formatted).expect("Failed to write formatted code to file");
        return;
    }

    let parser_result = parse(&tokens);
    let expression = match parser_result {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(exprs) => exprs,
    };
    if args.verbose > 0 {
        println!("AST:");
        print_tree(&expression);
    }

    let eval_result = eval(&expression, &mut HashMap::new());
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
