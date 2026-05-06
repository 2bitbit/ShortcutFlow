use serde::{Deserialize, Serialize};
use std::cmp::Ord;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tauri_plugin_global_shortcut::Shortcut;

/// 数据传输格式
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DataEnvelope {
    pub metadata: Option<serde_json::Value>,
    pub payload: Option<serde_json::Value>,
}

/// 流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub startup_delay_ms: u64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub shortcut: Option<Shortcut>,
    #[serde(default)]
    pub nodes: Option<Vec<Node>>,
    #[serde(default)]
    pub cwd: Option<PathBuf>,
}
impl PartialEq for Flow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Flow {}
impl Hash for Flow {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialOrd for Flow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Flow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.display_name.cmp(&other.display_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// 用于在流里面唯一标识一个节点
    pub id: String,
    pub component_name: String, // e.g., "ScreenCapture", "LLM", "Paste"
    #[serde(default)]
    pub pass_through: bool,
    #[serde(default)]
    pub delay_before_ms: Option<u64>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}
