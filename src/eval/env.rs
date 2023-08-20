use std::collections::HashMap;

use super::object::Object;

pub struct Env {
    store: HashMap<String, Object>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn get(&self, id: &String) -> Option<Object> {
        if let Some(value) = self.store.get(id) {
            Some(value.clone())
        } else {
            None
        }
    }

    pub fn assign(&mut self, id: String, value: Object) {
        self.store.insert(id, value);
    }
}
