pub mod ast;
pub mod lexer;
pub mod parser;
pub mod repl;

use anyhow::Result;

fn main() -> Result<()> {
    println!("Hello world! This is the Monkey programming language!");
    println!("Type in commands:");
    repl::run()?;

    Ok(())
}
