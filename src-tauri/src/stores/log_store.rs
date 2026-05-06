use crate::app_state::AppState;
use anyhow::{Context, Result};
use std::path::Path;

pub fn read_logs(state: &AppState) -> Result<String> {
    let entries = std::fs::read_dir(state.dirs().logs_dir()).with_context(|| "日志目录读取失败")?;

    for entry in entries.flatten() {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                return Ok(content);
            }
        }
    }
    anyhow::bail!("未能成功找到日志文件");
}

pub fn export_logs_to_file(path: &Path, content: &str) -> Result<()> {
    std::fs::write(&path, content).with_context(|| "导出日志失败")?;
    Ok(())
}
