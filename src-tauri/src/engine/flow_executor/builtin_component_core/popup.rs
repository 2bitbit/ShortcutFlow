use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::Result;
use tauri_plugin_dialog::DialogExt;

pub async fn execute(app: tauri::AppHandle, ctx: ExecutionContext) -> Result<DataEnvelope> {
    let msg = match ctx.input_data.payload {
        Some(payload) => {
            if let Some(t) = payload.as_str() {
                t.to_string()
            } else {
                format!("原始Json为{}", payload)
            }
        }
        None => anyhow::bail!("没有数据用来显示"),
    };

    let display_text = format!("来自内置 popup 组件的输入: \n{}", msg);

    app.dialog()
        .message(display_text)
        .title("ShortcutFlow")
        .show(|_| {});

    Ok(DataEnvelope::default())
}
