use shared::route::{Handler, Route, RouteMethod};
use crate::router::Router;

pub struct RouteBuilder {
    router: &'static Router,
    handler: Box<Handler>,
    path: String,
    method: RouteMethod
}

impl RouteBuilder {
    pub fn new(router: &'static mut Router, method: RouteMethod, mut path: String, handler: Box<Handler>) -> Self {
        if !path.starts_with('/') {
            path.insert(0, '/');
        }

        RouteBuilder {
            router,
            handler,
            path,
            method
        }
    }
}