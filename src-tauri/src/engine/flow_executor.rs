use crate::app_state::AppState;
use crate::engine::flow_executor::builtin_component_core::BuiltinComponentType;
use crate::models::{Component, DataEnvelope, Flow};
use crate::stores::custom_component_store;
use crate::stores::flow_store;
use anyhow::{Context, Result};
use std::fmt::Debug;
use std::path::PathBuf;
use tauri::AppHandle;

mod builtin_component_core;

pub async fn run_flow_by_id(
    app: AppHandle,
    state: &AppState,
    flow_id: String,
    initial_data: DataEnvelope,
) -> Result<DataEnvelope> {
    let flow = flow_store::get_flow_by_id(state, &flow_id)
        .with_context(|| format!("通过id: {flow_id} 获取流失败"))?
        .with_context(|| format!("id 为 {flow_id} 的流不存在"))?;
    run_flow(app, state, flow, initial_data).await
}

pub async fn run_flow(
    app: AppHandle,
    state: &AppState,
    flow: Flow,
    initial_data: DataEnvelope, // 保留这个形参，因为流可能作为父流中的一个子流执行。此外，不要设为Option，而是强制要求传入，以简化逻辑。
) -> Result<DataEnvelope> {
    // 流的延时
    if flow.startup_delay_ms > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(flow.startup_delay_ms)).await;
    }

    log::info!(
        "🚀 [Flow Engine] Starting Flow: '{}' (ID: {})",
        flow.display_name,
        flow.id
    );

    // 开始流的主循环
    let Some(nodes) = flow.nodes else {
        return Ok(DataEnvelope {
            // 应对流为空的情况
            ..Default::default()
        });
    };
    let flow_len = nodes.len();
    let mut output_data = initial_data;

    // 流级取消信号：任何组件都可以设置此标志来终止整条流
    // 使用 AtomicBool + Notify 双重机制：AtomicBool 保证状态不会丢失，Notify 用于唤醒 select!
    let flow_cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flow_cancel_notify = std::sync::Arc::new(tokio::sync::Notify::new());

    for (i, node) in nodes.into_iter().enumerate() {
        // 在进入节点前先检查取消标志（捕获上一次通知但 select 没选中的情况）
        if flow_cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            log::info!("🛑 [Flow Engine] Flow cancelled before node {}/{}", i + 1, flow_len);
            return Ok(DataEnvelope::default());
        }
        // 接受信封，然后拆信封。
        // 接收信封。
        // 拆信封环节留给组件自己的逻辑。
        let mut input_data = output_data;
        let input_data_bak = if node.pass_through {
            Some(input_data.clone())
        } else {
            None
        };
        // 组件的延时
        if let Some(delay_before_ms) = node.delay_before_ms {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_before_ms)).await;
        }

        let comp = custom_component_store::get_component_by_name(state, &node.component_name)
            .with_context(|| format!("未能通过名字获取组件({})", node.component_name))?;

        // 核心合并逻辑：comp.default_config <- node.config <- input_data.metadata
        // 先收集 node.config 的键名，用于冲突检测（只与用户显式设置冲突，不拦 default_config）
        let node_config_keys: std::collections::HashSet<String> = node
            .config
            .as_ref()
            .and_then(|v| v.as_object())
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();

        let mut merged_config = serde_json::Map::new();
        if let Some(serde_json::Value::Object(default_cfg)) = comp.default_config.clone() {
            for (k, v) in default_cfg {
                merged_config.insert(k, v);
            }
        }
        if let Some(serde_json::Value::Object(node_cfg)) = node.config.clone() {
            for (k, v) in node_cfg {
                merged_config.insert(k, v);
            }
        }
        if let Some(serde_json::Value::Object(meta_obj)) = input_data.metadata.clone() {
            for (k, v) in meta_obj {
                if node_config_keys.contains(&k) {
                    anyhow::bail!(
                        "合并配置冲突: 节点配置与上游传递的 metadata 中都存在键 `{}`",
                        k
                    );
                }
                merged_config.insert(k, v);
            }
        }

        // 自动解析 textarea 序列化的 JSON 字符串（如 "[{...}]" → 数组）
        for (_, v) in merged_config.iter_mut() {
            if let Some(s) = v.as_str() {
                let trimmed = s.trim();
                if (trimmed.starts_with('{') || trimmed.starts_with('['))
                    && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        *v = parsed;
                    }
            }
        }

        input_data.metadata = if merged_config.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(merged_config))
        };

        let ctx = ExecutionContext {
            target_working_dir: resolve_node_cwd(state, &comp, &flow.cwd).with_context(|| {
                format!(
                    "解析流(id:{:?} ,name: {:?})的工作路径({:?})失败",
                    flow.id, flow.display_name, flow.cwd
                )
            })?,
            input_data,
            flow_cancelled: Some(flow_cancelled.clone()),
            flow_cancel_notify: Some(flow_cancel_notify.clone()),
        };

        log::info!(
            "⏳ [Node {}/{}] Executing component '{}' ...",
            i + 1,
            flow_len,
            node.component_name,
        );

        // 用 select! 允许组件通过 notify + AtomicBool 中途取消整条流
        let node_result = tokio::select! {
            result = execute_single_node(app.clone(), state, comp, ctx) => result,
            _ = flow_cancel_notify.notified() => {
                log::info!("🛑 [Flow Engine] Flow cancelled during node execution");
                return Ok(DataEnvelope::default());
            }
        };

        // select 选了节点分支但取消标志可能已被设置（竞态窗口），再查一次
        if flow_cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            log::info!("🛑 [Flow Engine] Flow cancelled after node completion");
            return Ok(DataEnvelope::default());
        }

        match node_result {
            Ok(output_data_envelop) => {
                log::info!(
                    "✅ [Node {}/{}] Completed '{}'",
                    i + 1,
                    flow_len,
                    node.component_name
                );
                output_data = input_data_bak.unwrap_or(output_data_envelop);
            }
            Err(e) => {
                let err_msg = format!("Error executing component {}: {}", node.component_name, e);
                log::error!("❌ {}", err_msg);
                anyhow::bail!("{}", err_msg);
            }
        }
    }

    log::info!(
        "🎉 [Flow Engine] Flow '{}' finished successfully.",
        flow.display_name
    );
    Ok(output_data)
}

