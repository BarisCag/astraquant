use astra_api::config::Config;
use astra_api::demo::DemoMode;
use astra_api::server::build_router;

#[tokio::main]
async fn main() {
    let config = Config::load("config/default.toml");
    let demo = DemoMode::from_env();
    
    println!("AstraQuant Institutional API");
    println!("Mode: {}", if demo.is_demo() { "DEMO" } else { "LIVE" });
    println!("Starting on port {}", config.server.port);
    
    let app = build_router(config.clone(), demo);
    
    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", config.server.host, config.server.port)
    ).await.unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
