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
use shared::query::QueryParamValueType::{Str, UInt};
use shared::request::RequestData::{Json};
use crate::person::{DATA, Person};
use crate::wrust::WRust;

fn main(){
    let mut app = WRust::new();

    {
        let binding = Arc::clone(&app.router);
        let mut router = binding.write().unwrap();

        router.get(String::from("/get?name?&age:uint"), Box::new(move | _request, response| {
            let age = if let Some(param) = _request.queries_map.get("age") {
                match &param.value {
                    Single(UInt(age)) => age.clone(),
                    _ => 0
                }
            } else {
                0
            };

            let name = if let Some(param) = _request.queries_map.get("name") {
                match &param.value {
                    Single(Str(name)) => name.clone(),
                    _ => String::new()
                }
            } else {
                String::new()
            };

            let binding = DATA.clone().read().unwrap().clone();
            let data = binding.iter().filter(|person| {
                person.age > age && person.name.contains(name.as_str())
            }).collect::<Vec<&Person>>();

            response.json(data)
        }));

        router.get(String::from("/get-view"), Box::new(move | _request, response| {
            response.view("")
        }));

        router.get(String::from("/get-nested-view"), Box::new(move | _request, response| {
            response.view("nested")
        }));

        router.get(String::from("/get-nested-view-test"), Box::new(move | _request, response| {
            println!("{:?}", _request.data);
            response.view("nested/test")
        }));

        router.post(String::from("/create"), Box::new(move | _request, response| {
            match _request.data {
                Json(data) => {
                    let age = if let Some(age_value) = data.get("age") {
                        if let Some(age) = age_value.as_u64(){
                            if age < 18 || age > 120 {
                                response.status(400);
                                return response.text(format!("Invalid Age: {}", age));
                            }

                            age as usize
                        } else {
                            response.status(400);
                            return response.text(String::from("Age Not Provided"));
                        }
                    } else {
                        response.status(400);
                        return response.text(String::from("Age Not Provided"));
                    };

                    let name = if let Some(name_value) = data.get("name") {
                        if let Some(name) = name_value.as_str(){
                            if name.len() < 2 {
                                response.status(400);
                                return response.text(format!("Invalid Age: {}", age));
                            }

                            name.to_string()
                        } else {
                            response.status(400);
                            return response.text(String::from("Name Not Provided"));
                        }
                    } else {
                        response.status(400);
                        return response.text(String::from("Name Not Provided"));
                    };

                    let person = Person::new(age, name);
                    DATA.clone().write().unwrap().push(person.clone());
                    response.json(person)
                },
                _ => {
                    response.status(400);
                    response.text(String::from("Bad Request"))
                }
            }
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