# x402 Demo(Rust · Base Sepolia)

一个 Cargo workspace,包含 x402 协议的两端:

```
agent-demo/
├── Cargo.toml          # workspace,统一依赖版本
├── .env.example        # client / server 共用的环境变量模板
├── client/             # x402-reqwest:自动为 402 接口付费
│   └── src/main.rs
└── server/             # x402-axum:提供付费接口
    └── src/
        ├── main.rs     # 路由 + 定价
        ├── config.rs   # 环境变量
        └── routes.rs   # handler
```

流程:client 请求 → server 返回 `402 Payment Required`(带支付要求)→ client 用
Base Sepolia 上的 USDC 签 EIP-3009 `transferWithAuthorization` → 带 `PAYMENT-SIGNATURE`
头重试 → server 交给 facilitator 验签并上链结算 → 返回内容。

## 准备

1. **付款钱包**(client):一个 Base Sepolia 私钥,领取测试 USDC:<https://faucet.circle.com>(选 Base Sepolia)。
   付款走 gasless 授权签名,钱包**不需要** ETH。
2. **收款地址**(server):任意另一个 EVM 地址。
3. facilitator 用公共的 `https://facilitator.x402.rs`,不需要自己部署。

## 运行

```sh
cp .env.example .env     # 填 EVM_PRIVATE_KEY(付款)和 PAY_TO(收款)

# 终端 1:服务端
cargo run -p x402-server-demo
# 终端 2:客户端
cargo run -p x402-client-demo
# 或指定接口
ENDPOINT=http://localhost:3000/premium/weather cargo run -p x402-client-demo
```

### 服务端接口

| 路径 | 价格 | 说明 |
| --- | --- | --- |
| `GET /` | 免费 | 列出付费接口 |
| `GET /protected-route` | 0.01 USDC | 返回结算信息(含 tx hash) |
| `GET /premium/weather` | 0.001 USDC | 模拟天气数据 |

### 客户端输出

分两步:先裸请求打印 402 支付要求(V1/V2 自动识别),再用带 x402 中间件的客户端
自动付款并打印结算结果头(`PAYMENT-RESPONSE`,base64,含链上 tx hash)和响应体。
若钱包没有 USDC,会打印 facilitator 的拒付原因(如 `insufficient_funds`)。

## 切到 Base 主网

- server:`USDC::base_sepolia()` 改成 `USDC::base()`(`eip155:8453`)。
- client:不需要改代码,网络、代币、金额全部由服务端 402 响应指定,私钥对应地址在主网持有 USDC 即可。
