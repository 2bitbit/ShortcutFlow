use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx
        .input_data
        .metadata
        .with_context(|| "HttpRequest: 未收到配置数据")?;

    let url = config
        .get("url")
        .and_then(|v| v.as_str())
        .with_context(|| "HttpRequest: url 配置项为空")?;

    let method = config
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .to_uppercase();

    let timeout_secs = config
        .get("timeout_secs")
        .and_then(|v| v.as_u64())
        .unwrap_or(30);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .with_context(|| "HttpRequest: 构建 HTTP 客户端失败")?;

    // 解析 headers
    let headers_str = config
        .get("headers")
        .and_then(|v| v.as_str())
        .unwrap_or("{}");
    let headers_map: HashMap<String, String> =
        serde_json::from_str(headers_str).unwrap_or_else(|_| HashMap::new());

    let body_str = config.get("body").and_then(|v| v.as_str()).unwrap_or("");

    let mut request = match method.as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        other => anyhow::bail!("HttpRequest: 不支持的 HTTP 方法: {}", other),
    };

    for (key, value) in &headers_map {
        request = request.header(key.as_str(), value.as_str());
    }

    if !body_str.is_empty() && (method == "POST" || method == "PUT") {
        request = request.body(body_str.to_string());
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("HttpRequest: 请求 {} {} 失败", method, url))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .with_context(|| format!("HttpRequest: 读取响应体失败 (状态码: {})", status))?;

    if !status.is_success() {
        log::warn!("HttpRequest: 响应状态码 {} (URL: {})", status.as_u16(), url);
    }

    Ok(DataEnvelope {
        metadata: Some(serde_json::json!({
            "http_status": status.as_u16(),
            "http_url": url,
        })),
        payload: Some(Value::String(response_text)),
    })
}
