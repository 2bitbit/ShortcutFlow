use crate::app_state::AppState;
use crate::engine::flow_executor::{ExecutionContext, run_flow};
use crate::models::DataEnvelope;
use crate::stores::flow_store;
use anyhow::{Context, Result};
use tauri::Manager;

use std::future::Future;
use std::pin::Pin;

pub fn execute<'a>(
    app_handle: tauri::AppHandle,
    ctx: ExecutionContext,
) -> Pin<Box<dyn Future<Output = Result<DataEnvelope>> + Send + 'a>> {
    Box::pin(async move {
        let config = ctx.input_data.metadata.clone().unwrap_or_default();
        let flow_id = config
            .get("flow_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("CallFlow component requires a 'flow_id' string configuration.")
            })?;

        let flow = {
            let state_opt = app_handle.try_state::<AppState>();
            if let Some(state) = state_opt {
                flow_store::get_flow_by_id(&state, flow_id)
                    .with_context(|| format!("通过id: {flow_id} 获取流失败"))?
                    .with_context(|| format!("id 为 {flow_id} 的流不存在"))?
            } else {
                anyhow::bail!("CallFlow: AppState not found");
            }
        };

        let state_opt = app_handle.try_state::<AppState>();
        let state = state_opt.ok_or_else(|| anyhow::anyhow!("CallFlow: AppState not found"))?;

        let result = run_flow(app_handle.clone(), &state, flow, ctx.input_data).await?;

        Ok(result)
    })
}
