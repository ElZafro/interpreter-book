use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::ast::{BlockStatement, Identifier};

use super::env::Env;

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    Int(i64),
    Bool(bool),
    Null,
    ReturnValue(Box<Object>),
    Empty,
    Function(Vec<Identifier>, BlockStatement, Rc<RefCell<Env>>),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(num) => write!(f, "{}", num),
            Self::Bool(bool) => write!(f, "{}", bool),
            Self::Null => write!(f, "{}", "NULL"),
            Self::ReturnValue(value) => write!(f, "{}", *value),
            Self::Empty => Ok(()),
            Self::Function(params, _, _) => {
                write!(f, "fn({})", params.join(","))
            }
        }
    }
}

impl Object {
    pub fn get_type(&self) -> &str {
        match self {
            Object::Int(_) => "int",
            Object::Bool(_) => "bool",
            Object::Null => "null",
            Object::ReturnValue(val) => val.get_type(),
            Object::Empty => "empty",
            Object::Function(_, _, _) => "function",
        }
    }
}
