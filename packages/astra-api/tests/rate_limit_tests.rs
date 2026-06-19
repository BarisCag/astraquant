use astra_api::config::Config;
use astra_api::middleware::rate_limit::RateLimiter;

fn test_config() -> Config {
    let mut config = Config::load("config/default.toml");
    config.rate_limit.requests_per_minute = 100;
    config.rate_limit.burst = 20;
    config
}

#[tokio::test]
async fn test_burst_allowed() {
    let config = test_config();
    let limiter = RateLimiter::new(&config);
    let user_id = "test_user_burst";

    for _ in 0..20 {
        assert!(limiter.check(user_id).await.is_ok());
    }
    
    // 21st request should fail
    assert!(limiter.check(user_id).await.is_err());
}

#[tokio::test]
async fn test_rate_limit_enforced() {
    let mut config = test_config();
    config.rate_limit.burst = 1000; // bypass burst
    config.rate_limit.requests_per_minute = 100;
    let limiter = RateLimiter::new(&config);
    let user_id = "test_user_rpm";

    for _ in 0..100 {
        assert!(limiter.check(user_id).await.is_ok());
    }
    
    // 101st request should be rate limited
    let res = limiter.check(user_id).await;
    assert!(res.is_err());
    assert!(res.unwrap_err() > 0); // returns Retry-After
}
