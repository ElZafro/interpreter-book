use anyhow::{bail, Result};

#[derive(Debug, PartialEq, Default)]
pub enum Token {
    #[default]
    Illegal,
    Eof,

    Ident(String),
    Int(i64),
    Bool(bool),
    String(String),

    Assign,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Bang,
    Lt,
    Gt,

    Equal,
    NotEqual,

    Comma,
    Semicolon,

    Lparen,
    Rparen,
    LSquirly,
    RSquirly,

    Function,
    Let,
    If,
    Else,
    Return,
}

pub struct Lexer {
    input: Vec<u8>,
    position: usize,
    read_position: usize,
    ch: u8,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
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

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let token = match self.ch {
            b'=' => {
                if self.peek() == b'=' {
                    self.read_char();
                    Token::Equal
                } else {
                    Token::Assign
                }
            }
            b';' => Token::Semicolon,
            b'(' => Token::Lparen,
            b')' => Token::Rparen,
            b',' => Token::Comma,
            b'+' => Token::Plus,
            b'-' => Token::Minus,
            b'*' => Token::Asterisk,
            b'/' => Token::Slash,
            b'!' => {
                if self.peek() == b'=' {
                    self.read_char();
                    Token::NotEqual
                } else {
                    Token::Bang
                }
            }
            b'<' => Token::Lt,
            b'>' => Token::Gt,
            b'{' => Token::LSquirly,
            b'}' => Token::RSquirly,
            0 => Token::Eof,

            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                return Ok({
                    let ident = self.read_identifier();
                    match ident.as_str() {
                        "fn" => Token::Function,
                        "let" => Token::Let,
                        "if" => Token::If,
                        "else" => Token::Else,
                        "true" => Token::Bool(true),
                        "false" => Token::Bool(false),
                        "return" => Token::Return,
                        _ => Token::Ident(ident),
                    }
                })
            }

            b'0'..=b'9' => return Ok(Token::Int(self.read_int())),
            b'"' => return Ok(Token::String(self.read_string()?)),
            _ => bail!("No program should contain this token: {}", self.ch as char),
        };

        self.read_char();
        Ok(token)
    }

    fn read_string(&mut self) -> Result<String> {
        self.read_char();

        let pos = self.position;
        while self.ch != b'"' {
            self.read_char();
            if self.ch == 0 {
                bail!("String is not properly closed!")
            }
        }
        self.read_char();

        Ok(String::from_utf8_lossy(&self.input[pos..self.position - 1]).to_string())
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

    fn read_int(&mut self) -> i64 {
        let pos = self.position;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }
        String::from_utf8_lossy(&self.input[pos..self.position])
            .to_string()
            .parse()
            .unwrap()
    }

    fn peek(&self) -> u8 {
        if self.read_position >= self.input.len() {
            0
        } else {
            self.input[self.read_position]
        }
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
        let input = r#"let five = 5;
        let ten = 10;
        
        let add = fn(x, y) {
            x + y;
        };

        let result = add(five, ten);
        !-/*5;
        5 < 10 > 5;

        if (5 < 10) {
            return true;
        } else {
            return false;
        }

        10 == 10;
        10 != 9;

        "foobar"
        "foo bar""#;

        let mut lexer = Lexer::new(input.into());
        let tokens = vec![
            Token::Let,
            Token::Ident(String::from("five")),
            Token::Assign,
            Token::Int(5),
            Token::Semicolon,
            Token::Let,
            Token::Ident(String::from("ten")),
            Token::Assign,
            Token::Int(10),
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
            Token::Bang,
            Token::Minus,
            Token::Slash,
            Token::Asterisk,
            Token::Int(5),
            Token::Semicolon,
            Token::Int(5),
            Token::Lt,
            Token::Int(10),
            Token::Gt,
            Token::Int(5),
            Token::Semicolon,
            Token::If,
            Token::Lparen,
            Token::Int(5),
            Token::Lt,
            Token::Int(10),
            Token::Rparen,
            Token::LSquirly,
            Token::Return,
            Token::Bool(true),
            Token::Semicolon,
            Token::RSquirly,
            Token::Else,
            Token::LSquirly,
            Token::Return,
            Token::Bool(false),
            Token::Semicolon,
            Token::RSquirly,
            Token::Int(10),
            Token::Equal,
            Token::Int(10),
            Token::Semicolon,
            Token::Int(10),
            Token::NotEqual,
            Token::Int(9),
            Token::Semicolon,
            Token::String("foobar".into()),
            Token::String("foo bar".into()),
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
