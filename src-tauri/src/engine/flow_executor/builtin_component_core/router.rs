use crate::app_state::AppState;
use crate::engine::flow_executor::{ExecutionContext, run_flow};
use crate::models::DataEnvelope;
use crate::stores::flow_store;
use anyhow::{Context, Result};
use regex::Regex;
use tauri::Manager;

use std::future::Future;
use std::pin::Pin;

pub fn execute<'a>(
    app_handle: tauri::AppHandle,
    ctx: ExecutionContext,
) -> Pin<Box<dyn Future<Output = Result<DataEnvelope>> + Send + 'a>> {
    Box::pin(async move {
        let config = ctx.input_data.metadata.clone().unwrap_or_default();

        // rules 已被引擎层自动解析为数组（见 flow_executor.rs 合并逻辑）
        let rules: Vec<serde_json::Value> = config
            .get("rules")
            .and_then(|v| v.as_array())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Router: 'rules' 配置缺失或格式错误"))?;

        let default_subflow_id = config
            .get("default_subflow_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let payload_str = ctx
            .input_data
            .payload
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 顺序匹配规则
        for rule in rules {
            let condition = rule
                .get("condition")
                .ok_or_else(|| anyhow::anyhow!("Router: 规则缺少 'condition' 字段"))?;

            let condition_type = condition
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Router: condition 缺少 'type' 字段"))?;

            let matched = match condition_type {
                "regex" => {
                    let pattern = condition
                        .get("pattern")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Router: regex condition 缺少 'pattern'"))?;
                    Regex::new(pattern)
                        .with_context(|| format!("Router: 正则表达式无效: {pattern}"))?
                        .is_match(payload_str)
                }
                "char_count_lte" => {
                    let value = condition
                        .get("value")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| {
                            anyhow::anyhow!("Router: char_count_lte condition 缺少 'value'")
                        })? as usize;
                    payload_str.chars().count() <= value
                }
                "char_count_gt" => {
                    let value = condition
                        .get("value")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| {
                            anyhow::anyhow!("Router: char_count_gt condition 缺少 'value'")
                        })? as usize;
                    payload_str.chars().count() > value
                }
                other => anyhow::bail!("Router: 不支持的条件类型: {other}"),
            };

            if matched {
                let subflow_id = rule
                    .get("subflow_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Router: 命中的规则缺少 'subflow_id' 字段")
                    })?;

                log::info!(
                    "Router: 规则命中 (type={condition_type}) → 执行子流: {subflow_id}"
                );

                let state = app_handle
                    .try_state::<AppState>()
                    .ok_or_else(|| anyhow::anyhow!("Router: AppState not found"))?;

                let flow = flow_store::get_flow_by_id(&state, subflow_id)
                    .with_context(|| format!("Router: 子流不存在: {subflow_id}"))?
                    .with_context(|| format!("Router: 子流不存在: {subflow_id}"))?;

                let child_input = DataEnvelope {
                    payload: ctx.input_data.payload.clone(),
                    metadata: None,
                };

                return run_flow(app_handle.clone(), &state, flow, child_input).await;
            }
        }

        // 无规则命中 → 走默认子流
        if let Some(ref default_id) = default_subflow_id {
            log::info!("Router: 无规则命中 → 执行默认子流: {default_id}");

            let state = app_handle
                .try_state::<AppState>()
                .ok_or_else(|| anyhow::anyhow!("Router: AppState not found"))?;

            let flow = flow_store::get_flow_by_id(&state, default_id)
                .with_context(|| format!("Router: 默认子流不存在: {default_id}"))?
                .with_context(|| format!("Router: 默认子流不存在: {default_id}"))?;

            let child_input = DataEnvelope {
                payload: ctx.input_data.payload.clone(),
                metadata: None,
            };

            return run_flow(app_handle.clone(), &state, flow, child_input).await;
        }

        // 无默认子流 → 透传
        log::info!("Router: 无规则命中且无默认子流 → 透传");
        Ok(ctx.input_data)
    })
}
