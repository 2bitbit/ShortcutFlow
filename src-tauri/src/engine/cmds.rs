use crate::app_state::AppState;
use crate::dialogs::pop_do_nothing_dialog;
use crate::engine::flow_executor;
use crate::models::DataEnvelope;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

#[tauri::command]
pub async fn run_flow_by_id(
    app: AppHandle,
    state: State<'_, AppState>,
    flow_id: String,
    initial_data: DataEnvelope,
) -> Result<(), String> {
    match flow_executor::run_flow_by_id(app.clone(), &state, flow_id, initial_data).await {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("❌flow execution failed: {:?}", e);
            let title = "ShortcutFlow 运行错误";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, title, &msg);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn refreash_all_flow_shortcut(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    flow_executor::refreash_all_flow_shortcut(&state)
        .await
        .inspect_err(|e| {
            log::error!("❌刷新所有流的快捷键时失败: {:?}", e);
            let title = "ShortcutFlow 运行错误";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, title, &msg);
        })
        .map_err(|e| e.to_string())
}
