use crate::ast::{Expression, Literal, Program, Statement};

use anyhow::{bail, Error, Result};

#[derive(PartialEq, Debug)]
pub enum Object {
    Int(i64),
    Bool(bool),
    Null,
}

pub struct Eval {}

#[allow(dead_code)]
impl Eval {
    fn new() -> Self {
        Self {}
    }

    fn eval(&mut self, program: Program) -> Result<Object> {
        if program.is_empty() {
            bail!("Can not evaluate an empty program!");
        }

        let mut result = Object::Null;

        for statement in program {
            result = self.eval_statement(statement?)?;
        }

        Ok(result)
    }

    fn eval_statement(&mut self, statement: Statement) -> Result<Object> {
        match statement {
            Statement::Expression(expr) => self.eval_expr(expr),
            _ => bail!("Failed to evaluate statement: {:?}", statement),
        }
    }

    fn eval_expr(&mut self, expression: Expression) -> Result<Object> {
        match expression {
            Expression::Literal(literal) => self.eval_literal(literal),
            _ => bail!("Failed to evaluate expression: {:?}", expression),
        }
    }

    fn eval_literal(&mut self, literal: Literal) -> Result<Object> {
        Ok(match literal {
            Literal::Int(num) => Object::Int(num),
            _ => bail!("Failed to evaluate literal: {:?}", literal),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{eval::Object, lexer::Lexer, parser::Parser};

    use super::Eval;

    #[test]
    fn integer_expr() {
        let input = "5";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let mut eval = Eval::new();

        let output = eval.eval(parser.parse_program());

        println!("{:?}", output);
        assert!(output.is_ok());
        assert_eq!(output.unwrap(), Object::Int(5))
    }
}
