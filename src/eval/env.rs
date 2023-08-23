use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::object::Object;

#[derive(Debug, PartialEq, Clone)]
pub struct Env {
    store: HashMap<String, Object>,
    pub outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn get(&self, id: &String) -> Option<Object> {
        match self.store.get(id) {
            Some(value) => Some(value.clone()),
            None => match &self.outer {
                Some(outer) => outer.borrow().get(id),
                None => None,
            },
        }
    }

    pub fn assign(&mut self, id: String, value: Object) {
        self.store.insert(id, value);
    }
}
