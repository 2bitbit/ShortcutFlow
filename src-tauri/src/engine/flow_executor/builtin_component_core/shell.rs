use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::{Context, Result};
use tokio::process::Command;

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let metadata = ctx
        .input_data
        .metadata
        .clone()
        .with_context(|| format!("未发现任何metadata!\n当前ctx: \n{ctx:?}"))?;
    let cmd_str = metadata
        .get("command")
        .and_then(|v| v.as_str())
        .with_context(|| format!("在metadata中未发现任何cmd指令!\n当前ctx: \n{ctx:?}"))?;

    let mut cmd_builder = if cfg!(target_os = "windows") {
        let mut std_c = std::process::Command::new("cmd");
        std_c.arg("/C");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            std_c.raw_arg(cmd_str);
        }
        Command::from(std_c)
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", cmd_str]);
        c
    };

    if ctx.target_working_dir.exists() {
        cmd_builder.current_dir(ctx.target_working_dir);
    }

    use std::process::Stdio;
    cmd_builder
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd_builder
        .spawn()
        .map_err(|e| anyhow::anyhow!("启动子进程失败: {}", e))?;

    use tokio::io::AsyncWriteExt;
    if let Some(mut stdin) = child.stdin.take() {
        let input_str = serde_json::to_string(&ctx.input_data).unwrap_or_default();
        let _ = stdin.write_all(input_str.as_bytes()).await;
    }

    let timeout_duration = tokio::time::Duration::from_secs(60);

    let output_result = tokio::time::timeout(timeout_duration, child.wait_with_output()).await;

    let output = match output_result {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => anyhow::bail!("读取输出失败: {}", e),
        Err(_) => {
            anyhow::bail!("脚本执行超时 (60s) 被强制中止");
        }
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stripped = stdout.trim();
        if stripped.is_empty() {
            return Ok(DataEnvelope::default());
        }
        serde_json::from_str::<DataEnvelope>(stripped).or_else(|_e| {
            Ok(DataEnvelope {
                metadata: None,
                payload: Some(serde_json::Value::from(stripped)),
            })
        })
    } else {
        let err_str = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("组件运行报错: {}", err_str)
    }
}
