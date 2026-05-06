use crate::models::DataEnvelope;
use anyhow::Result;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::process::Command;

/// 检测前台窗口是否为 Windows Terminal（需要 Ctrl+Shift+C 而非 Ctrl+C）
#[cfg(target_os = "windows")]
fn is_terminal_window() -> bool {
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId,
        };

        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return false;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return false;
        }

        // 用 PowerShell 查进程名（比加 Win32 API feature 更简洁）
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-Process -Id {}).ProcessName", pid),
            ])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let name = String::from_utf8_lossy(&o.stdout).trim().to_lowercase();
                name == "windowsterminal"
            }
            _ => false,
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn is_terminal_window() -> bool {
    false
}

pub async fn execute() -> Result<DataEnvelope> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Release any potentially stuck/pressed physical modifiers to ensure clean output
    let _ = enigo.key(Key::Alt, Direction::Release);
    let _ = enigo.key(Key::Shift, Direction::Release);
    let _ = enigo.key(Key::Meta, Direction::Release);
    let _ = enigo.key(Key::Control, Direction::Release);

    let use_shift = is_terminal_window();

    let _ = enigo.key(Key::Control, Direction::Press);
    if use_shift {
        let _ = enigo.key(Key::Shift, Direction::Press);
    }
    let _ = enigo.key(Key::C, Direction::Click);
    if use_shift {
        let _ = enigo.key(Key::Shift, Direction::Release);
    }
    let _ = enigo.key(Key::Control, Direction::Release);

    // Wait a bit for clipboard to populate
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    Ok(DataEnvelope::default())
}
