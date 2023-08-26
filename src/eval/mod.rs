pub mod env;
pub mod object;

use std::{cell::RefCell, rc::Rc};

use crate::ast::{
    BlockStatement, Expression, Identifier, IfExpression, Infix, Literal, Prefix, Program,
    Statement,
};

use anyhow::{bail, Result};

use self::{env::Env, object::Object};

pub struct Eval {
    env: Rc<RefCell<Env>>,
}

impl Default for Eval {
    fn default() -> Self {
        Self::new()
    }
}

impl Eval {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Env::new())),
        }
    }

    pub fn eval(&mut self, program: Program) -> Result<Object> {
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

    fn eval_block_statement(&mut self, block: BlockStatement) -> Result<Object> {
        let mut result = Object::Null;

        for statement in block {
            match self.eval_statement(statement) {
                Err(error) => return Err(error),
                Ok(Object::ReturnValue(value)) => return Ok(Object::ReturnValue(value)),
                Ok(obj) => result = obj,
            }
        }
        Ok(result)
    }

    fn eval_statement(&mut self, statement: Statement) -> Result<Object> {
        Ok(match statement {
            Statement::Let(id, value) => {
                let value = self.eval_expr(value)?;
                self.env.borrow_mut().assign(id.0, value.clone());
                Object::Empty
            }
            Statement::Return(ret_value) => {
                Object::ReturnValue(Box::new(self.eval_expr(ret_value)?))
            }
            Statement::Expression(expr) => self.eval_expr(expr)?,
        })
    }

    fn eval_expr(&mut self, expression: Expression) -> Result<Object> {
        match expression {
            Expression::Literal(literal) => self.eval_literal(literal),
            Expression::Prefix(operator, right) => self.eval_prefix(operator, *right),
            Expression::Infix(operator, left, right) => self.eval_infix(operator, *left, *right),
            Expression::If(if_expr) => self.eval_if(if_expr),
            Expression::Identifier(id) => self.eval_identifier(id),
            Expression::Function { params, body } => {
                Ok(Object::Function(params, body, self.env.clone()))
            }
            Expression::Call { function, args } => self.eval_call(*function, args),
        }
    }

    fn eval_identifier(&mut self, id: Identifier) -> Result<Object> {
        if let Some(obj) = self.env.borrow().get(&id.0) {
            return Ok(obj);
        }

        bail!("Identifier {} not found!", id.0);
    }

    fn eval_if(&mut self, if_expr: IfExpression) -> Result<Object> {
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
            Literal::String(s) => Object::String(s),
        })
    }

    fn eval_infix(
        &mut self,
        operator: Infix,
        left: Expression,
        right: Expression,
    ) -> Result<Object> {
        let left = self.eval_expr(left)?;
        let right = self.eval_expr(right)?;

        match (&left, &right) {
            (Object::Int(l), Object::Int(r)) => {
                return Ok(self.eval_integer_infix(operator, *l, *r))
            }

            (Object::Bool(_), Object::Bool(_)) => {
                return self.eval_bool_infix(operator, left, right)
            }
            (Object::String(ref l), Object::String(ref r)) => {
                return self.eval_string_infix(operator, l, r)
            }
            _ => {}
        };
        bail!(format!(
            "Infix operator {} not found for the operands: {} & {}!",
            operator,
            left.get_type(),
            right.get_type()
        ));
    }

    fn eval_bool_infix(&self, operator: Infix, left: Object, right: Object) -> Result<Object> {
        Ok(match operator {
            Infix::Equal => Object::Bool(left == right),
            Infix::NotEqual => Object::Bool(left != right),
            _ => bail!(format!(
                "Infix operator {} not found for the operands: {} & {}!",
                operator,
                left.get_type(),
                right.get_type()
            )),
        })
    }

    fn eval_string_infix(&self, operator: Infix, left: &String, right: &String) -> Result<Object> {
        Ok(match operator {
            Infix::Plus => Object::String(String::from(left) + right),
            _ => bail!(format!(
                "Infix operator {} not found for the operands: string & string!",
                operator,
            )),
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

    fn eval_prefix(&mut self, operator: Prefix, right: Expression) -> Result<Object> {
        let expr = self.eval_expr(right);

        Ok(match operator {
            Prefix::Not => self.eval_bang(expr?)?,
            Prefix::Minus => self.eval_prefix_minus(expr?)?,
            Prefix::Plus => self.eval_prefix_plus(expr?)?,
        })
    }

    fn eval_prefix_plus(&self, obj: Object) -> Result<Object> {
        Ok(match obj {
            Object::Int(_) => obj,
            _ => bail!("Operator prefix + is not defined for {}!", obj.get_type()),
        })
    }

    fn eval_prefix_minus(&self, obj: Object) -> Result<Object> {
        Ok(match obj {
            Object::Int(num) => Object::Int(-num),
            _ => bail!("Operator prefix - is not defined for {}!", obj.get_type()),
        })
    }

    fn eval_bang(&self, obj: Object) -> Result<Object> {
        Ok(match obj {
            Object::Bool(value) => Object::Bool(!value),
            _ => bail!("Operator prefix ! is not defined for {}!", obj.get_type()),
        })
    }

    fn is_truthy(&self, condition: Object) -> bool {
        !matches!(condition, Object::Null | Object::Bool(false))
    }

    fn eval_call(&mut self, function: Expression, args: Vec<Expression>) -> Result<Object> {
        let args = args
            .iter()
            .map(|x| self.eval_expr(x.clone()))
            .collect::<Vec<_>>();

        let function = self.eval_expr(function)?;

        let (params, body, env) = match &function {
            Object::Function(p, b, e) => (p, b, e),
            _ => bail!("{} is not a valid function!", function),
        };

        if params.len() != args.len() {
            bail!(
                "Wrong number of arguments. Expected: {}. Given: {}",
                params.len(),
                args.len()
            );
        }

        let current_env = self.env.clone();

        let mut scoped_env = Env::new();
        scoped_env.outer = Some(env.clone());

        for (id, value) in params.iter().zip(args.into_iter()) {
            scoped_env.assign(id.0.clone(), value?);
        }

        self.env = Rc::new(RefCell::new(scoped_env));
        let obj = self.eval_block_statement(body.clone());

        self.env = current_env;

        obj
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::{
        ast::{Expression, Identifier, Infix, Literal, Statement},
        eval::Object,
        lexer::Lexer,
        parser::Parser,
    };

    use super::{env::Env, Eval};

    use anyhow::{anyhow, Result};

    fn test(tests: HashMap<&str, Result<Object>>) {
        for (input, output) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let mut eval = Eval::new();

            let result = eval.eval(parser.parse_program().unwrap());

            match result {
                Ok(result) => {
                    assert_eq!(output.unwrap(), result);
                }
                _ => {
                    println!("{:?}", result);
                    assert!(output.is_err());
                    assert_eq!(
                        output.err().unwrap().to_string(),
                        result.err().unwrap().to_string()
                    )
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
        ]);

        test(tests);
    }

    #[test]
    fn string_literal() {
        let tests = HashMap::from([(
            r#""Hello World!""#,
            Ok(Object::String("Hello World!".into())),
        )]);

        test(tests);
    }

    #[test]
    fn string_concat() {
        let tests = HashMap::from([(
            r#"
            "Hello" + " "+ "World!"
            "#,
            Ok(Object::String("Hello World!".into())),
        )]);

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
            ("!!true", Ok(Object::Bool(true))),
            ("!!false", Ok(Object::Bool(false))),
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

    #[test]
    fn error_handling() {
        let tests = HashMap::from([
            (
                "5 + true;",
                Err(anyhow!(
                    "Infix operator + not found for the operands: int & bool!"
                )),
            ),
            (
                "5 + true; 5;",
                Err(anyhow!(
                    "Infix operator + not found for the operands: int & bool!"
                )),
            ),
            (
                "-true",
                Err(anyhow!("Operator prefix - is not defined for bool!")),
            ),
            (
                "5 + - !5",
                Err(anyhow!("Operator prefix ! is not defined for int!")),
            ),
            (
                "true + false;",
                Err(anyhow!(
                    "Infix operator + not found for the operands: bool & bool!"
                )),
            ),
            (
                "5; true + false; 5",
                Err(anyhow!(
                    "Infix operator + not found for the operands: bool & bool!"
                )),
            ),
            (
                "if (10 > 1) { true + false; }",
                Err(anyhow!(
                    "Infix operator + not found for the operands: bool & bool!",
                )),
            ),
            (
                "
                if (10 > 1) {
                    if (10 > 1) {
                        return true + false;
                    }
                    return 1;
                }
                ",
                Err(anyhow!(
                    "Infix operator + not found for the operands: bool & bool!",
                )),
            ),
            ("foobar", Err(anyhow!("Identifier foobar not found!"))),
            (
                r#"
            "Hello" - "world"
            "#,
                Err(anyhow!(
                    "Infix operator - not found for the operands: string & string!"
                )),
            ),
        ]);

        test(tests);
    }

    #[test]
    fn let_statements() {
        let tests = HashMap::from([
            ("let a = 5; a;", Ok(Object::Int(5))),
            ("let a = 5 * 5; a;", Ok(Object::Int(25))),
            ("let a = 5; let b = a; b;", Ok(Object::Int(5))),
            (
                "let a = 5; let b = a; let c = a + b + 5; c;",
                Ok(Object::Int(15)),
            ),
        ]);

        test(tests);
    }

    #[test]
    fn function() {
        let tests = HashMap::from([(
            "fn(x) { x + 2; }; ",
            Ok(Object::Function(
                vec![Identifier("x".into())],
                vec![Statement::Expression(Expression::Infix(
                    Infix::Plus,
                    Box::new(Expression::Identifier(Identifier("x".into()))),
                    Box::new(Expression::Literal(Literal::Int(2))),
                ))],
                Rc::new(RefCell::new(Env::new())),
            )),
        )]);

        test(tests);
    }

    #[test]
    fn function_application() {
        let tests = HashMap::from([
            (
                "let identity = fn(x) { x; }; identity(5);",
                Ok(Object::Int(5)),
            ),
            (
                "let identity = fn(x) { return x; }; identity(5);",
                Ok(Object::Int(5)),
            ),
            (
                "let double = fn(x) { x * 2; }; double(5);",
                Ok(Object::Int(10)),
            ),
            (
                "let add = fn(x, y) { x + y; }; add(5, 5);",
                Ok(Object::Int(10)),
            ),
            ("let id = fn(x) { x; }; id(id(5));", Ok(Object::Int(5))),
            (
                "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
                Ok(Object::Int(20)),
            ),
            ("fn(x) { x; }(5)", Ok(Object::Int(5))),
        ]);

        test(tests);
    }

    #[test]
    fn closures() {
        let tests = HashMap::from([(
            "
                let newAdder = fn(x) {
                    fn(y) { x + y };
                };
                let addTwo = newAdder(2);
                addTwo(2);",
            Ok(Object::Int(4)),
        )]);

        test(tests);
    }
}
