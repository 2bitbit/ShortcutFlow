use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::Result;

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx.input_data.metadata.unwrap_or_default();
    // payload/metadata 已由引擎层自动解析（见 flow_executor.rs 合并逻辑）
    let payload = config.get("payload").cloned();
    let metadata = config.get("metadata").cloned();
    Ok(DataEnvelope { payload, metadata })
}
