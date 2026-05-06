// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use shortcutflow_lib::dialogs::pop_error_dialog_native;

fn main() -> Result<()> {
    if let Err(e) = check_single_instance() {
        eprintln!("Error checking single instance: {}", e);
        std::process::exit(1);
    } else {
        setup_panic_hook();
        shortcutflow_lib::run();
    }
    Ok(())
}
fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        // 尝试提取 panic 信息 (比如 expect 传入的字符串)
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "未知的错误"
        };

        // 获取错误发生的位置（文件名和行号）
        let location = panic_info
            .location()
            .map_or(String::from("未知位置"), |loc| {
                format!("{}:{}", loc.file(), loc.line())
            });

        let error_description = format!(
            "发生致命错误，程序即将终止。\n\n错误信息: {}\n\n位置: {}",
            msg, location
        );

        // 弹出错误对话框
        pop_error_dialog_native("致命错误 (Panic)", &error_description);
    }));
}

fn check_single_instance() -> Result<()> {
    // 保证 ShortcutFlow 全局只有一个实例
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();

    let current_pid = sysinfo::get_current_pid().unwrap();
    let mut is_already_running = false;

    for (pid, process) in sys.processes() {
        if pid != &current_pid && process.name().to_lowercase().contains("shortcutflow") {
            is_already_running = true;
        }
        if is_already_running {
            pop_error_dialog_native("程序已运行", "检测到后台 ShortcutFlow 早已正在运行。");
            std::process::exit(1);
        }
    }
    Ok(())
}