pub async fn refreash_all_flow_shortcut(state: &AppState) -> Result<()> {
    state
        .shortcut_manager()
        .remove_all()
        .with_context(|| "未能移除所有流的快捷键")?;

    let flows = flow_store::get_all_flows(state).with_context(|| "未能获取所有的流")?;

    for f in flows {
        if let Some(shortcut) = f.shortcut {
            state
                .shortcut_manager()
                .add(&f.id, shortcut)
                .with_context(|| {
                    format!(
                        "未能为流(名为{},id: {})添加快捷键({})",
                        f.display_name, f.id, shortcut
                    )
                })?;
        }
    }

    Ok(())
}

// 定义统一的执行上下文
#[derive(Debug)]
pub struct ExecutionContext {
    pub target_working_dir: PathBuf,
    pub input_data: DataEnvelope,
    /// 设置此标志为 true 将取消整条流的执行
    pub flow_cancelled: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// 通知流执行器检查取消标志（与 flow_cancelled 配对使用）
    pub flow_cancel_notify: Option<std::sync::Arc<tokio::sync::Notify>>,
}

async fn execute_single_node(
    app: AppHandle,
    state: &AppState,
    target_component: Component,
    ctx: ExecutionContext,
) -> Result<DataEnvelope> {
    let comp_type = target_component
        .name
        .parse::<BuiltinComponentType>()
        .unwrap_or(BuiltinComponentType::Shell); // 保底用 Shell，作为自定义组件触发。

    builtin_component_core::run_component_logic(app, state, comp_type, ctx).await
}

fn resolve_node_cwd(
    app_state: &AppState,
    comp: &Component,
    flow_cwd_opt: &Option<PathBuf>,
) -> Result<PathBuf> {
    let default_cwd = if comp.is_builtin {
        app_state.dirs().builtin_components_dir().join(&comp.name)
    } else {
        app_state.dirs().custom_components_dir().join(&comp.name)
    };

    if let Some(flow_cwd_path) = flow_cwd_opt {
        if flow_cwd_path.is_absolute() {
            Ok(flow_cwd_path.to_path_buf())
        } else {
            anyhow::bail!("流的工作目录不是合法的绝对路径!");
        }
    } else {
        Ok(default_cwd)
    }
}
