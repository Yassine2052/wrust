use std::string::String;
use std::any::Any;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use crate::error::RequestError;
use crate::wrust_traits::InjectStructTrait;
use crate::query::{QueriesHashMap, QueryParam, QueryParamValueType::{Str}};
use crate::query::QueryParamValue::Multiple;
use crate::route::{RouteMethod::{self, RouteGet, RoutePost}};
use crate::url_encoding::UrlEncoding;

pub type RequestQueriesHashMap = HashMap<String, QueryParam>;

// Request Line method - path - http version
pub struct HttpRequestFirstLine {
    pub method: HttpMethod,
    pub path: String,
    pub http_version: String,
    pub query_string: String
}

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    GET,
    POST
}

#[derive(Debug)]
pub struct IpAddress {
    ipv4: String,
    ipv6: Option<String>
}

#[derive(Debug)]
pub struct Request<T: InjectStructTrait = RequestQueriesHashMap>
{
    pub path: String,
    pub data: Box<dyn Any>,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub queries_map: RequestQueriesHashMap,
    pub user_agent: String,
    pub ip: IpAddress,
    pub http_version: String,
    pub query_string: String,
    pub queries: T
}

impl InjectStructTrait for RequestQueriesHashMap {
    fn init() -> Self where Self: Sized {
        HashMap::new()
    }

    fn from_hashmap(hashmap: &RequestQueriesHashMap) -> Self where Self: Sized {
        let mut result = RequestQueriesHashMap::new();

        for (key, &ref value) in hashmap {
            result.insert(key.clone(), value.clone());
        }

        result
    }
}

impl HttpMethod {
    pub fn get_route_method(&self) -> RouteMethod {
        match self {
            HttpMethod::GET => RouteGet,
            HttpMethod::POST => RoutePost
        }
    }
}

impl IpAddress {
    pub fn from(ipv4: String, ipv6: Option<String>) -> Self {
        Self {
            ipv4,
            ipv6
        }
    }
}

impl<T: InjectStructTrait> Request<T> {
    fn from(path: String, method: HttpMethod, http_version: String, query_string: String) -> Self {
        Self {
            path,
            method,
            data: Box::new(String::new()),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            queries_map: RequestQueriesHashMap::new(),
            queries: T::init(),
            user_agent: String::new(),
            ip: IpAddress::from(String::from("127.0.0.1"), None),
            http_version,
            query_string,
        }
    }

    pub fn read_request_data(stream: &TcpStream) -> Result<Request<T>, String> {
        // Read request from input stream, to provide efficient reading of chars, arrays, and lines
        let buf_reader = BufReader::new(stream);

        // iterate Over Lines till finding an empty line (NO CRLF \r\n)
        let http_request: Vec<_> = buf_reader
            // Read Lines
            .lines()
            // Unwrap Result<Line> After Reading
            .map(|result| result.unwrap())
            // Iterate till finding an empty line (end of request content)
            .take_while(|line| !line.is_empty())
            // Convert to Vec
            .collect();

        if let Some(request_first_line) = http_request.first() {
            println!("processing {:?}", request_first_line);

            return match Self::extract_first_line_data(request_first_line) {
                Ok(HttpRequestFirstLine{ method, http_version, path, query_string }) => {
                    let request = Self::from(path, method, http_version, query_string);
                    Ok(request)
                },
                Err(err) => Err(err)
            }
        }

        Err(String::from("Invalid Http Request"))
    }

    pub fn map_queries(&mut self, queries_hash_map: &QueriesHashMap) -> Result<(), RequestError> {
        let query_string = self.query_string.clone();

        for param in query_string.split("&") {
            let param_split = param.trim().split("=").collect::<Vec<&str>>();

            if param_split.is_empty() {
                continue;
            }

            let param_name = String::from(param_split[0]);
            let param_value = String::from(
                if param_split.len() > 1  {
                    param_split[1]
                } else {
                    ""
                }
            );

            let value = match self.queries_map.get_mut(param) {
                Some(query_param) => {
                    match queries_hash_map.get(&param_name) {
                        Some(query_param_type) => {
                            query_param.add_value(param_value, query_param_type._type.clone())
                        },
                        None => {
                            query_param.add_value(param_value, Str(String::new()))
                        }
                    }
                },
                None => {
                    match queries_hash_map.get(&param_name) {
                        Some(query_param_type) => {
                            QueryParam::from(param_value, query_param_type._type.clone(), query_param_type.flags.is_array)
                        },
                        None => {
                            QueryParam::from(param_value, Str(String::new()), true)
                        }
                    }
                }
            };

            if let Some(query_param) = value {
                self.queries_map.insert(param_name, query_param);
            }
        }

        let mut request_error = RequestError::new(String::from("query string"));

        for (name, param_type) in queries_hash_map {
            if !param_type.flags.is_optional && !self.queries_map.contains_key(name) {
                if param_type.flags.is_array && !param_type.flags.allow_empty {
                    request_error.set_error(name.clone(), format!("{} can not be empty", name.clone()));
                    continue;
                }
                request_error.set_error(name.clone(), format!("{} is required", name.clone()));
                continue;
            }

            if let Some(query) = self.queries_map.get(name) {
                if let Multiple(value) = &query.value {
                    if value.is_empty() && !param_type.flags.allow_empty {
                        request_error.set_error(name.clone(), format!("{} can not be empty", name.clone()));
                    }
                }
            } else {
                if param_type.flags.is_array && !param_type.flags.allow_empty {
                    request_error.set_error(name.clone(), format!("{} can not be empty", name.clone()));
                }
            }
        }

        if request_error.has_error() {
            return Err(request_error);
        }

        self.queries = T::from_hashmap(&self.queries_map);

        Ok(())
    }

    fn extract_first_line_data(request: &String) -> Result<HttpRequestFirstLine, String> {
        let request_split = request.split(' ').collect::<Vec<&str>>();

        if request_split.len() != 3 {
            return Err(String::from("Invalid Http Request"));
        }

        let method = match request_split[0] {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            _ => return Err(String::from("Unknown Method"))
        };

        let mut path = UrlEncoding::url_decode(String::from(request_split[1]))?;
        let mut query_string = String::new();

        if let Some((endpoint, queries)) = path.to_string().split_once('?') {
            path = String::from(endpoint);
            query_string = String::from(queries);
        }

        let http_version = request_split[2];

        if !["HTTP/1.1", "HTTP/2"].contains(&http_version) {
            return Err(String::from("Invalid Http Version"))
        }

        let http_version = String::from(http_version);

        return Ok(HttpRequestFirstLine {
            method,
            path,
            http_version,
            query_string
        });
    }
}