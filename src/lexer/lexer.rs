use anyhow::Result;

#[derive(Debug, PartialEq)]
enum Token {
    Illegal,
    Eof,

    Ident(String),
    Int(String),

    Assign,
    Plus,

    Comma,
    Semicolon,

    Lparen,
    Rparen,
    LSquirly,
    RSquirly,

    Function,
    Let,
}

struct Lexer {
    input: Vec<u8>,
    position: usize,
    read_position: usize,
    ch: u8,
}

impl Lexer {
    fn new(input: String) -> Self {
        let mut lexer = Self {
            input: input.into(),
            position: 0,
            read_position: 0,
            ch: 0,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        self.ch = if self.read_position >= self.input.len() {
            0
        } else {
            self.input[self.read_position]
        };

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let token = match self.ch {
            b'=' => Token::Assign,
            b';' => Token::Semicolon,
            b'(' => Token::Lparen,
            b')' => Token::Rparen,
            b',' => Token::Comma,
            b'+' => Token::Plus,
            b'{' => Token::LSquirly,
            b'}' => Token::RSquirly,
            0 => Token::Eof,

            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                return Ok({
                    let ident = self.read_identifier();
                    match ident.as_str() {
                        "fn" => Token::Function,
                        "let" => Token::Let,
                        _ => Token::Ident(ident),
                    }
                })
            }

            b'0'..=b'9' => return Ok(Token::Int(self.read_int())),

            _ => {
                Token::Illegal
                // unreachable!("No program should contain this token.")
            }
        };

        self.read_char();
        Ok(token)
    }

    fn read_identifier(&mut self) -> String {
        let pos = self.position;
        while self.ch.is_ascii_alphabetic() || self.ch == b'_' {
            self.read_char();
        }
        String::from_utf8_lossy(&self.input[pos..self.position]).to_string()
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }

    fn read_int(&mut self) -> String {
        let pos = self.position;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }
        String::from_utf8_lossy(&self.input[pos..self.position]).to_string()
    }
}

#[cfg(test)]
mod test {
    use anyhow::{Ok, Result};

    use super::{Lexer, Token};

    #[test]
    fn get_next_token() -> Result<()> {
        let input = "=+(){},;";
        let mut lexer = Lexer::new(input.into());

        let tokens = vec![
            Token::Assign,
            Token::Plus,
            Token::Lparen,
            Token::Rparen,
            Token::LSquirly,
            Token::RSquirly,
            Token::Comma,
            Token::Semicolon,
        ];

        for token in tokens {
            assert_eq!(token, lexer.next_token()?);
        }

        Ok(())
    }

    #[test]
    fn get_next_complete() -> Result<()> {
        let input = "let five = 5;
        let ten = 10;
        let add = fn(x, y) {
            x + y;
        };
        let result = add(five, ten);
        ";

        let mut lexer = Lexer::new(input.into());
        let tokens = vec![
            Token::Let,
            Token::Ident(String::from("five")),
            Token::Assign,
            Token::Int(String::from("5")),
            Token::Semicolon,
            Token::Let,
            Token::Ident(String::from("ten")),
            Token::Assign,
            Token::Int(String::from("10")),
            Token::Semicolon,
            Token::Let,
            Token::Ident(String::from("add")),
            Token::Assign,
            Token::Function,
            Token::Lparen,
            Token::Ident(String::from("x")),
            Token::Comma,
            Token::Ident(String::from("y")),
            Token::Rparen,
            Token::LSquirly,
            Token::Ident(String::from("x")),
            Token::Plus,
            Token::Ident(String::from("y")),
            Token::Semicolon,
            Token::RSquirly,
            Token::Semicolon,
            Token::Let,
            Token::Ident(String::from("result")),
            Token::Assign,
            Token::Ident(String::from("add")),
            Token::Lparen,
            Token::Ident(String::from("five")),
            Token::Comma,
            Token::Ident(String::from("ten")),
            Token::Rparen,
            Token::Semicolon,
            Token::Eof,
        ];

        for token in tokens {
            let next_token = lexer.next_token()?;
            println!("expected: {:?}, received: {:?}", token, next_token);
            assert_eq!(token, next_token);
        }

        Ok(())
    }
}
