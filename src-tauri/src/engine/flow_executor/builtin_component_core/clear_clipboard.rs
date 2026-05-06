use crate::models::DataEnvelope;
use anyhow::Result;

pub async fn execute() -> Result<DataEnvelope> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text("".to_string())?;
    Ok(DataEnvelope::default())
}
