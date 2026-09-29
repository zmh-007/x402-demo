use std::env;

use alloy_primitives::Address;
use anyhow::{Context, Result};
use url::Url;

pub struct Config {
    /// 收款地址(Base Sepolia 上接收 USDC 的钱包)
    pub pay_to: Address,
    /// facilitator:代服务端验签 + 上链结算的服务
    pub facilitator_url: String,
    /// 对外访问地址,写入 402 响应的 resource.url
    pub base_url: Url,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let pay_to = env::var("PAY_TO")
            .context("PAY_TO 未设置(收款钱包地址)")?
            .parse()
            .context("PAY_TO 不是合法的 EVM 地址")?;
        let facilitator_url =
            env::var("FACILITATOR_URL").unwrap_or_else(|_| "https://facilitator.x402.rs".to_string());
        let port = env::var("PORT")
            .ok()
            .map(|p| p.parse::<u16>().context("PORT 不是合法端口"))
            .transpose()?
            .unwrap_or(3000);
        let base_url = env::var("BASE_URL")
            .unwrap_or_else(|_| format!("http://localhost:{port}"))
            .parse()
            .context("BASE_URL 不是合法 URL")?;
        Ok(Self { pay_to, facilitator_url, base_url, port })
    }
}
