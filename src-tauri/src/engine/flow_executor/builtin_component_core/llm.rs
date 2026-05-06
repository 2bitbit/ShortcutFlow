use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::{Context, Result};
use serde_json::{Value, json};

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx
        .input_data
        .metadata
        .with_context(|| format!("metadata为空！"))?;
    let base_url = config
        .get("base_url")
        .and_then(|v| v.as_str())
        .with_context(|| "base_url为空")?;
    let api_key = config.get("api_key").and_then(|v| v.as_str()).unwrap_or("");
    let model = config
        .get("model")
        .and_then(|v| v.as_str())
        .with_context(|| "model为空")?;
    let system_prompt = config
        .get("system_prompt")
        .and_then(|v| v.as_str())
        .with_context(|| "system_prompt为空")?;
    let client = reqwest::Client::new();
    let input_format = config
        .get("input_format")
        .and_then(|v| v.as_str())
        .with_context(|| "input_format为空")?;

    let mut messages = vec![];
    if !system_prompt.is_empty() {
        messages.push(json!({ "role": "system", "content": system_prompt }));
    }

    if let Some(payload) = ctx.input_data.payload {
        let user_message = if input_format == "image" {
            // payload 应为纯 base64 字符串（由 ScreenCapture 等组件产生）
            let b64 = payload.as_str().unwrap_or("");
            let data_url = format!("data:image/png;base64,{}", b64);
            json!({
                "role": "user",
                "content": [
                    { "type": "image_url", "image_url": { "url": data_url } },
                    { "type": "text",      "text": system_prompt }
                ]
            })
        } else {
            let text = payload
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| payload.to_string());
            json!({ "role": "user", "content": text })
        };
        messages.push(user_message);
    } else {
        anyhow::bail!("LLM: 未收到输入数据（payload 为空），无法发送 API 请求");
    }

    let req_body = json!({ "model": model, "messages": messages });

    let res = client
        .post(base_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req_body)
        .send()
        .await?;

    let res_json: Value = res.json().await?;

    if let Some(content) = res_json["choices"][0]["message"]["content"].as_str() {
        Ok(DataEnvelope {
            metadata: None,
            payload: Some(Value::String(content.to_string())),
        })
    } else {
        anyhow::bail!("LLM API Error: {}", res_json)
    }
}
