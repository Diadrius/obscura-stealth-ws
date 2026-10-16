//! Page `WebSocket` over the stealth transport: a real round trip, and the same
//! private-network gate as scripted fetch.
#![cfg(feature = "stealth")]

use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use obscura::Browser;

/// Echo server; the counter is how many upgrades it accepted.
fn spawn_ws_echo() -> (String, Arc<AtomicU32>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let accepted = Arc::new(AtomicU32::new(0));
    let count = accepted.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let count = count.clone();
            std::thread::spawn(move || {
                let Ok(mut ws) = tungstenite::accept(stream) else { return };
                count.fetch_add(1, Ordering::SeqCst);
                while let Ok(msg) = ws.read() {
                    if msg.is_close() || ws.send(msg).is_err() {
                        break;
                    }
                }
            });
        }
    });
    (addr.to_string(), accepted)
}

/// Serves one empty HTML page.
fn spawn_http() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for mut s in listener.incoming().flatten() {
            let mut buf = [0u8; 2048];
            let _ = s.read(&mut buf);
            let body = "<title>ws</title>";
            let _ = write!(
                s,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
        }
    });
    format!("http://{addr}")
}

/// Opens a socket from page JS, sends "hi", and records what happens in `window.r`.
/// `evaluate` takes an expression, hence the function wrapper.
fn open_script(ws_addr: &str) -> String {
    format!(
        "(() => {{ window.r = []; const ws = new WebSocket('ws://{ws_addr}'); \
         ws.onopen = () => ws.send('hi'); \
         ws.onmessage = e => {{ window.r.push(e.data); ws.close(); }}; \
         ws.onerror = () => window.r.push('error'); return true; }})()"
    )
}

async fn run_and_collect(page: &mut obscura::Page, script: &str) -> serde_json::Value {
    assert_eq!(page.evaluate(script), serde_json::json!(true), "script did not run");
    for _ in 0..20 {
        page.settle(250).await;
        if page.evaluate("window.r.length").as_u64().is_some_and(|n| n > 0) {
            break;
        }
    }
    page.evaluate("window.r")
}

#[tokio::test]
async fn page_websocket_echoes() {
    std::env::set_var("OBSCURA_ALLOW_PRIVATE_NETWORK", "1");
    let (ws_addr, accepted) = spawn_ws_echo();
    let base = spawn_http();

    let browser = Browser::builder().stealth(true).build().unwrap();
    let mut page = browser.new_page().await.unwrap();
    page.goto(&base).await.unwrap();

    let r = run_and_collect(&mut page, &open_script(&ws_addr)).await;
    assert_eq!(r, serde_json::json!(["hi"]));
    assert_eq!(accepted.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn page_websocket_respects_private_network_gate() {
    std::env::set_var("OBSCURA_ALLOW_PRIVATE_NETWORK", "1");
    let (ws_addr, accepted) = spawn_ws_echo();
    let base = spawn_http();

    let browser = Browser::builder().stealth(true).build().unwrap();
    let mut page = browser.new_page().await.unwrap();
    page.goto(&base).await.unwrap();

    // The page is loaded; from here on loopback is off limits, as for fetch().
    std::env::remove_var("OBSCURA_ALLOW_PRIVATE_NETWORK");
    let r = run_and_collect(&mut page, &open_script(&ws_addr)).await;
    assert_eq!(r, serde_json::json!(["error"]));
    assert_eq!(accepted.load(Ordering::SeqCst), 0);
}
