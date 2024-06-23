mod test;
mod thread_pool;
mod wrust;
pub mod router;
pub mod person;
mod route_builder;

extern crate lazy_static;

use std::process::exit;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use shared::query::QueryParamValue::Single;
use shared::query::QueryParamValueType::{Int, Str};
use crate::person::{DATA, Person};
use crate::wrust::WRust;

fn main(){
    let mut app = WRust::new();

    {
        let binding = Arc::clone(&app.router);
        let mut router = binding.write().unwrap();

        router.get(String::from("/get"), Box::new(move | _request, response| {
            response.json(DATA.clone().read().unwrap().clone())
        }));

        router.get(String::from("/get-view"), Box::new(move | _request, response| {
            response.view("")
        }));

        router.get(String::from("/get-nested-view"), Box::new(move | _request, response| {
            response.view("nested")
        }));

        router.get(String::from("/get-nested-view-test"), Box::new(move | _request, response| {
            response.view("nested/test")
        }));

        router.post(String::from("/create?name&age:int&ids:int+"), Box::new(move | _request, response| {
            let age = if let Some(Single(value)) = _request.queries.get("age").map(|q| &q.value) {
                if let Int(age) = value.clone() {
                    if age >= 18 && age < 256 {
                        let age = age as usize;
                        age
                    } else {
                        response.status(400);
                        return response.text(format!("Invalid Age: {}", age));
                    }
                } else {
                    response.status(400);
                    return response.text(String::from("Invalid Age Format"));
                }
            } else {
                response.status(400);
                return response.text(String::from("Age Not Provided"));
            };

            let name = if let Some(Single(value)) = _request.queries.get("name").map(|q| &q.value) {
                if let Str(name) = value.clone() {
                    if name.len() > 0 {
                        name
                    } else {
                        response.status(400);
                        return response.text(String::from("Name can not be empty"));
                    }
                } else {
                    response.status(400);
                    return response.text(String::from("Invalid Name Format"));
                }
            } else {
                response.status(400);
                return response.text(String::from("Age Not Provided"));
            };

            let person = Person::new(age, name);
            DATA.clone().write().unwrap().push(person.clone());
            response.json(person)
        }));

        router.all(String::from("/all"), Box::new(move | _request, response| {
            response.text(String::from("Hello from any endpoint"))
        }));
    }

    if let Err(err) = app.listen() {
        eprintln!("Failed to bind to a port: {:?}", err);
        exit(1);
    }
}
