use crate::app_state::AppState;
use crate::models::Flow;
use crate::stores::utils::get_all_json_file_paths;
use anyhow::{Context, Result};
use std::collections::BTreeSet;

pub fn get_all_flows(state: &AppState) -> Result<BTreeSet<Flow>> {
    let all_flow_json_paths = get_all_json_file_paths(vec![state.dirs().shortcut_flows_dir()])?;

    let all_flows: BTreeSet<Flow> = all_flow_json_paths
        .into_iter()
        .map(|path| {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("读取组件json: {path:?}失败"))?;
            let flow = serde_json::from_str::<Flow>(&content)
                .with_context(|| format!("组件json: {path:?}格式无效"))?;
            Ok(flow)
        })
        .collect::<Result<BTreeSet<Flow>>>()?;

    Ok(all_flows)
}

pub fn get_flow_by_id(state: &AppState, flow_id: &str) -> Result<Option<Flow>> {
    let flow_path = state
        .dirs()
        .shortcut_flows_dir()
        .join(format!("{}.json", flow_id));
    if !flow_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&flow_path)
        .with_context(|| format!("未能成功读取流文件({flow_path:?})"))?;
    let flow = serde_json::from_str::<Flow>(&content)
        .with_context(|| format!("组件json: {flow_path:?}格式无效"))?;
    Ok(Some(flow))
}

pub fn save_flow(state: &AppState, flow: Flow) -> Result<()> {
    let opt_old_flow = get_flow_by_id(state, &flow.id).with_context(|| {
        format!(
            "未能按id成功获取流(流名字: {}, 流id: {})",
            flow.display_name, flow.id
        )
    })?;

    let new_shortcut = flow.shortcut;

    // 快捷键的更新逻辑：
    // 有新快捷键的前提下：
    //     - 旧流存在   -> state.shortcut_manager().update
    //     - 旧流不存在 -> state.shortcut_manager().add
    // 在无新快捷键的前提下：
    //     - 旧流存在:
    //          - 旧流有快捷键 -> state.shortcut_manager().remove
    //          - 旧流无快捷键 -> 掠过快捷键逻辑
    //     - 旧流不存在 -> 略过快捷键逻辑
    match (new_shortcut, opt_old_flow) {
        (Some(new_shortcut), Some(old_flow)) => {
            state
                .shortcut_manager()
                .update(&old_flow.id, new_shortcut)
                .with_context(|| {
                    format!(
                        "更新流(id:{}, name: {})的快捷键时失败",
                        flow.id, flow.display_name
                    )
                })?;
        }
        (Some(new_shortcut), None) => {
            state
                .shortcut_manager()
                .add(&flow.id, new_shortcut)
                .with_context(|| {
                    format!(
                        "未能为新流(id:{}, name: {})添加快捷键",
                        flow.id, flow.display_name
                    )
                })?;
        }
        (None, Some(old_flow)) => {
            if let Some(old_shortcut) = old_flow.shortcut {
                state
                    .shortcut_manager()
                    .remove_by_shortcut(old_shortcut)
                    .with_context(|| {
                        format!(
                            "无法移除流(id:{}, name: {})原来的快捷键",
                            flow.id, flow.display_name
                        )
                    })?;
            }
        }
        (None, None) => {}
    }

    // 写入新流 json
    std::fs::write(
        state
            .dirs()
            .shortcut_flows_dir()
            .join(format!("{}.json", &flow.id)),
        serde_json::to_string_pretty(&flow)
            .expect("什么！我自己的flow结构体未能序列化成功！？严重失败！"),
    )
    .with_context(|| {
        format!(
            "未能成功将流写入{}.json文件 (id: {})",
            flow.display_name, flow.id
        )
    })?;

    Ok(())
}

pub fn delete_flow_by_id(state: &AppState, flow_id: &str) -> Result<()> {
    let flow = get_flow_by_id(state, &flow_id)
        .with_context(|| format!("未能按id成功获取流(流id: {})", flow_id))?;

    let Some(flow) = flow else {
        anyhow::bail!("原来的流文件不存在");
    };

    if let Some(shortcut) = flow.shortcut {
        state
            .shortcut_manager()
            .remove_by_shortcut(shortcut)
            .with_context(|| {
                format!(
                    "未能为流(id: {}, name: {})删除快捷键",
                    flow.id, flow.display_name
                )
            })?;
    }

    std::fs::remove_file(
        state
            .dirs()
            .shortcut_flows_dir()
            .join(format!("{}.json", flow_id)),
    )
    .with_context(|| {
        format!(
            "未能删除流文件 (id: {},name: {})",
            flow.id, flow.display_name
        )
    })?;

    Ok(())
}
