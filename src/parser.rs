use std::mem::take;

use anyhow::{bail, Error, Result};

use crate::{
    ast::{Expression, Identifier, Precedence, Program, Statement},
    lexer::{Lexer, Token},
};

#[allow(dead_code)]
struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
    errors: Vec<Error>,
}

#[allow(dead_code)]
impl Parser {
    fn new(lexer: Lexer) -> Self {
        let mut parser = Self {
            lexer,
            current_token: Token::default(),
            peek_token: Token::default(),
            errors: Vec::new(),
        };

        parser.next_token();
        parser.next_token();

        parser
    }

    fn next_token(&mut self) {
        self.current_token = take(&mut self.peek_token);
        self.peek_token = self.lexer.next_token().unwrap();
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
        self.next_token();

        let name = match self.current_token {
            Token::Ident(_) => self.parse_ident(),
            _ => bail!("Missing indentifier in let statement"),
        };

        self.next_token();
        if self.current_token != Token::Assign {
            bail!("Missing assign token after identifier in let statement");
        }

        self.next_token();
        Ok(Statement::Let(
            name?,
            self.parse_expression(Precedence::Lowest)?,
        ))
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.next_token();

        Ok(Statement::Return(
            self.parse_expression(Precedence::Lowest)?,
        ))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        println!("{:?}, {:?}", self.current_token, self.peek_token);

        let left = match self.current_token {
            Token::Ident(_) => self.parse_ident_expr(),
            Token::Int(_) => self.parse_int_expr(),
            _ => unreachable!("Expression type is unhandled yet!"),
        };

        left
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
        while self.current_token != Token::Semicolon {
            self.next_token();
        }
        self.next_token();
        statement
    }

    fn parse_program(&mut self) -> Program {
        let mut program = Program::new();

        while self.current_token != Token::Eof {
            program.push(self.parse_statement());
        }

        program
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
        let y = 10;
        let foobar = 89;
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

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
        return add(15);
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        assert_eq!(program.len(), 3);

        for p in &program {
            println!("{:?}", p);
        }

        assert!(program.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn identifier_expression() {
        let input = "foobar;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

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

        let program = parser.parse_program();

        assert_eq!(program.len(), 1);
        println!("{:?}", program);
        assert!(program.iter().all(|x| x.is_ok()));
    }
}
