mod config;
mod routes;

use anyhow::Result;
use axum::Router;
use axum::routing::get;
use tracing_subscriber::EnvFilter;
use x402_axum::X402Middleware;
use x402_chain_eip155::{KnownNetworkEip155, V2Eip155Exact};
use x402_types::networks::USDC;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .init();

    let cfg = Config::from_env()?;
    let x402 = X402Middleware::try_from(cfg.facilitator_url.as_str())
        .map_err(|e| anyhow::anyhow!("facilitator 配置错误: {e}"))?
        .with_base_url(cfg.base_url.clone());
    let usdc = USDC::base_sepolia();

    // 一个路由只能挂同一协议版本的 price tag(V1/V2 类型不同),这里用 V2
    let protected_price =
        x402.with_price_tag(V2Eip155Exact::price_tag(cfg.pay_to, usdc.parse("0.01")?));
    let weather_price =
        x402.with_price_tag(V2Eip155Exact::price_tag(cfg.pay_to, usdc.parse("0.001")?));

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/protected-route", get(routes::protected).layer(protected_price))
        .route("/premium/weather", get(routes::weather).layer(weather_price));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", cfg.port)).await?;
    tracing::info!("收款地址: {}", cfg.pay_to);
    tracing::info!("facilitator: {}", x402.facilitator_url());
    tracing::info!("监听 http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
