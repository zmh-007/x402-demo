use std::env;
use std::sync::Arc;

use alloy_signer_local::PrivateKeySigner;
use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use x402_chain_eip155::{V1Eip155ExactClient, V2Eip155ExactClient};
use x402_reqwest::{ReqwestWithPayments, ReqwestWithPaymentsBuild, X402Client, parse_payment_required};
use x402_types::proto::PaymentRequired;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let endpoint =
        env::var("ENDPOINT").unwrap_or_else(|_| "http://localhost:3000/protected-route".to_string());
    let signer: PrivateKeySigner = env::var("EVM_PRIVATE_KEY")
        .context("EVM_PRIVATE_KEY 未设置(请在 .env 中配置 Base Sepolia 钱包私钥)")?
        .parse()
        .context("EVM_PRIVATE_KEY 不是合法的 secp256k1 私钥")?;
    println!("付款地址: {}", signer.address());
    println!("目标接口: {endpoint}\n");

    // 第一步:不带支付直接请求,观察服务端返回的 402 支付要求
    println!("== 第一步:裸请求,查看 402 支付要求 ==");
    let plain = Client::new().get(&endpoint).send().await?;
    println!("状态码: {}", plain.status());
    if plain.status() == StatusCode::PAYMENT_REQUIRED {
        match parse_payment_required(plain).await {
            Some(PaymentRequired::V1(v1)) => {
                println!("协议版本: x402 V1(JSON body)");
                println!("支付要求:\n{}", serde_json::to_string_pretty(&v1)?);
            }
            Some(PaymentRequired::V2(v2)) => {
                println!("协议版本: x402 V2(PAYMENT-REQUIRED header)");
                println!("支付要求:\n{}", serde_json::to_string_pretty(&v2)?);
            }
            None => println!("无法解析 402 响应中的支付要求"),
        }
    } else {
        println!("响应体: {}", plain.text().await?);
        println!("该接口未要求付费,直接结束。");
        return Ok(());
    }

    // 第二步:带 x402 中间件的客户端,自动签 USDC transferWithAuthorization 并重试
    println!("\n== 第二步:自动支付并重试 ==");
    let signer = Arc::new(signer);
    let x402_client = X402Client::new()
        .register(V1Eip155ExactClient::new(signer.clone()))
        .register(V2Eip155ExactClient::new(signer));
    let paying_client = Client::new().with_payments(x402_client).build();

    let response = paying_client.get(&endpoint).send().await?;
    println!("状态码: {}", response.status());

    // 付款后仍返回 402 = facilitator 拒绝结算(常见:钱包无 USDC),原因在 error 字段
    if response.status() == StatusCode::PAYMENT_REQUIRED {
        let reason = match parse_payment_required(response).await {
            Some(PaymentRequired::V1(v1)) => v1.error,
            Some(PaymentRequired::V2(v2)) => v2.error,
            None => None,
        };
        println!("支付被拒: {}", reason.unwrap_or_else(|| "<未给出原因>".into()));
        return Ok(());
    }

    // V1 用 X-PAYMENT-RESPONSE,V2 用 PAYMENT-RESPONSE,内容是 base64 的结算结果(含链上 tx hash)
    for name in ["PAYMENT-RESPONSE", "X-PAYMENT-RESPONSE"] {
        if let Some(value) = response.headers().get(name) {
            println!("{name}: {}", value.to_str().unwrap_or("<非 UTF-8>"));
        }
    }

    println!("响应体: {}", response.text().await?);
    Ok(())
}
