//! 处理 Custom Component的增删查改
use crate::app_state::AppState;
use crate::models::Component;
use crate::stores::utils::get_all_json_file_paths;
use anyhow::{Context, Result};
use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
/// 如果出现重名的组件，就会get失败
pub fn get_all_components(state: &AppState) -> Result<BTreeSet<Component>> {
    let all_component_json_paths: Vec<PathBuf> = get_all_json_file_paths(vec![
        state.dirs().builtin_components_dir(),
        state.dirs().custom_components_dir(),
    ])?;

    // 文件系统层面已经保证了自定义组件之间不会重名
    // 此处主要是检查自定义组件是否与某个内置组件重名
    let mut names = HashSet::new();
    for path in &all_component_json_paths {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(anyhow::anyhow!("文件名包含非 UTF-8 字符"))?;
        if !names.insert(name) {
            anyhow::bail!("发现自定义组件{name}与内置组件重名");
        }
    }
    let all_comps: BTreeSet<Component> = all_component_json_paths
        .into_iter()
        .map(|path| {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("未能成功读取组件({path:?})"))?;
            let component = serde_json::from_str::<Component>(&content)
                .with_context(|| format!("组件({path:?})出问题了,其内容为:{content:?}"))?;
            Ok(component)
        })
        .collect::<Result<BTreeSet<Component>>>()?;

    Ok(all_comps)
}

/// 如果出现重名的组件，就会 get 失败
pub fn get_component_by_name(state: &AppState, comp_name: &str) -> Result<Component> {
    let is_builtin_component = is_builtin_component_name(state, comp_name);
    let is_custom_component = is_custom_component_name(state, comp_name);

    let component_path = match (is_builtin_component, is_custom_component) {
        (true, true) => anyhow::bail!("组件 ({comp_name}) 与内置组件重名"),
        (false, false) => anyhow::bail!("组件 ({comp_name}) 不存在"),
        // 现在，自定义组件必然存在，而且不与内置组件重名，不与其他自定义组件重名（这一点由文件系统保证）
        (true, false) => state
            .dirs()
            .builtin_components_dir()
            .join(comp_name)
            .join(format!("{comp_name}.json")),
        (false, true) => state
            .dirs()
            .custom_components_dir()
            .join(comp_name)
            .join(format!("{comp_name}.json")),
    };
    let content = std::fs::read_to_string(&component_path)
        .with_context(|| format!("读取{component_path:?}文件失败"))?;
    let component = serde_json::from_str::<Component>(&content)
        .with_context(|| format!("{component_path:?}文件的内容不合法"))?;
    Ok(component)
}

pub fn save_custom_component(state: &AppState, comp: Component) -> Result<()> {
    // 检查自定义组件是否与内置组件重名 （由于文件系统已经保证了自定义组件之间不能重名。所以只需要检查是否和内置组件重名）
    if is_builtin_component_name(state, &comp.name) {
        anyhow::bail!("存在重名的内置组件，未能成功保存");
    }

    let content = serde_json::to_string_pretty(&comp)
        .with_context(|| format!("未能成功序列化组件\"{}\"", comp.name))?;

    let target_dir = state.dirs().custom_components_dir().join(&comp.name);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)
            .with_context(|| format!("创建组件(name: {})目录失败", comp.name))?;
    }

    std::fs::write(target_dir.join(format!("{}.json", comp.name)), content)?;
    Ok(())
}

pub fn delete_custom_component(state: &AppState, comp_name: &str) -> Result<()> {
    std::fs::remove_dir_all(state.dirs().custom_components_dir().join(comp_name))?;
    Ok(())
}

/// 用于检查是否为内置组件
fn is_builtin_component_name(state: &AppState, comp_name: &str) -> bool {
    state
        .dirs()
        .builtin_components_dir()
        .join(comp_name)
        .join(format!("{}.json", comp_name))
        .exists()
}

/// 用于检查是否为自定义组件
fn is_custom_component_name(state: &AppState, comp_name: &str) -> bool {
    state
        .dirs()
        .custom_components_dir()
        .join(comp_name)
        .join(format!("{}.json", comp_name))
        .exists()
}
