use std::collections::HashMap;
use std::fmt::{Debug};
use shared::request::{HttpMethod, Request};
use shared::response::Response;
use shared::route::{Handler, MethodsHashMap, Route, RouteMethod};
use shared::route::RouteMethod::{RouteAny, RouteGet, RoutePost};

pub struct Router {
    routes: MethodsHashMap,
    listening: bool
}

impl Router {
    pub fn new() -> Router {
        Router{
            routes: HashMap::new(),
            listening: false
        }
    }

    pub fn get(&mut self, path: String, handler: Box<Handler>) -> &Self {
        self.add_route(RouteGet, path, handler)
    }

    pub fn post(&mut self, path: String, handler: Box<Handler>) -> &Self {
        self.add_route(RoutePost, path, handler)
    }

    pub fn all(&mut self, path: String, handler: Box<Handler>) -> &Self {
        self.add_route(RouteAny, path, handler)
    }

    fn add_route(&mut self, method: RouteMethod, mut path: String, handler: Box<Handler>) -> &Self {
        if self.listening {
            return self;
        }

        if !path.starts_with('/') {
            path.insert(0, '/');
        }

        let (route, path) = Route::new(path.clone(), handler);

        if !self.routes.contains_key(&method) {
            self.routes.insert(method, HashMap::new());
        }

        self.routes.get_mut(&method).unwrap().insert(path, route);
        self
    }

    pub fn start_listening(&mut self){
        self.listening = true;
    }

    pub fn get_request_endpoint(&self, method: HttpMethod, path: &String) -> Result<&Route, String>{
        if let Some(handler) = self.get_method_endpoint(&method.get_route_method(), path) {
            return Ok(handler);
        }

        if let Some(handler) = self.get_method_endpoint(&RouteAny, path) {
            return Ok(handler);
        }

        Err(format!("No corresponding endpoint: {:?}", path))
    }

    fn get_method_endpoint(&self, method: &RouteMethod, path: &String) -> Option<&Route> {
        if let Some(routes) = self.routes.get(method) {
            let route = routes.get(path);
            return route;
        }

        None
    }
}