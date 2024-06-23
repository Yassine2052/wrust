use std::sync::{Arc, Mutex, RwLock};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use inject_struct::InjectStruct;

lazy_static! {
    pub static ref IDS_COUNTER: Mutex<usize> = Mutex::new(0);
    pub static ref DATA: Arc<RwLock<Vec<Person> >> = {
        let father = Person::new(74, "Miguel L. Hake".to_string());
        let child1 = Person::new(45, "Annette J. Johnson".to_string());
        let child2 = Person::new(40, "Michael B. Tidwell".to_string());
        let child3 = Person::new(33, "Timothy M. Bad".to_string());
        let child4 = Person::new(23, "Nora J. Cline".to_string());

        let data = Vec::from([father, child1, child2, child3, child4]);

        Arc::new(RwLock::new(data))
    };
}

#[derive(Serialize, Deserialize, InjectStruct, Clone, Debug)]
pub struct Person {
    pub id: usize,
    pub name: String,
    pub age: usize
}

impl Person {
    pub fn new(age: usize, name: String) -> Self {
        let mut current_id = IDS_COUNTER.lock().unwrap();

        *current_id += 1;

        let id = current_id.clone();
        Self {
            id,
            age,
            name
        }
    }
}