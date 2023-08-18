use std::fmt::Display;

use crate::ast::{Expression, IfExpression, Infix, Literal, Prefix, Program, Statement};

use anyhow::Result;

#[derive(PartialEq, Debug)]
pub enum Object {
    Int(i64),
    Bool(bool),
    Null,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Int(num) => write!(f, "{}", num),
            Object::Bool(bool) => write!(f, "{}", bool),
            Object::Null => write!(f, "{}", "NULL"),
        }
    }
}

pub struct Eval {}

#[allow(dead_code)]
impl Eval {
    pub fn new() -> Self {
        Self {}
    }

    pub fn eval_program(&self, program: Program) -> Object {
        if program.iter().any(Result::is_err) {
            return Object::Null;
        }

        let program = program.into_iter().map(Result::unwrap).collect();

        self.eval(&program)
    }

    pub fn eval(&self, program: &Vec<Statement>) -> Object {
        let mut result = Object::Null;

        for statement in program {
            match statement {
                Statement::Let(id, value) => result = Object::Null,
                Statement::Return(ret_value) => {
                    result = self.eval_expr(ret_value);
                    break;
                }
                Statement::Expression(expr) => result = self.eval_expr(expr),
            }
        }

        result
    }
    fn eval_expr(&self, expression: &Expression) -> Object {
        match expression {
            Expression::Literal(literal) => self.eval_literal(literal),
            Expression::Prefix(operator, right) => self.eval_prefix(operator, right),
            Expression::Infix(operator, left, right) => self.eval_infix(operator, left, right),
            Expression::If(if_expr) => self.eval_if(if_expr),
            _ => Object::Null,
        }
    }

    fn eval_if(&self, if_expr: &IfExpression) -> Object {
        let condition = self.eval_expr(&if_expr.condition);

        if self.is_truthy(condition) {
            self.eval(&if_expr.consequence)
        } else {
            self.eval(&if_expr.alternative)
        }
    }

    fn eval_literal(&self, literal: &Literal) -> Object {
        match literal {
            Literal::Int(num) => Object::Int(*num),
            Literal::Bool(bool) => Object::Bool(*bool),
            _ => Object::Null,
        }
    }

    fn eval_infix(&self, operator: &Infix, left: &Expression, right: &Expression) -> Object {
        let left = self.eval_expr(left);
        let right = self.eval_expr(right);

        match (left, right) {
            (Object::Int(l), Object::Int(r)) => self.eval_integer_infix(operator, l, r),
            (Object::Bool(l), Object::Bool(r)) => self.eval_bool_infix(operator, l, r),
            _ => Object::Null,
        }
    }

    fn eval_bool_infix(&self, operator: &Infix, left: bool, right: bool) -> Object {
        match operator {
            Infix::Equal => Object::Bool(left == right),
            Infix::NotEqual => Object::Bool(left != right),
            _ => Object::Null,
        }
    }

    fn eval_integer_infix(&self, operator: &Infix, left: i64, right: i64) -> Object {
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

    fn eval_prefix(&self, operator: &Prefix, right: &Expression) -> Object {
        let expr = self.eval_expr(right);

        match operator {
            Prefix::Not => self.eval_bang(expr),
            Prefix::Minus => self.eval_prefix_minus(expr),
            Prefix::Plus => self.eval_prefix_plus(expr),
        }
    }

    fn eval_prefix_plus(&self, obj: Object) -> Object {
        match obj {
            Object::Int(_) => obj,
            _ => Object::Null,
        }
    }

    fn eval_prefix_minus(&self, obj: Object) -> Object {
        match obj {
            Object::Int(num) => Object::Int(-num),
            _ => Object::Null,
        }
    }

    fn eval_bang(&self, obj: Object) -> Object {
        Object::Bool(match obj {
            Object::Bool(true) => false,
            Object::Bool(false) => true,
            Object::Null => true,
            _ => false,
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

    fn test(tests: HashMap<&str, Object>) {
        for (input, output) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let eval = Eval::new();

            let result = eval.eval_program(parser.parse_program());
            assert_eq!(output, result);
        }
    }

    #[test]
    fn integer_expr() {
        let tests = HashMap::from([
            ("5", Object::Int(5)),
            ("10", Object::Int(10)),
            ("-5", Object::Int(-5)),
            ("-10", Object::Int(-10)),
            ("+10", Object::Int(10)),
            ("5 + 5 + 5 + 5 - 10", Object::Int(10)),
            ("2 * 2 * 2 * 2 * 2", Object::Int(32)),
            ("-50 + 100 + -50", Object::Int(0)),
            ("5 * 2 + 10", Object::Int(20)),
            ("5 + 2 * 10", Object::Int(25)),
            ("20 + 2 * -10", Object::Int(0)),
            ("50 / 2 * 2 + 10", Object::Int(60)),
            ("2 * (5 + 10)", Object::Int(30)),
            ("3 * 3 * 3 + 10", Object::Int(37)),
            ("3 * (3 * 3) + 10", Object::Int(37)),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Object::Int(50)),
        ]);

        test(tests);
    }

    #[test]
    fn bool_expr() {
        let tests = HashMap::from([
            ("true", Object::Bool(true)),
            ("false", Object::Bool(false)),
            ("1 < 2", Object::Bool(true)),
            ("1 > 2", Object::Bool(false)),
            ("1 < 1", Object::Bool(false)),
            ("1 > 1", Object::Bool(false)),
            ("1 == 1", Object::Bool(true)),
            ("1 != 1", Object::Bool(false)),
            ("1 == 2", Object::Bool(false)),
            ("1 != 2", Object::Bool(true)),
            ("true == true", Object::Bool(true)),
            ("false == false", Object::Bool(true)),
            ("true == false", Object::Bool(false)),
            ("true != false", Object::Bool(true)),
            ("false != true", Object::Bool(true)),
            ("(1 < 2) == true", Object::Bool(true)),
            ("(1 < 2) == false", Object::Bool(false)),
            ("(1 > 2) == true", Object::Bool(false)),
            ("(1 > 2) == false", Object::Bool(true)),
        ]);

        test(tests);
    }
    #[test]
    fn bang_operator() {
        let tests = HashMap::from([
            ("!true", Object::Bool(false)),
            ("!false", Object::Bool(true)),
            ("!5", Object::Bool(false)),
            ("!!true", Object::Bool(true)),
            ("!!false", Object::Bool(false)),
            ("!!5", Object::Bool(true)),
        ]);

        test(tests);
    }

    #[test]
    fn if_else() {
        let tests = HashMap::from([
            ("if (true) { 10 }", Object::Int(10)),
            ("if (false) { 10 }", Object::Null),
            ("if (1) { 10 }", Object::Int(10)),
            ("if (1 < 2) { 10 }", Object::Int(10)),
            ("if (1 > 2) { 10 }", Object::Null),
            ("if (1 > 2) { 10 } else { 20 }", Object::Int(20)),
            ("if (1 < 2) { 10 } else { 20 }", Object::Int(10)),
        ]);

        test(tests);
    }

    #[test]
    fn return_statements() {
        let tests = HashMap::from([
            ("return 10;", Object::Int(10)),
            ("return 10; 9;", Object::Int(10)),
            ("return 2 * 5; 9;", Object::Int(10)),
            ("9; return 2 * 5; 9;", Object::Int(10)),
            (
                "if (10 > 1) {
                    if (10 > 1) {
                        return 10;
                    }
                    return 1;
                }",
                Object::Int(10),
            ),
        ]);

        test(tests);
    }
}
