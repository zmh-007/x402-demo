use axum::Extension;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;
use x402_types::proto::SettleResponse;

/// 免费接口:列出所有付费接口和价格
pub async fn index() -> impl IntoResponse {
    Json(json!({
        "name": "x402 server demo",
        "network": "Base Sepolia (eip155:84532)",
        "asset": "USDC",
        "routes": {
            "/protected-route": "0.01 USDC",
            "/premium/weather": "0.001 USDC",
        },
    }))
}

/// 付费接口:通过 x402 中间件后才会执行到这里
/// `settlement` 为 Some 表示中间件已在执行前完成链上结算(含 tx hash)
pub async fn protected(Extension(settlement): Extension<Option<SettleResponse>>) -> impl IntoResponse {
    match &settlement {
        Some(s) => tracing::info!(settlement = ?s, "支付已结算"),
        None => tracing::info!("支付将在响应后结算"),
    }
    Json(json!({
        "message": "这是付费内容,感谢支付!",
        "settlement": settlement.map(|s| s.0),
    }))
}

pub async fn weather() -> impl IntoResponse {
    Json(json!({
        "city": "Shanghai",
        "temperature_c": 26,
        "condition": "sunny",
    }))
}
