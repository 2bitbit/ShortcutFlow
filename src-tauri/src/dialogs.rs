use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::MessageDialogKind;

/// rfd 是处理程序崩溃级别的工具。
pub fn pop_error_dialog_native(title: &str, msg: &str) {
    rfd::MessageDialog::new()
        .set_title(title)
        .set_description(msg)
        .set_level(rfd::MessageLevel::Error)
        .show();
}

/// 用于处理失败，但应用还能继续运行的常规业务错误。  
///
/// 这家伙只应该在 `#[tauri::command]` 接口函数中使用，且最好是只调用一次（错误信息根据普通`rust`函数的`anyhow::bail!`来决定）。
///
/// 这个函数仅仅是用来弹窗告知，并未设计任何交互逻辑。
pub fn pop_do_nothing_dialog(app: AppHandle, kind: MessageDialogKind, title: &str, msg: &str) {
    app.dialog() // Tauri 的 dialog() API 留给那些处理失败，但应用还能继续运行的常规业务错误。
        .message(msg.to_string())
        .title(title)
        .kind(kind)
        .show(|_| {});
}
