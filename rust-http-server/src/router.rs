use crate::http::{Method, Request, Response};
use std::collections::HashMap;

pub type Handler = Box<
    dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
        + Send
        + Sync,
>;

pub struct Route {
    pub method: Method,
    pub pattern: Vec<String>,
    pub handler: Handler,
}

pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Router { routes: vec![] }
    }

    pub fn add<F, Fut>(&mut self, method: Method, pattern: &str, handler: F)
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Response> + Send + 'static,
    {
        let segments = pattern
            .trim_matches('/')
            .split('/')
            .map(str::to_string)
            .collect();

        self.routes.push(Route {
            method,
            pattern: segments,
            handler: Box::new(move |req| Box::pin(handler(req))),
        });
    }

    pub fn get<F, Fut>(&mut self, p: &str, h: F)
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Response> + Send + 'static,
    {
        self.add(Method::Get, p, h)
    }

    pub fn post<F, Fut>(&mut self, p: &str, h: F)
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Response> + Send + 'static,
    {
        self.add(Method::Post, p, h)
    }

    pub fn resolve<'a>(
        &'a self,
        method: &Method,
        path: &str,
    ) -> Option<(&'a Handler, HashMap<String, String>)> {
        let segments: Vec<&str> = path.trim_matches('/').split('/').collect();

        for route in &self.routes {
            if &route.method != method {
                continue;
            }
            if route.pattern.len() != segments.len() {
                continue;
            }

            let mut params = HashMap::new();
            let mut matched = true;

            for (pat, seg) in route.pattern.iter().zip(segments.iter()) {
                if let Some(key) = pat.strip_prefix(':') {
                    // Dynamic segment — capture it
                    params.insert(key.to_string(), seg.to_string());
                } else if pat != seg {
                    matched = false;
                    break;
                }
            }

            if matched {
                return Some((&route.handler, params));
            }
        }
        None
    }
}
