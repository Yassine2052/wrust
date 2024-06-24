use std::io::Write;
use std::net::{TcpListener};
use std::sync::{Arc, Mutex, RwLock};
use shared::constants::{DEFAULT_STATUS_CODE, STATUS_CODES_MAP};
use shared::request::{Request};
use shared::response::Response;
use crate::router::Router;
use crate::thread_pool::ThreadPool;

const CRLF: &str = "\r\n";

static USED_PORTS: Mutex<Vec<u16>> = Mutex::new(Vec::new());

pub struct WRust{
    pub router: Arc<RwLock<Router>>,
    port: u16
}

impl WRust {
    pub fn new() -> Self {
        WRust {
            router: Arc::new(RwLock::new(Router::new())),
            port: 8080
        }
    }

    pub fn listen(&mut self) -> Result<(), String> {
        // Bind the port
        if let Some((port, listener)) = Self::get_available_port() {
            // Create a pool of 4 threads to handle requests
            let pool = ThreadPool::new(4);

            USED_PORTS.lock()
                .unwrap()
                .push(port);

            self.port = port.clone();

            {
                let router = Arc::clone(&self.router);
                if let Ok(mut router) = router.write() {
                    router.start_listening();
                };
            }

            println!("Server is listening at {}", port);

            // Listening for incoming TcpStream Requests
            for stream in listener.incoming() {
                // Here we try to get the TcpStream Struct or Panic if error using unwrap
                let mut stream = stream.unwrap();
                let router_wrapper = Arc::clone(&self.router);

                // Handle The request
                pool.execute(move || {
                    let response = &mut Response::new();

                    match Request::read_request_data(&stream) {
                        Ok(mut request) => {
                            match router_wrapper.read() {
                                Ok(router) => {
                                    match router.get_request_endpoint(request.method.clone(), &request.path)  {
                                        Ok(route) => {
                                            match request.map_queries(&route.queries) {
                                                Ok(_) => {
                                                    match route.controller.read() {
                                                        Ok(controller) => {
                                                            controller(request, response);
                                                        },
                                                        Err(err) => {
                                                            response.status(500);
                                                            response.text(err.to_string());
                                                        }
                                                    }
                                                },
                                                Err(err) => {
                                                    response.status(400);
                                                    response.json(err);
                                                }
                                            }
                                        },
                                        Err(err) => {
                                            response.status(404);
                                            response.text(err);
                                        }
                                    }
                                },
                                Err(err) => {
                                    response.status(500);
                                    response.text(err.to_string());
                                }
                            }
                        }
                        Err(err) => {
                            response.status(400);
                            response.text(err);
                        }
                    };

                    let content = response.get_data();
                    let content_length = content.bytes().len();
                    let content_type = response.get_content_type();
                    let status_code = response.get_status();
                    let status_code_description = Self::get_status_code_description(status_code.clone());

                    let res_status = format!("HTTP/1.1 {} {}", status_code, status_code_description);
                    let response = format!("{res_status}{CRLF}Content-Length: {content_length}{CRLF}Content-Type: {content_type}{CRLF}{CRLF}{content}");
                    stream.write_all(response.as_bytes()).unwrap();
                });

                // When connection received and no error is there we print this 💩
                println!("Connection established!");
            }

            return Ok(());
        }

        Err(String::from("No port is available in this range [8080, 8091]"))
    }

    fn get_available_port() -> Option<(u16, TcpListener)>{
        for port in 8080..8091 {
            if let Ok(ports) = USED_PORTS.lock() {
                if ports.contains(&port) {
                    continue;
                }
            }

            let listener = Self::port_is_available(port);

            if listener.is_some() {
                return Some((port, listener.unwrap()));
            }
        }

        None
    }

    fn port_is_available(port: u16) -> Option<TcpListener> {
        match TcpListener::bind(("192.168.1.55", port)) {
            Ok(listener) => Some(listener),
            _ => None
        }
    }

    fn get_status_code_description(status_code: usize) -> String {
        match STATUS_CODES_MAP.get(&status_code) {
            Some(&result) => {
                String::from(result)
            },
            None => {
                let mut result = DEFAULT_STATUS_CODE;

                if status_code > 199 && status_code < 600 {
                    let status_code = (status_code / 100) * 100;

                    if let Some(&description) = STATUS_CODES_MAP.get(&status_code) {
                        result = description;
                    }
                }

                String::from(result)
            }
        }
    }
}