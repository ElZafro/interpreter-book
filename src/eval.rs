use std::fmt::Display;

use crate::ast::{
    BlockStatement, Expression, IfExpression, Infix, Literal, Prefix, Program, Statement,
};

use anyhow::{bail, Result};

#[derive(PartialEq, Debug)]
pub enum Object {
    Int(i64),
    Bool(bool),
    Null,
    ReturnValue(Box<Object>),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(num) => write!(f, "{}", num),
            Self::Bool(bool) => write!(f, "{}", bool),
            Self::Null => write!(f, "{}", "NULL"),
            Self::ReturnValue(value) => write!(f, "{}", *value),
        }
    }
}

pub struct Eval {}

#[allow(dead_code)]
impl Eval {
    pub fn new() -> Self {
        Self {}
    }

    pub fn eval(&self, program: Program) -> Result<Object> {
        let mut result = Object::Null;

        for statement in program {
            match self.eval_statement(statement?) {
                Err(error) => return Err(error),
                Ok(Object::ReturnValue(value)) => return Ok(*value),
                Ok(obj) => result = obj,
            }
        }

        Ok(result)
    }

    fn eval_block_statement(&self, block: BlockStatement) -> Result<Object> {
        let mut result = Object::Null;

        for statement in block {
            match self.eval_statement(statement?) {
                Err(error) => return Err(error),
                Ok(Object::ReturnValue(value)) => return Ok(Object::ReturnValue(value)),
                Ok(obj) => result = obj,
            }
        }
        Ok(result)
    }

    fn eval_statement(&self, statement: Statement) -> Result<Object> {
        Ok(match statement {
            Statement::Let(id, value) => Object::Null,
            Statement::Return(ret_value) => {
                Object::ReturnValue(Box::new(self.eval_expr(ret_value)?))
            }
            Statement::Expression(expr) => self.eval_expr(expr)?,
        })
    }

    fn eval_expr(&self, expression: Expression) -> Result<Object> {
        match expression {
            Expression::Literal(literal) => self.eval_literal(literal),
            Expression::Prefix(operator, right) => self.eval_prefix(operator, *right),
            Expression::Infix(operator, left, right) => self.eval_infix(operator, *left, *right),
            Expression::If(if_expr) => self.eval_if(if_expr),
            _ => Ok(Object::Null),
        }
    }

    fn eval_if(&self, if_expr: IfExpression) -> Result<Object> {
        let condition = self.eval_expr(*if_expr.condition);

        if self.is_truthy(condition?) {
            self.eval_block_statement(if_expr.consequence)
        } else {
            self.eval_block_statement(if_expr.alternative)
        }
    }

    fn eval_literal(&self, literal: Literal) -> Result<Object> {
        Ok(match literal {
            Literal::Int(num) => Object::Int(num),
            Literal::Bool(bool) => Object::Bool(bool),
            _ => Object::Null,
        })
    }

    fn eval_infix(&self, operator: Infix, left: Expression, right: Expression) -> Result<Object> {
        let left = self.eval_expr(left)?;
        let right = self.eval_expr(right)?;

        match (&left, &right) {
            (Object::Int(l), Object::Int(r)) => {
                return Ok(self.eval_integer_infix(operator, *l, *r))
            }

            (Object::Bool(l), Object::Bool(r)) => {
                return Ok(self.eval_bool_infix(operator, *l, *r)?)
            }
            _ => {}
        };
        bail!(
            "No infix operator found for the operands: {:?} & {:?}!",
            left,
            right
        );
    }

    fn eval_bool_infix(&self, operator: Infix, left: bool, right: bool) -> Result<Object> {
        Ok(match operator {
            Infix::Equal => Object::Bool(left == right),
            Infix::NotEqual => Object::Bool(left != right),
            _ => bail!("Operator {:?} is not defined for bool values!", operator),
        })
    }

    fn eval_integer_infix(&self, operator: Infix, left: i64, right: i64) -> Object {
        match operator {
            Infix::Plus => Object::Int(left + right),
            Infix::Minus => Object::Int(left - right),
            Infix::Divide => Object::Int(left / right),
            Infix::Product => Object::Int(left * right),
            Infix::Equal => Object::Bool(left == right),
            Infix::GreaterThan => Object::Bool(left > right),
            Infix::LessThan => Object::Bool(left < right),
            Infix::NotEqual => Object::Bool(left != right),
        }
    }

    fn eval_prefix(&self, operator: Prefix, right: Expression) -> Result<Object> {
        let expr = self.eval_expr(right);

        Ok(match operator {
            Prefix::Not => self.eval_bang(expr?)?,
            Prefix::Minus => self.eval_prefix_minus(expr?),
            Prefix::Plus => self.eval_prefix_plus(expr?)?,
        })
    }

    fn eval_prefix_plus(&self, obj: Object) -> Result<Object> {
        Ok(match obj {
            Object::Int(_) => obj,
            _ => bail!("Operator prefix + is not defined for {:?}!", obj),
        })
    }

