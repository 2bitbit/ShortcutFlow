//! 处理 Custom Component的增删查改
use crate::app_state::AppState;
use crate::dialogs::pop_do_nothing_dialog;
use crate::models::{Component, Flow};
use crate::stores::{custom_component_store, flow_store, log_store};
use std::collections::BTreeSet;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

/// 如果出现重名的组件，就会get失败
#[tauri::command]
pub fn get_all_components(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<BTreeSet<Component>, String> {
    custom_component_store::get_all_components(&state)
        .inspect_err(|e| {
            let title = "获取所有组件时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}

/// 如果出现重名的组件，就会get失败
#[tauri::command]
pub fn get_component_by_name(
    app: AppHandle,
    state: State<'_, AppState>,
    name: &str,
) -> Result<Component, String> {
    custom_component_store::get_component_by_name(&state, name)
        .inspect_err(|e| {
            let title = "按名字获取组件时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn save_custom_component(
    app: AppHandle,
    state: State<'_, AppState>,
    comp: Component,
) -> Result<(), String> {
    custom_component_store::save_custom_component(&state, comp)
        .inspect_err(|e| {
            let title = "保存自定义组件时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn delete_custom_component(
    app: AppHandle,
    state: State<'_, AppState>,
    comp_name: &str,
) -> Result<(), String> {
    custom_component_store::delete_custom_component(&state, comp_name)
        .inspect_err(|e| {
            let title = "删除自定义组件时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn get_all_flows(app: AppHandle, state: State<'_, AppState>) -> Result<BTreeSet<Flow>, String> {
    flow_store::get_all_flows(&state)
        .inspect_err(|e| {
            let title = "获取所有流时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn get_flow_by_id(
    app: AppHandle,
    state: State<'_, AppState>,
    flow_id: &str,
) -> Result<Option<Flow>, String> {
    flow_store::get_flow_by_id(&state, flow_id)
        .inspect_err(|e| {
            let title = "通过id获取流时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn save_flow(app: AppHandle, state: State<'_, AppState>, flow: Flow) -> Result<(), String> {
    flow_store::save_flow(&state, flow)
        .inspect_err(|e| {
            let title = "保存流时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn delete_flow(
    app: AppHandle,
    state: State<'_, AppState>,
    flow_id: &str,
) -> Result<(), String> {
    flow_store::delete_flow_by_id(&state, flow_id)
        .inspect_err(|e| {
            let title = "删除流时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn read_logs(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    log_store::read_logs(&state)
        .inspect_err(|e| {
            let title = "读取日志时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn export_logs_to_file(app: AppHandle, path: String, content: String) -> Result<(), String> {
    log_store::export_logs_to_file(path.as_ref(), &content)
        .inspect_err(|e| {
            let title = "导出日志时失败";
            let msg = format!("错误原因: {:?}", e);
            pop_do_nothing_dialog(app, MessageDialogKind::Error, &title, &msg);
        })
        .map_err(|e| format!("{:?}", e))
}
