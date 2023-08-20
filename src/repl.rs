use std::io::Write;

use anyhow::Result;

use crate::{eval::Eval, lexer::Lexer, parser::Parser};

pub fn run() -> Result<()> {
    print!(">> ");
    std::io::stdout().flush()?;

    std::io::stdin().lines().for_each(|line| {
        if let Ok(line) = line {
            let lexer = Lexer::new(line.as_str());
            let mut parser = Parser::new(lexer);
            let eval = Eval::new();

            let result = eval.eval(parser.parse_program());
            println!("{:?}", result);
            print!(">> ");
            _ = std::io::stdout().flush();
        }
    });

    Ok(())
}
