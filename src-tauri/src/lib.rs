mod app_state;
pub mod dialogs;
mod engine;
mod models;
mod stores;

use crate::app_state::AppState;
use anyhow::Result;
use serde::Deserialize;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

/// GitHub 最新 Release 的 JSON 结构（只关心 tag_name 和 html_url）
#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

/// 检查更新：返回 (是否有更新, 最新版本号, 下载页面URL)
#[tauri::command]
async fn check_for_update() -> Result<(bool, String, String), String> {
    let current = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/repos/2bitbit/ShortcutFlow/releases/latest")
        .header("User-Agent", "ShortcutFlow-Update-Checker")
        .header("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API 返回错误: {}", resp.status()));
    }

    let release: GitHubRelease = resp
        .json()
        .await
        .map_err(|e| format!("解析 GitHub 响应失败: {e}"))?;

    let latest = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let has_update = compare_versions(latest, current) == std::cmp::Ordering::Greater;

    Ok((has_update, latest.to_string(), release.html_url))
}

/// 比较两个语义化版本号，返回 Ordering
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse =
        |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect() };
    let va = parse(a);
    let vb = parse(b);
    va.cmp(&vb)
}

#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

/// 进入轻量模式：销毁主窗口，释放 WebView 内存，保留托盘和快捷键
/// 使用 destroy() 而非 close()：close() 会触发 CloseRequested 事件，
/// 前端 onCloseRequested handler 会 preventDefault() 阻止关闭，导致窗口
/// 从未真正关闭 + WebView 内存从未释放 + 按钮卡在"正在进入...".
/// destroy() 强制销毁窗口，不触发 CloseRequested，绕过前端拦截。
///
/// force=true：无条件销毁（手动点击"立即进入"按钮）
/// force=false：仅当窗口不可见时才销毁（自动轻量 timer，防止用户在延迟期内
///   通过托盘恢复了窗口后被误杀）
#[tauri::command]
fn enter_lightweight_mode(app: tauri::AppHandle, force: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if !force && window.is_visible().unwrap_or(false) {
            log::info!("轻量模式跳过：窗口已恢复可见（用户可能通过托盘恢复）");
            return Ok(());
        }
        window.destroy().map_err(|e| e.to_string())?;
        log::info!("已进入轻量模式：主窗口已销毁，WebView 已释放");
    }
    Ok(())
}

/// 退出轻量模式：若窗口已销毁则重建，否则仅显示
#[tauri::command]
fn exit_lightweight_mode(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    use tauri::WebviewWindowBuilder;
    let window =
        WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .title("ShortcutFlow")
            .inner_size(800.0, 600.0)
            .build()
            .map_err(|e| e.to_string())?;

    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    log::info!("已退出轻量模式：主窗口已重建");
    Ok(())
}

#[tauri::command]
fn get_root_dir(state: tauri::State<'_, AppState>) -> String {
    state.dirs().root_dir().to_string_lossy().to_string()
}

#[tauri::command]
fn get_mouse_position() -> Result<(i32, i32), String> {
    use enigo::{Enigo, Mouse, Settings};
    let enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.location().map_err(|e| e.to_string())
}

/// 延迟后捕获鼠标坐标（给 SimulateMouse 组件的坐标选取器用）
#[tauri::command]
async fn capture_mouse_coords(delay_ms: u64) -> Result<(i32, i32), String> {
    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    get_mouse_position()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = app_state::AppState::new();

    

tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                        path: app_state.dirs().logs_dir().to_owned(),
                        file_name: None,
                    }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(app_state);

            // 通过自动启动运行时最小化窗口（autostart 插件传入 --minimized 标志）
            if std::env::args().any(|a| a == "--minimized") {
                let state = app.state::<AppState>();
                let store_path = state.dirs().root_dir().join("ui_settings.json");
                let should_minimize = app
                    .store(store_path)
                    .ok()
                    .and_then(|store| store.get("minimize_on_startup"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true); // 读取失败时默认最小化
                if should_minimize
                    && let Some(window) = app.get_webview_window("main")
                {
                    let _ = window.minimize();
                    log::info!("自动启动：已最小化窗口");
                }
            }

            app.state::<AppState>()
                .init_shortcut_manager(app.handle().clone());

            // 此处省略快捷键注册。
            // 原因：初始化时，快捷键注册统一由前端调用 refreash_all_flow_shortcut 处理，
            // 避免 setup 和前端刷新产生双重注册导致 on_shortcut 内部状态冲突。

            let menu = tauri::menu::MenuBuilder::new(app)
                .item(&tauri::menu::MenuItemBuilder::with_id("quit", "退出").build(app)?)
                .build()?;

            let _tray = tauri::tray::TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app: &tauri::AppHandle, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        // 轻量模式兼容：窗口可能已销毁
                        if let Some(window) = app.get_webview_window("main") {
                            if let Err(e) = window.show() {
                                log::error!("无法显示主窗口: {}", e);
                            }
                            if let Err(e) = window.set_focus() {
                                log::error!("无法聚焦主窗口: {}", e);
                            }
                        } else {
                            // 轻量模式中窗口已销毁，重建
                            use tauri::WebviewWindowBuilder;
                            match WebviewWindowBuilder::new(
                                app,
                                "main",
                                tauri::WebviewUrl::App("index.html".into()),
                            )
                            .title("ShortcutFlow")
                            .inner_size(800.0, 600.0)
                            .build()
                            {
                                Ok(w) => {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                    log::info!("轻量模式：托盘点击，主窗口已重建");
                                }
                                Err(e) => log::error!("轻量模式：重建主窗口失败: {}", e),
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 组件相关的
            stores::cmds::get_component_by_name,
            stores::cmds::get_all_components,
            stores::cmds::save_custom_component,
            stores::cmds::delete_custom_component,
            // 流相关的
            stores::cmds::get_flow_by_id,
            stores::cmds::get_all_flows,
            stores::cmds::save_flow,
            stores::cmds::delete_flow,
            // 引擎相关的
            engine::cmds::refreash_all_flow_shortcut,
            engine::cmds::run_flow_by_id,
            // 日志相关的
            stores::cmds::read_logs,
            stores::cmds::export_logs_to_file,
            // 辅助工具
            exit_app,
            get_root_dir,
            get_mouse_position,
            capture_mouse_coords,
            check_for_update,
            // 轻量模式
            enter_lightweight_mode,
            exit_lightweight_mode
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                // 自动退出（所有窗口关闭）→ 阻止，让托盘保持进程存活
                // 显式退出（exit(0)、托盘"退出"菜单）→ 放行（code = Some(0)）
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
