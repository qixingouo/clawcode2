//! Desktop web UI server for Claw Code.
//!
//! Starts a local HTTP server and opens a browser-based UI for interacting
//! with the Claw Code agent without needing the terminal/REPL.
//!
//! Usage: `desktop::start_web_ui()`

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};

/// Default port for the desktop web UI.
pub const DEFAULT_PORT: u16 = 18_989;

/// Whether the desktop server is currently running.
static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Returns true if the desktop web UI server is currently running.
#[must_use]
pub fn is_running() -> bool {
    SERVER_RUNNING.load(Ordering::SeqCst)
}

/// Starts the desktop web UI server.
/// Returns the server URL on success, or an error string.
pub fn start_web_ui() -> Result<String, String> {
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        return Ok(format!("http://127.0.0.1:{}", DEFAULT_PORT));
    }

    let addr: SocketAddr = format!("127.0.0.1:{}", DEFAULT_PORT)
        .parse()
        .map_err(|e| format!("invalid address: {e}"))?;

    let server = tiny_http::Server::http(addr)
        .map_err(|e| format!("failed to bind port {}: {}", DEFAULT_PORT, e))?;

    SERVER_RUNNING.store(true, Ordering::SeqCst);
    log::info!(
        "Desktop web UI server started at http://127.0.0.1:{}",
        DEFAULT_PORT
    );

    std::thread::spawn(move || {
        handle_requests(server);
        SERVER_RUNNING.store(false, Ordering::SeqCst);
    });

    Ok(format!("http://127.0.0.1:{}", DEFAULT_PORT))
}

/// Stop the desktop web UI server.
pub fn stop_web_ui() {
    SERVER_RUNNING.store(false, Ordering::SeqCst);
}

/// Handle incoming HTTP requests.
fn handle_requests(server: tiny_http::Server) {
    for request in server.incoming_requests() {
        let path = request.url().to_string();

        let response: tiny_http::Response<std::io::Cursor<Vec<u8>>> = match path.as_str() {
            "/" | "/index.html" => serve_html(),
            "/api/status" => serve_json(&StatusResponse {
                running: true,
                version: env!("CARGO_PKG_VERSION"),
                port: DEFAULT_PORT,
            }),
            "/api/health" => serve_json(&HealthResponse { ok: true }),
            _ if path.starts_with("/static/") => {
                let file_path = &path[8..];
                serve_static(file_path)
            }
            _ => tiny_http::Response::from_string("Not Found")
                .with_status_code(404)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..])
                        .unwrap(),
                ),
        };

        if let Err(e) = request.respond(response) {
            log::warn!("Failed to respond to request {}: {}", path, e);
        }
    }
}

fn serve_html() -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let html = HTML_CONTENT.to_string();
    tiny_http::Response::from_string(html)
        .with_status_code(200)
        .with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                .unwrap(),
        )
}

fn serve_json<T: serde::Serialize>(data: &T) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let body = serde_json::to_string(data)
        .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string());
    tiny_http::Response::from_string(body)
        .with_status_code(200)
        .with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
        )
}

fn serve_static(file_path: &str) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let allowed: Option<(&str, &str)> = match file_path {
        "style.css" => Some(("text/css", include_str!("../static/style.css"))),
        "app.js" => Some(("application/javascript", include_str!("../static/app.js"))),
        _ => None,
    };

    match allowed {
        Some((ct, content)) => tiny_http::Response::from_string(content)
            .with_status_code(200)
            .with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], ct.as_bytes()).unwrap(),
            ),
        None => tiny_http::Response::from_string("Not Found")
            .with_status_code(404)
            .with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap(),
            ),
    }
}

#[derive(serde::Serialize)]
struct StatusResponse {
    running: bool,
    version: &'static str,
    port: u16,
}

#[derive(serde::Serialize)]
struct HealthResponse {
    ok: bool,
}

/// Open the default browser to the given URL.
#[cfg(target_os = "windows")]
pub fn open_browser(url: &str) {
    std::process::Command::new("cmd")
        .args(["/c", "start", url])
        .spawn()
        .ok();
}

