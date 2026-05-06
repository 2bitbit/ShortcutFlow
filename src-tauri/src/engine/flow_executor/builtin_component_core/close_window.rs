use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::{Context, Result};
use tauri::Manager;

pub async fn execute(app: tauri::AppHandle, ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx
        .input_data
        .metadata
        .clone()
        .with_context(|| "CloseWindow: 未收到配置数据")?;

    let label = config
        .get("label")
        .and_then(|v| v.as_str())
        .with_context(|| "CloseWindow: 缺少 'label' 字段")?;

    if let Some(window) = app.get_webview_window(label) {
        window.destroy().map_err(|e| anyhow::anyhow!("{e}"))?;
        log::info!("CloseWindow: 已销毁窗口 '{}'", label);
    } else {
        log::warn!("CloseWindow: 窗口 '{}' 不存在，可能已被关闭", label);
    }

    Ok(DataEnvelope::default())
}
