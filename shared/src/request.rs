use std::string::String;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use serde_json::Value;
use crate::constants::{CONTENT_TYPE_HEADER, COOKIES_HEADER, DEFAULT_CONTENT_TYPE, USER_AGENT_HEADER};
use crate::error::RequestError;
use crate::form_data::FormData;
use crate::wrust_traits::InjectStructTrait;
use crate::query::{QueriesHashMap, QueryParam, QueryParamValueType::{Str}};
use crate::query::QueryParamValue::Multiple;
use crate::request::RequestData::{Json, Text};
use crate::route::{RouteMethod::{self, RouteGet, RoutePost}};
use crate::url_encoding::UrlEncoding;

pub type RequestQueriesHashMap = HashMap<String, QueryParam>;
pub type RequestHeadersHashMap = HashMap<String, String>;
pub type RequestCookiesHashMap = HashMap<String, String>;

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    GET,
    POST
}

#[derive(Debug)]
pub enum RequestData {
    Json(Value),
    Form(FormData),
    Text(String)
}

// Request Line method - path - http version
#[derive(Clone)]
pub struct HttpRequestFirstLine {
    pub method: HttpMethod,
    pub path: String,
    pub http_version: String,
    pub query_string: String
}

#[derive(Debug)]
pub struct IpAddress {
    value: String,
    is_ipv6: bool
}

#[derive(Debug)]
pub struct Request<T: InjectStructTrait = RequestQueriesHashMap>
{
    pub path: String,
    pub method: HttpMethod,
    pub headers: RequestHeadersHashMap,
    pub cookies: RequestCookiesHashMap,
    pub queries_map: RequestQueriesHashMap,
    pub user_agent: String,
    pub ip: IpAddress,
    pub data: RequestData,
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
    pub fn from(value: String, is_ipv6: bool) -> Self {
        Self {
            value,
            is_ipv6
        }
    }
}

impl<T: InjectStructTrait> Request<T> {
    fn from(request_line: HttpRequestFirstLine, headers: RequestHeadersHashMap, cookies: RequestCookiesHashMap, ip: IpAddress, data: RequestData) -> Self {
        Self {
            path: request_line.path,
            method: request_line.method,
            user_agent: headers.get(USER_AGENT_HEADER).unwrap_or(&String::new()).clone(),
            http_version: request_line.http_version,
            query_string: request_line.query_string,
            queries_map: RequestQueriesHashMap::new(),
            queries: T::init(),
            data,
            headers,
            cookies,
            ip
        }
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

    pub fn read_request_data(stream: &TcpStream) -> Result<Request<T>, String> {
        // Read request headers from input stream, to provide efficient reading of chars, arrays, and lines
        let mut buf_reader = BufReader::new(stream);

        // Store Headers Here
        let mut http_request_header = Vec::new();

        // Content Length used to extract the body
        let mut content_length = 0usize;

        // Ip Address
        let ip = if let Ok(socket_addr) = stream.peer_addr() {
            let value = socket_addr.ip().to_string();

            IpAddress {
                value,
                is_ipv6: socket_addr.is_ipv6()
            }
        } else {
            return Err(String::from("No Ip Address Specified"));
        };

        // Iterate over lines till finding an empty line (NO CRLF \r\n)
        for line in buf_reader.by_ref().lines() {
            if let Ok(line) = line {
                if line.is_empty() {
                    break;
                }

                if line.to_lowercase().starts_with("content-length:") {
                    if let Ok(value) = line["content-length:".len()..].trim().parse::<usize>() {
                        content_length = value;
                    }
                }

                http_request_header.push(line);
                continue;
            }

            return Err(String::from("Invalid Http Request"));
        }

        if let Some(request_first_line) = http_request_header.first() {
            println!("processing {:?}", request_first_line);

            let request_line = Self::extract_request_line(request_first_line)?;
            let (headers, cookies) = Self::extract_headers_and_cookies(&http_request_header);

            let content_type = headers.get(CONTENT_TYPE_HEADER).unwrap_or(&String::from(DEFAULT_CONTENT_TYPE)).clone();
            let data = Self::extract_request_data(buf_reader.by_ref(), request_line.method, content_length, content_type);

            let request = Self::from(request_line, headers, cookies, ip, data);
            return Ok(request);
        }

        Err(String::from("Invalid Http Request"))
    }

    fn extract_request_line(request: &String) -> Result<HttpRequestFirstLine, String> {
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

    fn extract_headers_and_cookies(request: &Vec<String>) -> (RequestHeadersHashMap, RequestCookiesHashMap) {
        let mut headers = RequestHeadersHashMap::new();
        let mut cookies = RequestCookiesHashMap::new();

        for i in 1..request.len() {
            if let Some((header_name, header_value)) = request[i].split_once(": ") {
                let header_value = String::from(header_value);

                match header_name {
                    COOKIES_HEADER => {
                        let cookies_array = header_value.split("; ").collect::<Vec<&str>>();
                        for cookie in cookies_array {
                            if let Some((cookie_name, cookie_value)) = cookie.split_once('=') {
                                let cookie_name = String::from(cookie_name);
                                let cookie_value = String::from(cookie_value);

                                cookies.insert(cookie_name, cookie_value);
                            }
                        }
                    },
                    _ => {
                        headers.insert(String::from(header_name), header_value);
                    }
                }

            }
        }

        (headers, cookies)
    }

    fn extract_request_data(buf_reader: &mut BufReader<&TcpStream>, method: HttpMethod, content_length: usize, content_type: String) -> RequestData {
        match method {
            HttpMethod::POST => {
                match content_type.to_lowercase().as_str() {
                    "application/json" => {
                        let mut body = vec![0; content_length];
                        buf_reader.read_exact(&mut body).unwrap();

                        match String::from_utf8(body) {
                            Ok(body_string) => {
                                match serde_json::from_str(body_string.as_str()) {
                                    Ok(value) => return Json(value),
                                    _ => ()
                                }
                            },
                            _ => ()
                        }
                    },
                    _ => ()
                }
            },
            _ => ()
        };

        Text(String::new())
    }
}