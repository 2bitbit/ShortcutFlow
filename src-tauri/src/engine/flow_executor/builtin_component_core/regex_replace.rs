use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::Result;
use regex::RegexBuilder;
use serde_json::Value;

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let payload = ctx
        .input_data
        .payload
        .ok_or_else(|| anyhow::anyhow!("Regex Component requires Text data input"))?;
    let text = payload
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Regex Component requires Text data input"))?
        .to_string();

    let config = ctx.input_data.metadata.unwrap_or_default();
    let pattern = config.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
    let replacement = config
        .get("replacement")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let case_insensitive = config
        .get("case_insensitive")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let multi_line = config
        .get("multi_line")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let dot_matches_new_line = config
        .get("dot_matches_new_line")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if pattern.is_empty() {
        return Ok(DataEnvelope {
            metadata: None,
            payload: Some(Value::String(text)),
        }); // Do nothing
    }

    let re = RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .multi_line(multi_line)
        .dot_matches_new_line(dot_matches_new_line)
        .build()
        .map_err(|e| anyhow::anyhow!("正则表达式格式错误: {}", e))?;

    let result = re.replace_all(&text, replacement).into_owned();

    Ok(DataEnvelope {
        metadata: None,
        payload: Some(Value::String(result)),
    })
}
