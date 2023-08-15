use anyhow::Result;

#[derive(PartialEq, PartialOrd)]
pub enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
}

#[derive(Debug)]
pub enum Prefix {
    Plus,
    Minus,
    Not,
}

#[derive(Debug)]
pub enum Infix {
    Plus,
    Minus,
    Divide,
    Product,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
}

#[derive(Debug)]
pub struct Identifier(pub String);

#[derive(Debug)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    Prefix(Prefix, Box<Expression>),
    Infix(Infix, Box<Expression>, Box<Expression>),
}

#[derive(Debug)]
pub enum Literal {
    Int(i64),
    String(String),
    Bool(bool),
}

#[derive(Debug)]
pub enum Statement {
    Let(Identifier, Expression),
    Return(Expression),
    Expression(Expression),
}

pub type Program = Vec<Result<Statement>>;
