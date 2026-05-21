use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    Get,
    Post,
    Delete,
    Put,
    Head,
    Options,
    Unkown(String),
}

impl Method {
    pub fn from_str(s: &str) -> Self {
        match s {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "DELETE" => Method::Delete,
            "PUT" => Method::Put,
            "HEAD" => Method::Head,
            "OPTIONS" => Method::Options,
            other => Method::Unkown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone)]

pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub params: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub reason: &'static str,
}

impl Response {
    pub fn new(status: u16, reason: &'static str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Server".into(), "rust-http/0.1".into());
        headers.insert("Connecion".into(), "close".into());
        Response {
            status,
            reason,
            headers,
            body: vec![],
        }
    }

    pub fn ok() -> Self {
        Self::new(200, "OK")
    }
    pub fn not_found() -> Self {
        Self::new(404, "Not Found")
    }
    pub fn internal_error() -> Self {
        Self::new(500, "Internal Server Error")
    }

    pub fn method_not_allowed() -> Self {
        Self::new(405, "Method Not Allowed")
    }

    pub fn bad_request() -> Self {
        Self::new(400, "Bad Request")
    }

    pub fn with_text(mut self, body: impl Into<String>) -> Self {
        let b = body.into().into_bytes();
        self.headers
            .insert("Content-Type".into(), "text/plain; charset=utf-8".into());
        self.headers
            .insert("Content-Length".into(), b.len().to_string());
        self.body = b;
        self
    }

    pub fn with_json(mut self, body: impl Into<String>) -> Self {
        let b = body.into().into_bytes();
        self.headers
            .insert("Content-Type".into(), "application/json".into());
        self.headers
            .insert("Content-Length".into(), b.len().to_string());
        self.body = b;
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = format!("HTTP/1.1 {} {}\r\n", self.status, self.reason).into_bytes();

        for (k, v) in &self.headers {
            out.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
        }
        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(&self.body);
        out
    }
}
