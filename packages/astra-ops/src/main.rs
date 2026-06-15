use astra_core::exchange::ExchangeRuntime;
use astra_core::gateway::ExecutionGateway;
use astra_core::hashing::DeterministicState;
use astra_core::journal::EventJournal;
use astra_core::kernel::AstraKernel;
use astra_core::replay::EventReducer;
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::StrategyRuntime;
use astra_core::types::{Money, Quantity};
use astra_ops::telemetry::OperationalTelemetry;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

struct DaemonConfig {
    node_id: String,
    http_port: u16,
    journal_path: String,
}

impl DaemonConfig {
    fn from_env() -> Self {
        let node_id = env::var("ASTRA_NODE_ID").unwrap_or_else(|_| "0".to_string());
        Self {
            node_id: node_id.clone(),
            http_port: env::var("ASTRA_HTTP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            journal_path: env::var("ASTRA_JOURNAL_PATH")
                .unwrap_or_else(|_| format!("journal_data_{}.astra_jl", node_id)),
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    telemetry: Arc<OperationalTelemetry>,
    node_id: String,
    gateway: Arc<Mutex<ExecutionGateway>>,
) {
    let mut buf = [0u8; 1024];
    let n = match stream.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request = String::from_utf8_lossy(&buf[..n]);
    let first_line = request.lines().next().unwrap_or("");

    let (status, content_type, body) = if first_line.contains("GET /metrics") {
        let body = telemetry.render_prometheus();
        ("200 OK", "text/plain; version=0.0.4; charset=utf-8", body)
    } else if first_line.contains("GET /health") {
        let body = format!("{{\"status\":\"ok\",\"node\":\"{node_id}\"}}\n");
        ("200 OK", "application/json", body)
    } else if first_line.contains("POST /ingest") {
        // Extract payload from body
        let parts: Vec<&str> = request.split("\r\n\r\n").collect();
        let payload = if parts.len() > 1 {
            parts[1].as_bytes().to_vec()
        } else {
            vec![]
        };

        let mut gw = gateway.lock().await;
        if let Err(e) = gw.process_external_payload(payload) {
            let body = format!("{{\"status\":\"error\",\"reason\":\"{}\"}}\n", e);
            ("500 Internal Server Error", "application/json", body)
        } else {
            let body = format!("{{\"status\":\"ok\",\"node\":\"{node_id}\"}}\n");
            ("200 OK", "application/json", body)
        }
    } else {
        ("404 Not Found", "text/plain", "Not Found\n".to_string())
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );

    let _ = stream.write_all(response.as_bytes()).await;
}

#[tokio::main]
async fn main() {
    let config = DaemonConfig::from_env();
    let telemetry = OperationalTelemetry::new();

    // 1. EventJournal::open(journal_path from env)
    let journal = EventJournal::open(&config.journal_path)
        .or_else(|_| EventJournal::create(&config.journal_path, 0))
        .expect("Failed to open or create journal");

    // 2. AstraKernel::new()
    let limits =
        create_default_risk_engine(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));
    let mut kernel = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)));

    // 3. ExecutionGateway::new(journal)
    let gateway = Arc::new(Mutex::new(ExecutionGateway::new(journal)));

    // 4. Wrap telemetry in Arc<OperationalTelemetry>
    let tel_a = Arc::clone(&telemetry);
    let tel_b = Arc::clone(&telemetry);

    let gw_a = Arc::clone(&gateway);
    let gw_b = Arc::clone(&gateway);

    let node_id_clone = config.node_id.clone();

    // Task A: Kernel event loop
    let task_a = tokio::spawn(async move {
        loop {
            let event_opt = {
                let mut gw = gw_a.lock().await;
                gw.next_event()
            };

            if let Some(event) = event_opt {
                let _ = kernel.apply(&event);
                tel_a.increment_events();
                tel_a.update_kernel_hash(&kernel.state_hash());
            } else {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    });

    // Task B: HTTP server
    let task_b = tokio::spawn(async move {
        let addr: SocketAddr = format!("0.0.0.0:{}", config.http_port)
            .parse()
            .expect("invalid bind address");

        let listener = TcpListener::bind(addr)
            .await
            .unwrap_or_else(|e| panic!("Failed to bind HTTP server on {addr}: {e}"));

        println!(
            "AstraDaemon node={} listening on http://{}",
            config.node_id, addr
        );
        println!("  GET /health  → node health JSON");
        println!("  GET /metrics → Prometheus text exposition");
        println!("  POST /ingest → Inject market tick event");

        loop {
            tokio::select! {
                Ok((stream, _peer)) = listener.accept() => {
                    let tel = Arc::clone(&tel_b);
                    let nid = node_id_clone.clone();
                    let gw = Arc::clone(&gw_b);
                    tokio::spawn(async move {
                        handle_connection(stream, tel, nid, gw).await;
                    });
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("AstraDaemon: SIGINT received. Shutting down gracefully.");
                    break;
                }
            }
        }
    });

    let _ = tokio::join!(task_a, task_b);
}
