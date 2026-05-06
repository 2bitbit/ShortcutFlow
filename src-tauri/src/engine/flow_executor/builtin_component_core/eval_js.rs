use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::{Context, Result};
use tauri::Manager;

pub async fn execute(app: tauri::AppHandle, ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx
        .input_data
        .metadata
        .with_context(|| "EvalJs: 未收到配置数据")?;

    let label = config
        .get("window_label")
        .and_then(|v| v.as_str())
        .with_context(|| "EvalJs: 缺少 'window_label' 字段")?;

    let mode = config
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("js");

    let js_code = config
        .get("js_code")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // 有效 JS 代码来源：config.js_code > upstream payload
    let payload_str = ctx
        .input_data
        .payload
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let effective_js = if js_code.trim().is_empty() {
        match mode {
            "html" => {
                // payload 是 HTML 字符串，注入为 innerHTML
                let escaped = payload_str
                    .replace('\\', "\\\\")
                    .replace('\'', "\\'")
                    .replace('\n', "\\n")
                    .replace('\r', "");
                format!("document.body.innerHTML = '{}';document.body.style.overflow='auto'", escaped)
            }
            _ => {
                // 直接当 JS 执行
                payload_str.to_string()
            }
        }
    } else {
        js_code.to_string()
    };

    if effective_js.is_empty() {
        anyhow::bail!("EvalJs: js_code 为空且上游无 payload");
    }

    let window = app
        .get_webview_window(label)
        .with_context(|| format!("EvalJs: 窗口 '{}' 不存在", label))?;

    window
        .eval(&effective_js)
        .map_err(|e| anyhow::anyhow!("EvalJs: 执行失败: {e}"))?;

    log::info!("EvalJs: 已在窗口 '{}' 中执行 JS (mode={})", label, mode);
    Ok(DataEnvelope::default())
}
