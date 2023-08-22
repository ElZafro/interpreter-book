use std::io::Write;

use anyhow::Result;

use crate::{
    eval::{object::Object, Eval},
    lexer::Lexer,
    parser::Parser,
};

pub fn run() -> Result<()> {
    print!(">> ");
    std::io::stdout().flush()?;

    let mut eval = Eval::new();
    std::io::stdin().lines().for_each(|line| {
        if let Ok(line) = line {
            let lexer = Lexer::new(line.as_str());
            let mut parser = Parser::new(lexer);

            let result = eval.eval(parser.parse_program());
            match result {
                Ok(Object::Empty) => {}
                Ok(result) => println!("{}", result),
                Err(result) => println!("ERROR: {}", result),
            }
            print!(">> ");
            _ = std::io::stdout().flush();
        }
    });

    Ok(())
}
