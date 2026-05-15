use dirs::Dirs;
use shortcut_manager::ShortcutManager;
use std::sync::OnceLock;
use tauri::AppHandle;

mod dirs;
mod shortcut_manager;

pub struct AppState {
    dirs: Dirs,
    shortcut_manager: OnceLock<ShortcutManager>,
}

/// 只读访问器
impl AppState {
    pub fn dirs(&self) -> &Dirs {
        &self.dirs
    }
    pub fn shortcut_manager(&self) -> &ShortcutManager {
        self.shortcut_manager
            .get()
            .expect("程序居然在shortcut_manager初始化前读取了它！")
    }
}

/// 关联函数
impl AppState {
    pub fn new() -> Self {
        let dirs = Dirs::new();
        Self {
            dirs,
            shortcut_manager: OnceLock::new(),
        }
    }
}

/// 用于延迟初始化的相关方法
impl AppState {
    pub fn init_shortcut_manager(&self, app_handle: AppHandle) {
        let shortcut_manager: ShortcutManager = ShortcutManager::new(app_handle);
        if self.shortcut_manager.set(shortcut_manager).is_err() {
            panic!("未能成功初始化快捷键管理器，可能是已经多次初始化");
        }
    }
}
