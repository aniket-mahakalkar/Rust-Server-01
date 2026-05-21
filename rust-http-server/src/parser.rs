use crate::http::{Method, Request};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};

pub async fn parse_request<R>(stream: &mut BufReader<R>) -> Result<Option<Request>, String>
where
    R: AsyncRead + Unpin,
{
    let mut request_line = String::new();

    let n = stream
        .read_line(&mut request_line)
        .await
        .map_err(|e| e.to_string())?;

    if n == 0 {
        return Ok(None);
    }

    let request_line = request_line.trim_end();

    let parts: Vec<&str> = request_line.splitn(3, ' ').collect();

    if parts.len() != 3 {
        return Err(format!("Malformed request line: {:?}", request_line));
    }

    let method = Method::from_str(parts[0]);
    let raw_path = parts[1].to_string();
    let version = parts[2].to_string();

    let path = raw_path.split('?').next().unwrap_or(&raw_path).to_string();

    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        stream
            .read_line(&mut line)
            .await
            .map_err(|e| e.to_string())?;

        let line = line.trim_end();
        if line.is_empty() {
            break;
        }

        // "Header-Name: value"
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(
                key.trim().to_lowercase(), // normalise keys
                value.trim().to_string(),
            );
        }
    }

    let body = if let Some(len_str) = headers.get("content-length") {
        let len: usize = len_str
            .parse()
            .map_err(|_| "Invalid Content-Length".to_string())?;

        let mut buf = vec![0u8; len];
        stream
            .read_exact(&mut buf)
            .await
            .map_err(|e| e.to_string())?;
        buf
    } else {
        vec![]
    };

    Ok(Some(Request {
        method,
        path,
        version,
        headers,
        body,
        params: HashMap::new(),
    }))
}
