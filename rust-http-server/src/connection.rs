use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::http::Response;
use crate::parser::parse_request;
use crate::router::Router;

pub async fn handle_connection(stream: TcpStream, router: Arc<Router>) {
    let peer = stream.peer_addr().ok();
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    loop {
        let req = match parse_request(&mut buf_reader).await {
            Ok(Some(r)) => r,
            Ok(None) => break,
            Err(e) => {
                eprintln!("[{:?}] Parse error: {}", peer, e);
                let resp = Response::bad_request().with_text(format!("Bad request: {}", e));
                let _ = writer.write_all(&resp.to_bytes()).await;
                break;
            }
        };

        println!("[{:?}] {:?} {}", peer, req.method, req.path);

        // Route the request
        let response = match router.resolve(&req.method, &req.path) {
            Some((handler, params)) => {
                let mut req = req;
                req.params = params;
                handler(req).await
            }
            None => Response::not_found().with_text("404 – route not found"),
        };

        if let Err(e) = writer.write_all(&response.to_bytes()).await {
            eprintln!("[{:?}] Write error: {}", peer, e);
            break;
        }

        let conn_header = response.headers.get("Connection").map(|s| s.as_str());
        if conn_header == Some("close") {
            break;
        }
    }
}
