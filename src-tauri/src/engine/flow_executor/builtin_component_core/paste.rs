use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::{Context, Result};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let payload = ctx
        .input_data
        .payload
        .with_context(|| "Paste: 未收到上游数据")?;

    let text = payload
        .as_str()
        .with_context(|| "Paste: payload 不是字符串类型，无法粘贴")?;

    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(text.to_string());
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Release any potentially stuck/pressed physical modifiers to ensure clean output
    let _ = enigo.key(Key::Alt, Direction::Release);
    let _ = enigo.key(Key::Shift, Direction::Release);
    let _ = enigo.key(Key::Meta, Direction::Release);
    let _ = enigo.key(Key::Control, Direction::Release);

    let _ = enigo.key(Key::Control, Direction::Press);
    let _ = enigo.key(Key::V, Direction::Click);
    let _ = enigo.key(Key::Control, Direction::Release);

    Ok(DataEnvelope::default())
}
