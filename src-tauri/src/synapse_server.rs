// ============================================================================
// PageSynapse Local HTTP & IPC Server (Zero-Dependency Lightweight Engine)
// Exposes /status, /exec, and /callback for External Brain Orchestration
// ============================================================================

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

struct ServerState {
    callbacks: Mutex<HashMap<String, mpsc::Sender<String>>>,
    hwnd_str: String,
}

pub fn start_synapse_server(app_handle: AppHandle, hwnd_str: String) {
    let state = Arc::new(ServerState {
        callbacks: Mutex::new(HashMap::new()),
        hwnd_str,
    });

    thread::spawn(move || {
        let port = std::env::var("PAKE_SYNAPSE_PORT").unwrap_or_else(|_| "39999".to_string());
        let addr = format!("127.0.0.1:{}", port);

        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[PageSynapse Server] Failed to bind to {}: {}", addr, e);
                return;
            }
        };

        println!("[PageSynapse Server] Listening on http://{}", addr);

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let state_clone = Arc::clone(&state);
                let app_clone = app_handle.clone();
                let port_clone = port.clone();
                thread::spawn(move || {
                    handle_connection(stream, state_clone, app_clone, &port_clone);
                });
            }
        }
    });
}

fn handle_connection(mut stream: TcpStream, state: Arc<ServerState>, app: AppHandle, port: &str) {
    let mut buffer = [0u8; 16384];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut lines = req_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    // Find HTTP Body
    let body_str = if let Some(idx) = req_str.find("\r\n\r\n") {
        &req_str[idx + 4..]
    } else if let Some(idx) = req_str.find("\n\n") {
        &req_str[idx + 2..]
    } else {
        ""
    };

    if method == "GET" && path.starts_with("/status") {
        let status_json = format!(
            r#"{{"status":"ready","hwnd":"{}","port":{},"pageSynapseReady":true}}"#,
            state.hwnd_str, port
        );
        send_json_response(&mut stream, 200, &status_json);
    } else if method == "POST" && path.starts_with("/exec") {
        handle_exec(&mut stream, &state, &app, body_str, port);
    } else if method == "POST" && path.starts_with("/callback") {
        handle_callback(&mut stream, &state, body_str);
    } else {
        send_json_response(&mut stream, 404, r#"{"error":"Not Found"}"#);
    }
}

fn handle_exec(
    stream: &mut TcpStream,
    state: &Arc<ServerState>,
    app: &AppHandle,
    body_str: &str,
    port: &str,
) {
    let req_json: serde_json::Value = match serde_json::from_str(body_str.trim_matches('\0')) {
        Ok(v) => v,
        Err(_) => {
            send_json_response(stream, 400, r#"{"error":"Invalid JSON body"}"#);
            return;
        }
    };

    let action = req_json
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("harvest");
    let selector = req_json
        .get("selector")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let text = req_json.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let x = req_json.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y = req_json.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let req_id = format!(
        "req_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(12345)
    );

    let (tx, rx) = mpsc::channel();
    {
        let mut callbacks = state.callbacks.lock().unwrap();
        callbacks.insert(req_id.clone(), tx);
    }

    let window = match app.get_webview_window("pake") {
        Some(w) => w,
        None => {
            send_json_response(stream, 500, r#"{"error":"Pake window not found"}"#);
            return;
        }
    };

    // Construct JS payload to evaluate in the page
    let js_command = match action {
        "locate" => format!("window.__PageSynapse__.locate({})", serde_json::to_string(selector).unwrap()),
        "click" => format!("window.__PageSynapse__.click({}, {})", x, y),
        "write" => format!(
            "window.__PageSynapse__.write({}, {})",
            serde_json::to_string(selector).unwrap(),
            serde_json::to_string(text).unwrap()
        ),
        _ => "window.__PageSynapse__.harvest()".to_string(),
    };

    let eval_js = format!(
        r#"(() => {{
            const send = (id, res, isErr) => {{
                const payload = isErr ? {{ id: id, error: String(res) }} : {{ id: id, result: res }};
                if (window.__PageSynapseSendCallback__) {{
                    window.__PageSynapseSendCallback__(JSON.stringify(payload));
                }} else {{
                    fetch('http://127.0.0.1:{}/callback', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }}).catch(e => console.error(e));
                }}
            }};
            try {{
                if (!window.__PageSynapse__) {{
                    throw new Error("PageSynapse JS not loaded yet in this frame");
                }}
                const res = {};
                send('{}', res, false);
            }} catch (err) {{
                send('{}', err, true);
            }}
        }})();"#,
        port, js_command, req_id, req_id
    );

    if let Err(e) = window.eval(&eval_js) {
        send_json_response(
            stream,
            500,
            &format!(r#"{{"error":"Failed to execute eval: {}"}}"#, e),
        );
        return;
    }

    // Wait up to 5 seconds for JS to respond via POST /callback
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(result_str) => {
            send_json_response(stream, 200, &result_str);
        }
        Err(_) => {
            send_json_response(
                stream,
                504,
                r#"{"error":"Timeout waiting for PageSynapse JS response"}"#,
            );
        }
    }

    // Cleanup callback channel
    {
        let mut callbacks = state.callbacks.lock().unwrap();
        callbacks.remove(&req_id);
    }
}

fn handle_callback(stream: &mut TcpStream, state: &Arc<ServerState>, body_str: &str) {
    let cb_json: serde_json::Value = match serde_json::from_str(body_str.trim_matches('\0')) {
        Ok(v) => v,
        Err(_) => {
            send_json_response(stream, 400, r#"{"error":"Invalid callback JSON"}"#);
            return;
        }
    };

    if let Some(id) = cb_json.get("id").and_then(|v| v.as_str()) {
        let mut callbacks = state.callbacks.lock().unwrap();
        if let Some(tx) = callbacks.remove(id) {
            let res_payload = cb_json
                .get("result")
                .map(|r| serde_json::to_string(r).unwrap_or_else(|_| "{}".to_string()))
                .or_else(|| {
                    cb_json
                        .get("error")
                        .map(|e| format!(r#"{{"error":{}}}"#, e))
                })
                .unwrap_or_else(|| r#"{"status":"ok"}"#.to_string());
            let _ = tx.send(res_payload);
        }
    }

    send_json_response(stream, 200, r#"{"status":"callback_received"}"#);
}

fn send_json_response(stream: &mut TcpStream, status_code: u16, body: &str) {
    let status_text = match status_code {
        200 => "200 OK",
        400 => "400 Bad Request",
        404 => "404 Not Found",
        500 => "500 Internal Server Error",
        504 => "504 Gateway Timeout",
        _ => "500 Internal Server Error",
    };

    let response = format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status_text,
        body.len(),
        body
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}