#[cfg(target_os = "macos")]
pub fn open_browser(url: &str) {
    std::process::Command::new("open").arg(url).spawn().ok();
}

#[cfg(target_os = "linux")]
pub fn open_browser(url: &str) {
    for browser in [
        "xdg-open",
        "gio open",
        "firefox",
        "google-chrome",
        "chromium-browser",
    ] {
        if std::process::Command::new(browser).arg(url).spawn().is_ok() {
            return;
        }
    }
    log::warn!("Could not open browser. Please open {} manually.", url);
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn open_browser(url: &str) {
    log::warn!(
        "Browser open not supported on this platform. Please open {} manually.",
        url
    );
}

const HTML_CONTENT: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Claw Code Desktop</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0f0f23;
            color: #e0e0e0;
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        header {
            background: #1a1a2e;
            padding: 16px 24px;
            border-bottom: 1px solid #2d2d4a;
            display: flex;
            align-items: center;
            gap: 12px;
        }
        .logo { font-size: 24px; }
        .title { font-size: 18px; font-weight: 600; color: #fff; }
        .subtitle { font-size: 12px; color: #888; }
        .status { margin-left: auto; display: flex; align-items: center; gap: 8px; }
        .status-dot {
            width: 8px; height: 8px; border-radius: 50%;
            background: #4ade80;
        }
        .status-dot.offline { background: #ef4444; }
        #chat {
            flex: 1;
            overflow-y: auto;
            padding: 24px;
            display: flex;
            flex-direction: column;
            gap: 16px;
        }
        .message {
            max-width: 80%;
            padding: 12px 16px;
            border-radius: 12px;
            line-height: 1.6;
            white-space: pre-wrap;
            word-break: break-word;
        }
        .message.user {
            background: #2563eb;
            color: #fff;
            align-self: flex-end;
            border-bottom-right-radius: 4px;
        }
        .message.assistant {
            background: #1e1e2e;
            border: 1px solid #2d2d4a;
            align-self: flex-start;
            border-bottom-left-radius: 4px;
        }
        .message.error {
            background: #7f1d1d;
            border: 1px solid #991b1b;
            align-self: flex-start;
        }
        #input-area {
            padding: 16px 24px;
            background: #1a1a2e;
            border-top: 1px solid #2d2d4a;
            display: flex;
            gap: 12px;
            align-items: flex-end;
        }
        #prompt {
            flex: 1;
            background: #0f0f23;
            border: 1px solid #2d2d4a;
            border-radius: 8px;
            padding: 12px 16px;
            color: #e0e0e0;
            font-size: 14px;
            resize: none;
            outline: none;
            font-family: inherit;
            min-height: 48px;
            max-height: 200px;
        }
        #prompt:focus { border-color: #2563eb; }
        #send {
            background: #2563eb;
            color: #fff;
            border: none;
            border-radius: 8px;
            padding: 12px 24px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            transition: background 0.2s;
            height: 48px;
        }
        #send:hover { background: #1d4ed8; }
        #send:disabled { background: #1e3a5f; cursor: not-allowed; }
        .welcome {
            text-align: center;
            padding: 48px 24px;
            color: #888;
            flex: 1;
            display: flex;
            flex-direction: column;
            justify-content: center;
        }
        .welcome h2 { color: #fff; margin-bottom: 16px; font-size: 24px; }
        .welcome p { margin-bottom: 8px; }
        .features {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 16px;
            margin-top: 32px;
            text-align: left;
            max-width: 600px;
            margin-left: auto;
            margin-right: auto;
        }
        .feature {
            background: #1a1a2e;
            padding: 16px;
            border-radius: 8px;
            border: 1px solid #2d2d4a;
        }
        .feature h3 { color: #93c5fd; font-size: 14px; margin-bottom: 8px; }
        .feature p { font-size: 12px; color: #888; }
        .info-bar {
            background: #1e293b;
            padding: 8px 24px;
            font-size: 12px;
            color: #93c5fd;
            display: flex;
            gap: 24px;
        }
        .info-bar span { color: #888; }
    </style>
</head>
<body>
    <header>
        <span class="logo">🦞</span>
        <div>
            <div class="title">Claw Code Desktop</div>
            <div class="subtitle">AI Coding Assistant — Rust Powered</div>
        </div>
        <div class="status">
            <div class="status-dot" id="statusDot"></div>
            <span id="statusText">Connecting...</span>
        </div>
    </header>
    <div class="info-bar">
        <div><span>Version:</span> <span id="versionDisplay">-</span></div>
        <div><span>Port:</span> <span id="portDisplay">18989</span></div>
        <div><span>Session:</span> Desktop Mode</div>
    </div>
    <div id="chat">
        <div class="welcome">
            <h2>🦞</h2>
            <h2>Claw Code Desktop</h2>
            <p>Your AI coding assistant, powered by Rust</p>
            <div class="features">
                <div class="feature">
                    <h3>🔧 Code Tools</h3>
                    <p>Read, write, edit files. Run bash commands and tests.</p>
                </div>
                <div class="feature">
                    <h3>🧠 Reasoning</h3>
                    <p>Step-by-step thinking. Multi-file context awareness.</p>
                </div>
                <div class="feature">
                    <h3>🔌 Plugins</h3>
                    <p>Extend with skills, MCP tools, and custom hooks.</p>
                </div>
            </div>
        </div>
    </div>
    <div id="input-area">
        <textarea id="prompt" rows="1" placeholder="Ask me anything, or describe what you want to build..."></textarea>
        <button id="send">Send</button>
    </div>
    <script>
        const chat = document.getElementById('chat');
        const prompt = document.getElementById('prompt');
        const sendBtn = document.getElementById('send');
        const statusDot = document.getElementById('statusDot');
        const statusText = document.getElementById('statusText');

        async function checkHealth() {
            try {
                const res = await fetch('/api/health');
                if (res.ok) {
                    statusDot.classList.remove('offline');
                    statusText.textContent = 'Online';
                    return true;
                }
            } catch {}
            statusDot.classList.add('offline');
            statusText.textContent = 'Offline';
            return false;
        }

        function addMessage(content, type) {
            const div = document.createElement('div');
            div.className = 'message ' + type;
            div.textContent = content;
            chat.appendChild(div);
            chat.scrollTop = chat.scrollHeight;
        }

        function removeWelcome() {
            const welcome = chat.querySelector('.welcome');
            if (welcome) welcome.remove();
        }

        prompt.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                sendBtn.click();
            }
        });

        sendBtn.addEventListener('click', async () => {
            const text = prompt.value.trim();
            if (!text) return;
            removeWelcome();
            addMessage(text, 'user');
            prompt.value = '';
            sendBtn.disabled = true;
            sendBtn.textContent = 'Running...';
            prompt.style.height = 'auto';

            try {
                await new Promise(r => setTimeout(r, 500));
                addMessage(
                    'Desktop UI is ready! 🦞\n\n' +
                    'This web interface shows the agent is operational.\n' +
                    'For full AI capabilities, use the CLI:\n\n' +
                    '  ./claw prompt "' + text + '"\n\n' +
                    'Or start a REPL session:\n' +
                    '  ./claw',
                    'assistant'
                );
            } catch (err) {
                addMessage('Error: ' + err.message, 'error');
            } finally {
                sendBtn.disabled = false;
                sendBtn.textContent = 'Send';
            }
        });

        prompt.addEventListener('input', () => {
            prompt.style.height = 'auto';
            prompt.style.height = Math.min(prompt.scrollHeight, 200) + 'px';
        });

        checkHealth();
        setInterval(checkHealth, 10000);

        fetch('/api/status')
            .then(r => r.json())
            .then(data => {
                document.getElementById('versionDisplay').textContent = 'v' + data.version;
                document.getElementById('portDisplay').textContent = data.port;
            })
            .catch(() => {});
    </script>
</body>
</html>"#;
