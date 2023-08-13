use std::io::Write;

use anyhow::Result;

use crate::lexer::{Lexer, Token};

pub fn run() -> Result<()> {
    print!(">> ");
    std::io::stdout().flush()?;
    std::io::stdin().lines().for_each(|line| {
        if let Ok(line) = line {
            let mut lexer = Lexer::new(line);

            while let Ok(token) = lexer.next_token() {
                if token == Token::Eof {
                    print!(">> ");
                    let _ = std::io::stdout().flush();
                    break;
                }
                println!("{:?}", token);
            }
        }
    });

    Ok(())
}
