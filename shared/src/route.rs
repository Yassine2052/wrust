use std::collections::HashMap;
use std::fmt::{Debug};
use std::sync::{Arc, RwLock};
use regex::Regex;
use crate::query::{Flags, QueriesHashMap, QueryParamType};
use crate::query::QueryParamValueType::{Boolean, Float, Int, Str, UInt};
use crate::request::{Request};
use crate::response::Response;

pub type Handler = dyn Fn(Request, &mut Response) -> &Response + Sync + Send;
pub type Controller = Arc<RwLock<Box<Handler>>>;
pub type RoutesHashMap = HashMap<String, Route>;
pub type MethodsHashMap = HashMap<RouteMethod, RoutesHashMap>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum RouteMethod {
    RouteGet,
    RoutePost,
    RouteAny
}

pub struct Route
{
    pub queries: QueriesHashMap,
    pub controller: Controller
}

impl Route {
    pub fn new(path: String, handler: Box<Handler>) -> (Route, String) {
        let (queries, path) = Route::generate_queries(path);
        let controller = Arc::new(RwLock::new(handler));

        let route = Route {
            queries,
            controller
        };

        (route, path)
    }

    pub fn generate_queries(path: String) -> (QueriesHashMap, String) {
        let mut result = HashMap::new();
        let mut clean_path = path.clone();

        if let Some((path, query_string)) = path.clone().split_once("?") {
            for param in query_string.split("&") {
                let param_split = param.trim().split(":").collect::<Vec<&str>>();

                if param_split.is_empty() {
                    continue;
                }

                let name = param_split[0].trim();

                match param_split.get(1) {
                    Some(data_type) => {
                        let _type = Self::extract_param_type(*data_type);

                        result.insert(name.to_string(), _type);
                    }
                    _ => {
                        let (_type, name) = Self::extract_name_and_type(name);
                        result.insert(name.to_string(), _type);
                    }
                }
            }

            clean_path = path.to_string();
        }

        (result, clean_path)
    }

    fn extract_param_type(_type: &str) -> QueryParamType {
        let (flags, data_type) = Self::extract_flags(_type);

        let _type = match data_type.as_str() {
            "bool" => Boolean(false),
            "float" => Float(0.0),
            "int" => Int(0),
            "uint" => UInt(0),
            _ => Str(String::new()),
        };

        QueryParamType {
            flags,
            _type
        }
    }

    fn extract_name_and_type(name: &str) -> (QueryParamType, String) {
        let (flags, name) = Self::extract_flags(name);

        let _type = QueryParamType {
            flags,
            _type: Str(String::new())
        };

        (_type, name)
    }

    fn extract_flags(_type: &str) -> (Flags, String) {
        let mut is_optional = false;
        let mut is_array = false;
        let mut allow_empty = false;

        let re = Regex::new(r"\??[*+]?$").unwrap();

        let flags = match re.captures(_type) {
            Some(flags) => {
                if flags.len() == 0 {
                    ""
                } else {
                    flags.get(0).map_or("", |m| m.as_str())
                }
            },
            _ => ""
        };

        if !flags.is_empty() {
            is_optional = flags.contains('?');
            allow_empty = flags.contains('*');
            is_array = allow_empty || flags.contains('+');
        }

        let _type = re.replace(_type, "").to_lowercase().trim().to_string();

        let flags = Flags {
            is_optional,
            is_array,
            allow_empty
        };

        (flags, _type)
    }
}