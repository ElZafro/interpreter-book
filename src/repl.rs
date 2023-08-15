use std::io::Write;

use anyhow::Result;

use crate::{lexer::Lexer, parser::Parser};

pub fn run() -> Result<()> {
    print!(">> ");
    std::io::stdout().flush()?;

    std::io::stdin().lines().for_each(|line| {
        if let Ok(line) = line {
            let lexer = Lexer::new(line.as_str());
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program();
            println!("{:?}", program);
            print!(">> ");
            _ = std::io::stdout().flush();
        }
    });

    Ok(())
}