    fn eval_prefix_minus(&self, obj: Object) -> Object {
        match obj {
            Object::Int(num) => Object::Int(-num),
            _ => Object::Null,
        }
    }

    fn eval_bang(&self, obj: Object) -> Result<Object> {
        Ok(match obj {
            Object::Bool(value) => Object::Bool(!value),
            _ => bail!("Operator prefix ! is not defined for {:?}", obj),
        })
    }

    fn is_truthy(&self, condition: Object) -> bool {
        match condition {
            Object::Null | Object::Bool(false) => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{eval::Object, lexer::Lexer, parser::Parser};

    use super::Eval;

    use anyhow::{anyhow, Result};

    fn test(tests: HashMap<&str, Result<Object>>) {
        for (input, output) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let eval = Eval::new();

            let result = eval.eval(parser.parse_program());

            match result {
                Ok(result) => {
                    assert_eq!(output.unwrap(), result);
                }
                _ => {
                    assert!(output.is_err())
                }
            }
        }
    }

    #[test]
    fn integer_expr() {
        let tests = HashMap::from([
            ("5", Ok(Object::Int(5))),
            ("10", Ok(Object::Int(10))),
            ("-5", Ok(Object::Int(-5))),
            ("-10", Ok(Object::Int(-10))),
            ("+10", Ok(Object::Int(10))),
            ("5 + 5 + 5 + 5 - 10", Ok(Object::Int(10))),
            ("2 * 2 * 2 * 2 * 2", Ok(Object::Int(32))),
            ("-50 + 100 + -50", Ok(Object::Int(0))),
            ("5 * 2 + 10", Ok(Object::Int(20))),
            ("5 + 2 * 10", Ok(Object::Int(25))),
            ("20 + 2 * -10", Ok(Object::Int(0))),
            ("50 / 2 * 2 + 10", Ok(Object::Int(60))),
            ("2 * (5 + 10)", Ok(Object::Int(30))),
            ("3 * 3 * 3 + 10", Ok(Object::Int(37))),
            ("3 * (3 * 3) + 10", Ok(Object::Int(37))),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Ok(Object::Int(50))),
            ("5++++5", Ok(Object::Int(10))),
            ("5 + - !5", Err(anyhow!(""))),
        ]);

        test(tests);
    }

    #[test]
    fn bool_expr() {
        let tests = HashMap::from([
            ("true", Ok(Object::Bool(true))),
            ("false", Ok(Object::Bool(false))),
            ("1 < 2", Ok(Object::Bool(true))),
            ("1 > 2", Ok(Object::Bool(false))),
            ("1 < 1", Ok(Object::Bool(false))),
            ("1 > 1", Ok(Object::Bool(false))),
            ("1 == 1", Ok(Object::Bool(true))),
            ("1 != 1", Ok(Object::Bool(false))),
            ("1 == 2", Ok(Object::Bool(false))),
            ("1 != 2", Ok(Object::Bool(true))),
            ("true == true", Ok(Object::Bool(true))),
            ("false == false", Ok(Object::Bool(true))),
            ("true == false", Ok(Object::Bool(false))),
            ("true != false", Ok(Object::Bool(true))),
            ("false != true", Ok(Object::Bool(true))),
            ("(1 < 2) == true", Ok(Object::Bool(true))),
            ("(1 < 2) == false", Ok(Object::Bool(false))),
            ("(1 > 2) == true", Ok(Object::Bool(false))),
            ("(1 > 2) == false", Ok(Object::Bool(true))),
        ]);

        test(tests);
    }
    #[test]
    fn bang_operator() {
        let tests = HashMap::from([
            ("!true", Ok(Object::Bool(false))),
            ("!false", Ok(Object::Bool(true))),
            ("!5", Err(anyhow!(""))),
            ("!!true", Ok(Object::Bool(true))),
            ("!!false", Ok(Object::Bool(false))),
            ("!!5", Err(anyhow!(""))),
        ]);

        test(tests);
    }

    #[test]
    fn if_else() {
        let tests = HashMap::from([
            ("if (true) { 10 }", Ok(Object::Int(10))),
            ("if (false) { 10 }", Ok(Object::Null)),
            ("if (1) { 10 }", Ok(Object::Int(10))),
            ("if (1 < 2) { 10 }", Ok(Object::Int(10))),
            ("if (1 > 2) { 10 }", Ok(Object::Null)),
            ("if (1 > 2) { 10 } else { 20 }", Ok(Object::Int(20))),
            ("if (1 < 2) { 10 } else { 20 }", Ok(Object::Int(10))),
        ]);

        test(tests);
    }

    #[test]
    fn return_statements() {
        let tests = HashMap::from([
            ("return 10;", Ok(Object::Int(10))),
            ("return 10; 9;", Ok(Object::Int(10))),
            ("return 2 * 5; 9;", Ok(Object::Int(10))),
            ("9; return 2 * 5; 9;", Ok(Object::Int(10))),
            (
                "if (10 > 1) {
                    if (10 > 1) {
                        return 10;
                    }
                    return 1;
                }",
                Ok(Object::Int(10)),
            ),
        ]);

        test(tests);
    }
}
