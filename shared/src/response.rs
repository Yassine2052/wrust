use std::collections::HashMap;
use std::{env, fs};
use serde::Serialize;
use crate::constants::{CONTENT_TYPE_HEADER, CONTENT_TYPE_MAP, DEFAULT_CONTENT_TYPE};

#[derive(Debug)]
pub struct Response {
    status: usize,
    data: String,
    headers: HashMap<String, String>,
    cookies: HashMap<String, String>
}

pub type ResponseResult = Result<Response, String>;


impl Response {
    pub fn new() -> Self {
        Response {
            status: 200,
            data: String::from(""),
            headers: HashMap::new(),
            cookies: HashMap::new()
        }
    }

    pub fn get_status(&self) -> usize {
        self.status.clone()
    }

    pub fn set_cookie(&mut self, key: String, value: String) {
        self.cookies.insert(key, value);
    }

    pub fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn status(&mut self, status: usize) -> &Self {
        self.status = status;
        self
    }
    pub fn text(&mut self, data: String) -> &Self {
        self.headers.insert(String::from(CONTENT_TYPE_HEADER), String::from(DEFAULT_CONTENT_TYPE));
        self.data = data;

        self
    }

    pub fn json<T>(&mut self, data: T) -> &Self
    where T: Serialize {
        match serde_json::to_string(&data) {
            Ok(data) => {
                if let Some(&content_type) = CONTENT_TYPE_MAP.get("json") {
                    self.headers.insert(String::from(CONTENT_TYPE_HEADER), String::from(content_type));
                }
                self.data = data;
            },
            Err(err) => {
                let type_name = std::any::type_name::<Option<T>>();
                let message = format!("Serialization of {:?} Failed: {:?}", type_name, err.to_string());

                self.data = message;
                self.status(500);
            }
        };

        self
    }

    pub fn view(&mut self, file_name: &str) -> &Self {
        let current_dir = env::current_dir().unwrap();
        let mut path = current_dir.as_path().join("src/views");

        let mut file_name = String::from(file_name);

        path = path.join(file_name.clone());

        if path.is_dir() {
            path = path.join("index.html");
        } else {
            if !file_name.ends_with(".html") {
                path.set_extension("html");
            }
        }

        print!("{:?}", path);

        match fs::read_to_string(path) {
            Ok(content) => {
                if let Some(&content_type) = CONTENT_TYPE_MAP.get("html") {
                    self.headers.insert(String::from(CONTENT_TYPE_HEADER), String::from(content_type));
                }

                self.data = content;
            },
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        self.data = String::from("Access Denied");
                        self.status(500);
                    },
                    _ => {
                        self.data = String::from("Not Found");
                        self.status(404);
                    }
                }
            }
        };

        self
    }

    pub fn get_data(&self) -> &String {
        &self.data
    }

    pub fn get_content_type(&self) -> String {
        self.headers.get(CONTENT_TYPE_HEADER).unwrap_or(&DEFAULT_CONTENT_TYPE.to_string()).to_string()
    }
}