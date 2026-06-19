use axum::{routing::get, Router};
use prometheus::{Encoder, GaugeVec, Registry, TextEncoder, Opts};

#[derive(Clone)]
pub struct MetricsExporter {
    registry: Registry,
    pub var_99: GaugeVec,
    pub es_97_5: GaugeVec,
    pub greeks_delta: GaugeVec,
    pub liquidity_runway: GaugeVec,
}

impl MetricsExporter {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let registry = Registry::new();
        
        let var_99 = GaugeVec::new(Opts::new("astra_risk_var_99", "Value at Risk 99%"), &["asset"]).unwrap();
        let es_97_5 = GaugeVec::new(Opts::new("astra_risk_es_97_5", "Expected Shortfall 97.5%"), &["asset"]).unwrap();
        let greeks_delta = GaugeVec::new(Opts::new("astra_risk_greeks_delta", "Portfolio Delta"), &["asset"]).unwrap();
        let liquidity_runway = GaugeVec::new(Opts::new("astra_risk_liquidity_runway_days", "Liquidity Runway in Days"), &["currency"]).unwrap();
        
        registry.register(Box::new(var_99.clone())).unwrap();
        registry.register(Box::new(es_97_5.clone())).unwrap();
        registry.register(Box::new(greeks_delta.clone())).unwrap();
        registry.register(Box::new(liquidity_runway.clone())).unwrap();

        Self {
            registry,
            var_99,
            es_97_5,
            greeks_delta,
            liquidity_runway,
        }
    }

    pub async fn start_server(self) {
        let app = Router::new()
            .route("/metrics", get({
                let registry = self.registry.clone();
                move || async move {
                    let mut buffer = vec![];
                    let encoder = TextEncoder::new();
                    let metric_families = registry.gather();
                    encoder.encode(&metric_families, &mut buffer).unwrap();
                    String::from_utf8(buffer).unwrap()
                }
            }));

        axum::Server::bind(&"0.0.0.0:9090".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
