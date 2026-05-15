use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let payload = ctx
        .input_data
        .payload
        .ok_or_else(|| anyhow::anyhow!("OCR Component requires Image data input"))?;
    let b64_img = payload
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("OCR Component requires string (base64) payload"))?;

    // 解码 Base64 图像
    let img_bytes = general_purpose::STANDARD
        .decode(b64_img)
        .map_err(|e| anyhow::anyhow!("Base64 decoding failed: {}", e))?;

    // 提取配置并解析为绝对路径
    let config = ctx.input_data.metadata.unwrap_or_default();
    let det_val = config
        .get("det_model")
        .and_then(|v: &Value| v.as_str())
        .with_context(|| "det_model路径为空")?;
    let rec_val = config
        .get("rec_model")
        .and_then(|v: &Value| v.as_str())
        .with_context(|| "rec_model路径为空")?;
    let keys_val = config
        .get("keys_path")
        .and_then(|v: &Value| v.as_str())
        .with_context(|| "字典路径keys_path为空")?;

    let comp_dir = ctx.target_working_dir;
    let resolve_path = |p: &str| -> String {
        let path = std::path::Path::new(p);
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            comp_dir.join(path)
        };
        let mut s = absolute_path.to_string_lossy().into_owned();
        if s.starts_with(r"\\?\") {
            s = s[4..].to_string();
        }
        s
    };

    let det_path = resolve_path(det_val);
    let rec_path = resolve_path(rec_val);
    let keys_path = resolve_path(keys_val);

    let result = tokio::task::spawn_blocking(move || -> Result<String> {
        let img = match image::load_from_memory(&img_bytes) {
            Ok(i) => i,
            Err(e) => anyhow::bail!("图片解码失败: {}", e),
        };

        // 初始化 OCR 引擎
        use ocr_rs::{OcrEngine, OcrEngineConfig};
        let engine_config = OcrEngineConfig::fast();

        let engine = match OcrEngine::new(&det_path, &rec_path, &keys_path, Some(engine_config)) {
            Ok(e) => e,
            Err(e) => anyhow::bail!(
                "OCR 引擎初始化失败，请检查您在流配置里的模型路径是否书写正确，且对应模型存在: {}",
                e
            ),
        };

        // 执行识别
        let results = match engine.recognize(&img) {
            Ok(r) => r,
            Err(e) => anyhow::bail!("OCR 识别执行失败: {}", e),
        };

        let mut lines = Vec::new();
        for r in results {
            lines.push(r.text);
        }

        Ok(lines.join("\n"))
    })
    .await
    .map_err(|e| anyhow::anyhow!("OCR 线程异常中止: {e}"))?;

    match result {
        Ok(text) => Ok(DataEnvelope {
            metadata: None,
            payload: Some(Value::String(text)),
        }),
        Err(e) => Err(e),
    }
}
