use astra_venue::feed::LiveFeedManager;

#[tokio::main]
async fn main() {
    println!("AstraQuant Trade — Live Feed Starting...");
    
    let mut manager = LiveFeedManager::new("live_journal.astra_jl");
    
    println!("Connecting to Binance BTCUSDT...");
    manager.run("btcusdt").await.unwrap();
}
