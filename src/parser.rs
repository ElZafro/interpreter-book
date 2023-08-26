use std::mem::take;

use anyhow::{bail, Result};

use crate::{
    ast::{
        BlockStatement, Expression, Identifier, IfExpression, Infix, Literal, Precedence, Prefix,
        Program, Statement,
    },
    lexer::{Lexer, Token},
};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Self {
            lexer,
            current_token: Token::default(),
            peek_token: Token::default(),
        };

        _ = parser.next_token();
        _ = parser.next_token();

        parser
    }

    fn next_token(&mut self) -> Result<()> {
        self.current_token = take(&mut self.peek_token);
        self.peek_token = self.lexer.next_token()?;
        Ok(())
    }

    fn parse_ident(&mut self) -> Result<Identifier> {
        match &self.current_token {
            Token::Ident(name) => Ok(Identifier(name.clone())),
            _ => bail!("Failed to parse identifier"),
        }
    }

    fn parse_ident_expr(&mut self) -> Result<Expression> {
        Ok(Expression::Identifier(self.parse_ident()?))
    }

    fn parse_int_expr(&mut self) -> Result<Expression> {
        match self.current_token {
            Token::Int(num) => Ok(Expression::Literal(crate::ast::Literal::Int(num))),
            _ => bail!("Failed to parse int"),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement> {
        self.next_token()?;

        let name = match self.current_token {
            Token::Ident(_) => self.parse_ident(),
            _ => bail!("Missing indentifier in let statement"),
        };

        self.next_token()?;
        if self.current_token != Token::Assign {
            bail!("Missing assign token after identifier in let statement");
        }

        self.next_token()?;
        Ok(Statement::Let(
            name?,
            self.parse_expression(Precedence::Lowest)?,
        ))
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.next_token()?;

        Ok(Statement::Return(
            self.parse_expression(Precedence::Lowest)?,
        ))
    }

    fn parse_block_statement(&mut self) -> Result<BlockStatement> {
        if self.current_token != Token::LSquirly {
            bail!("Failed to parse block statement!");
        }

        self.next_token()?;

        let mut block = BlockStatement::new();

        while self.current_token != Token::RSquirly && self.current_token != Token::Semicolon {
            block.push(self.parse_statement()?);
            self.next_token()?;
        }

        Ok(block)
    }

    fn parse_if_expr(&mut self) -> Result<Expression> {
        self.next_token()?;

        let condition = self.parse_expression(Precedence::Lowest);

        if self.current_token == Token::Rparen {
            self.next_token()?;
        }

        let consequence = self.parse_block_statement();
        self.next_token()?;

        let alternative = match self.current_token {
            Token::Else => {
                self.next_token()?;
                self.parse_block_statement()
            }
            _ => Ok(BlockStatement::new()),
        };

        Ok(Expression::If(IfExpression {
            condition: Box::new(condition?),
            consequence: consequence?,
            alternative: alternative?,
        }))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Identifier>> {
        let mut params = vec![];

        while self.current_token != Token::Rparen {
            params.push(self.parse_ident()?);

            self.next_token()?;
            if self.current_token == Token::Comma {
                self.next_token()?;
            }
        }
        self.next_token()?;

        Ok(params)
    }

    fn parse_function_expr(&mut self) -> Result<Expression> {
        self.next_token()?;

        if self.current_token != Token::Lparen {
            bail!("Failed to parse function expression!");
        }
        self.next_token()?;

        let params = self.parse_function_parameters()?;

        if self.current_token != Token::LSquirly {
            bail!("Failed to parse function body!");
        }

        let body = self.parse_block_statement()?;

        Ok(Expression::Function { params, body })
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expression>> {
        let mut args = vec![];

        while self.current_token != Token::Rparen {
            args.push(self.parse_expression(Precedence::Lowest)?);

            self.next_token()?;
            if self.current_token == Token::Comma {
                self.next_token()?;
            }
        }

        Ok(args)
    }

    fn parse_call_expr(&mut self, function: Expression) -> Result<Expression> {
        self.next_token()?;

        let args = self.parse_call_args()?;

        Ok(Expression::Call {
            function: Box::new(function),
            args,
        })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        let mut expr = match self.current_token {
            Token::Ident(_) => self.parse_ident_expr(),
            Token::Int(_) => self.parse_int_expr(),
            Token::Bool(_) => self.parse_bool_expr(),
            Token::Lparen => self.parse_grouped_expr(),
            Token::Plus | Token::Bang | Token::Minus => self.parse_prefix_expr(),
            Token::If => self.parse_if_expr(),
            Token::Function => self.parse_function_expr(),
            _ => bail!("Expression type {:?} is unhandled yet!", self.current_token),
        };

        while self.peek_token != Token::Semicolon
            && precedence < Self::get_precedence(&self.peek_token)
        {
            match self.peek_token {
                Token::Plus
                | Token::Minus
                | Token::Slash
                | Token::Asterisk
                | Token::Equal
                | Token::NotEqual
                | Token::Lt
                | Token::Gt => {
                    self.next_token()?;
                    expr = self.parse_infix_expr(expr?);
                }
                Token::Lparen => {
                    self.next_token()?;
                    expr = self.parse_call_expr(expr?);
                }
                _ => bail!("Invalid expression!"),
            }
        }

        expr
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        Ok(Statement::Expression(
            self.parse_expression(Precedence::Lowest)?,
        ))
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        let statement = match self.current_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        };

        if self.peek_token == Token::Semicolon || self.peek_token == Token::Eof {
            self.next_token()?;
        }

        statement
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut program = Program::new();

        while self.current_token != Token::Eof {
            program.push(self.parse_statement());
            self.next_token()?;
        }

        Ok(program)
    }

    fn parse_prefix_expr(&mut self) -> Result<Expression> {
        let prefix = match self.current_token {
            Token::Bang => Prefix::Not,
            Token::Plus => Prefix::Plus,
            Token::Minus => Prefix::Minus,
            _ => unreachable!(),
        };

        self.next_token()?;

        Ok(Expression::Prefix(
            prefix,
            Box::new(self.parse_expression(Precedence::Prefix)?),
        ))
    }

    fn get_precedence(token: &Token) -> Precedence {
        match token {
            Token::Equal | Token::NotEqual => Precedence::Equals,
            Token::Lt | Token::Gt => Precedence::LessGreater,
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Slash | Token::Asterisk => Precedence::Product,
            Token::Lparen => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    fn parse_infix_expr(&mut self, left: Expression) -> Result<Expression> {
        let infix = match self.current_token {
            Token::Plus => Infix::Plus,
            Token::Minus => Infix::Minus,
            Token::Slash => Infix::Divide,
            Token::Asterisk => Infix::Product,
            Token::Equal => Infix::Equal,
            Token::NotEqual => Infix::NotEqual,
            Token::Lt => Infix::LessThan,
            Token::Gt => Infix::GreaterThan,
            _ => bail!("No valid infix operator"),
        };

        let precedence = Self::get_precedence(&self.current_token);
        self.next_token()?;

        Ok(Expression::Infix(
            infix,
            Box::new(left),
            Box::new(self.parse_expression(precedence)?),
        ))
    }

    fn parse_bool_expr(&self) -> Result<Expression> {
        match self.current_token {
            Token::Bool(value) => Ok(Expression::Literal(Literal::Bool(value))),
            _ => bail!("Failed to parse bool expression!"),
        }
    }

    fn parse_grouped_expr(&mut self) -> Result<Expression> {
        self.next_token()?;

        let expr = self.parse_expression(Precedence::Lowest);

        if self.peek_token != Token::Rparen {
            bail!("Failed to parse grouped expression!");
        }

        self.next_token()?;

        expr
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::Lexer;

    use super::Parser;

    #[test]
    fn let_statements() {
        let input = "
        let x = 5;
        let y = true;
        let foobar = y;
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 3);

        for p in &program {
            println!("{:?}", p);
        }

        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn return_statements() {
        let input = "
        return 5;
        return 10;
        return foobar;
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 3);

        for p in &program {
            println!("{:?}", p);
        }

        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn identifier_expression() {
        let input = "foobar;
        foo";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 2);
        println!("{:?}", program);
        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn integer_expression() {
        let input = "555;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        println!("{:?}", program);
        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn prefix_expression() {
        let input = "-5;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        println!("{:?}", program);
        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn infix_expression() {
        let input = r#"10 - 5 * 5;
        -1 + 2;
        alice / bob;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        println!("{:?}", program);
        assert_eq!(program.len(), 3);
        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn boolean_expression() {
        let input = "true == !false;
        !true == 3 < 2 == false;
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 2);
        println!("{:?}", program);
        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn operator_precedence() {
        let input = "(1 + 2) * 5";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        println!("{:?}", program);
        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn simple_ast() {
        let input = "
        let my_var = another_var;
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        println!("{:?}", program);
        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn if_expression() {
        let input = "if (x < y) { x } else { return y; }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        println!("{:?}", program);
        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn call_expression() {
        let input = "add(1, 2 * 3,((alice)), 4 + 5);";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        println!("{:?}", program);
        assert_eq!(program.len(), 1);
        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn function_expression() {
        let input = "fn (x, y) { x + y };
        let foo = fn() {return 69;};
        fn(a){}
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        println!("{:?}", program);
        assert_eq!(program.len(), 3);
        assert!(program.iter().all(|x| x.is_ok()));
    }
}
