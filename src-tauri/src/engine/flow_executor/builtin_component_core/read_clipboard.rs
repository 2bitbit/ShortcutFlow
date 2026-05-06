use crate::models::DataEnvelope;
use anyhow::Result;
use serde_json::Value;

pub async fn execute() -> Result<DataEnvelope> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if let Ok(text) = clipboard.get_text() {
        return Ok(DataEnvelope {
            metadata: None,
            payload: Some(Value::String(text)),
        });
    }
    Ok(DataEnvelope::default())
}
