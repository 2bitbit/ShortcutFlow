use std::path::{Path, PathBuf};

/// 存储程序目录路径的结构体
pub struct Dirs {
    root_dir: PathBuf,
    shortcut_flows_dir: PathBuf,
    custom_components_dir: PathBuf,
    builtin_components_dir: PathBuf,
    logs_dir: PathBuf,
}

/// 实现只读访问器
impl Dirs {
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }
    pub fn shortcut_flows_dir(&self) -> &Path {
        &self.shortcut_flows_dir
    }
    pub fn custom_components_dir(&self) -> &Path {
        &self.custom_components_dir
    }
    pub fn builtin_components_dir(&self) -> &Path {
        &self.builtin_components_dir
    }
    pub fn logs_dir(&self) -> &Path {
        &self.logs_dir
    }
}

/// 关联函数 new
impl Dirs {
    /// 创建目录结构
    pub fn new() -> Self {
        let root_dir = std::env::current_exe()
            .expect("未能按预期找到当前程序的路径")
            .parent()
            .expect("未能按预期找到当前程序的父目录")
            .to_path_buf();

        let builtin_comps_dir = root_dir.join("BuiltinComponents");
        let custom_comps_dir = root_dir.join("CustomComponents");
        let flows_dir = root_dir.join("ShortcutFlows");
        let logs_dir = root_dir.join("logs");

        Self {
            root_dir,
            shortcut_flows_dir: flows_dir,
            custom_components_dir: custom_comps_dir,
            builtin_components_dir: builtin_comps_dir,
            logs_dir,
        }
    }
}
