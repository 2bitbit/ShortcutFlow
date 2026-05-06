use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::Result;

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    if let Some(payload) = ctx.input_data.payload {
        if let Some(text) = payload.as_str() {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(text.to_string());
            }
            return Ok(DataEnvelope::default());
        }
    }

    anyhow::bail!("WriteClipboard Component requires Text data input");
}
