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
    if req_str.to_lowercase().contains("expect: 100-continue") {
        let _ = stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
        let _ = stream.flush();
    }

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

    let content_length: usize = req_str
        .lines()
        .find_map(|l| {
            if l.to_lowercase().starts_with("content-length:") {
                l.split(':').nth(1)?.trim().parse().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    let mut full_body = String::new();
    let header_end_idx = if let Some(idx) = req_str.find("\r\n\r\n") {
        idx + 4
    } else if let Some(idx) = req_str.find("\n\n") {
        idx + 2
    } else {
        req_str.len()
    };

    if header_end_idx < req_str.len() {
        full_body.push_str(&req_str[header_end_idx..]);
    }

    while full_body.len() < content_length {
        let mut extra_buf = [0u8; 4096];
        match stream.read(&mut extra_buf) {
            Ok(n) if n > 0 => {
                full_body.push_str(&String::from_utf8_lossy(&extra_buf[..n]));
            }
            _ => break,
        }
    }

    let body_str = full_body.as_str();

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

    let script = req_json.get("script").and_then(|v| v.as_str()).unwrap_or("");

    // Construct JS payload to evaluate in the page (universal primitives, zero hardcoded business logic)
    let js_command = match action {
        "locate" => format!("SynapseInline.locate({})", serde_json::to_string(selector).unwrap()),
        "click" => format!("SynapseInline.click({}, {}, {})", x, y, serde_json::to_string(selector).unwrap()),
        "write" => format!(
            "SynapseInline.write({}, {})",
            serde_json::to_string(selector).unwrap(),
            serde_json::to_string(text).unwrap()
        ),
        "eval" => format!("(() => {{ {} }})()", script),
        _ => "SynapseInline.harvest()".to_string(),
    };

    let eval_js = format!(
        r#"(() => {{
            const SynapseInline = {{
                locate: (sel) => {{
                    const el = document.querySelector(sel);
                    if (!el) return {{ found: false, error: 'Element not found: ' + sel }};
                    const rect = el.getBoundingClientRect();
                    return {{ found: true, x: Math.round(rect.left + rect.width / 2), y: Math.round(rect.top + rect.height / 2), width: Math.round(rect.width), height: Math.round(rect.height), visible: rect.width > 0 && rect.height > 0 }};
                }},
                click: (x, y, sel) => {{
                    const el = sel ? document.querySelector(sel) : (document.elementFromPoint(x, y) || document.body);
                    if (!el) return {{ status: false, error: 'Element not found' }};
                    el.click();
                    return {{ status: true, action: 'click', targetTag: el.tagName }};
                }},
                write: (sel, txt) => {{
                    const el = document.querySelector(sel);
                    if (!el) return {{ status: false, error: 'Input not found: ' + sel }};
                    el.focus();
                    el.value = txt;
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return {{ status: (el.value === txt), actualValue: el.value, expectedText: txt }};
                }},
                harvest: () => {{
                    const ssrNext = document.getElementById('__NEXT_DATA__');
                    const ssrNuxt = document.getElementById('__NUXT__');
                    const results = Array.from(document.querySelectorAll('a')).map(a => ({{
                        title: (a.innerText || a.textContent || '').trim(),
                        href: a.href
                    }})).filter(r => r.title.length > 4 && r.href && !r.href.startsWith('javascript')).slice(0, 15);
                    return {{
                        url: window.location.href,
                        title: document.title,
                        results: results,
                        resultCount: results.length,
                        hasSSR: !!(ssrNext || ssrNuxt || window.__INITIAL_STATE__),
                        ssrData: ssrNext ? JSON.parse(ssrNext.textContent) : (window.__INITIAL_STATE__ || null)
                    }};
                }}
            }};
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
