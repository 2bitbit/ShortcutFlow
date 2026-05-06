use serde::{Deserialize, Serialize};
use std::cmp::Ord;
use std::hash::{Hash, Hasher};

/// 组件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// name 作为 唯一标识符
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub requires_input: bool,
    #[serde(default)]
    pub produces_output: bool,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub default_config: Option<serde_json::Value>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub ui_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub group: Option<String>,
}
impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Component {}
impl Hash for Component {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
impl PartialOrd for Component {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Component {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}
