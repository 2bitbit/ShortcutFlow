use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::Result;
use enigo::{Enigo, Keyboard, Settings};

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx.input_data.metadata.clone().unwrap_or_default();
    let speed_ms = config.get("speed_ms").and_then(|v| v.as_u64()).unwrap_or(0);
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    if let Some(payload) = &ctx.input_data.payload {
        if let Some(text) = payload.as_str() {
            for c in text.chars() {
                let _ = enigo.text(&c.to_string());
                if speed_ms > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(speed_ms)).await;
                }
            }
        }
    }
    Ok(DataEnvelope::default())
}
