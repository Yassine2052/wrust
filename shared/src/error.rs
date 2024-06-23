use std::collections::HashMap;
use serde::Serialize;

pub type ErrorsHashMap = HashMap<String, String>;

#[derive(Serialize)]
pub struct RequestError {
    name: String,
    errors: ErrorsHashMap
}

impl RequestError {
    pub fn new(name: String) -> Self {
        Self {
            name,
            errors: HashMap::new()
        }
    }

    pub fn set_error(&mut self, name: String, value: String) -> &Self {
        self.errors.insert(name, value);
        self
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}