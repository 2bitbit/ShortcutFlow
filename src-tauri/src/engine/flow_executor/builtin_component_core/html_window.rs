use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::{Context, Result};
use std::io::Write;
use tauri::WindowEvent;

pub async fn execute(app: tauri::AppHandle, ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx
        .input_data
        .metadata
        .with_context(|| "HtmlWindow: 未收到配置数据")?;

    let html_content = config
        .get("html_content")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            ctx.input_data
                .payload
                .as_ref()
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| {
            "<html><body style='background:#1a1a1a;color:#fff;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0'><h1>ShortcutFlow</h1></body></html>".to_string()
        });

    let x = config.get("x").and_then(|v| v.as_f64()).unwrap_or(200.0);
    let y = config.get("y").and_then(|v| v.as_f64()).unwrap_or(200.0);
    let width = config.get("width").and_then(|v| v.as_f64()).unwrap_or(600.0);
    let height = config.get("height").and_then(|v| v.as_f64()).unwrap_or(400.0);
    let blocking = config.get("blocking").and_then(|v| v.as_bool()).unwrap_or(true);
    let title = config.get("title").and_then(|v| v.as_str()).unwrap_or("ShortcutFlow HTML");
    let resizable = config.get("resizable").and_then(|v| v.as_bool()).unwrap_or(true);
    let always_on_top = config.get("always_on_top").and_then(|v| v.as_bool()).unwrap_or(false);
    let focus = config.get("focus").and_then(|v| v.as_bool()).unwrap_or(false);
    let close_aborts = config.get("close_aborts").and_then(|v| v.as_bool()).unwrap_or(true);
    let close_aborts_silent = config.get("close_aborts_silent").and_then(|v| v.as_bool()).unwrap_or(true);

    let auto_x = x < -0.5;
    let auto_y = y < -0.5;
    let auto_w = width < -0.5;
    let auto_h = height < -0.5;
    let has_auto = auto_x || auto_y || auto_w || auto_h;

    let init_w = if auto_w { 400.0 } else { width };
    let init_h = if auto_h { 300.0 } else { height };
    let init_x = if auto_x { 100.0 } else { x };
    let init_y = if auto_y { 100.0 } else { y };

    let html_content = if has_auto {
        let script = format!(
            "<script>(function(){{function r(){{try{{var t=window.__TAURI__;if(!t||!t.window)return;\
            var bw={aw}?Math.max(document.body.scrollWidth,document.documentElement.scrollWidth)*dpr+32:{w};\
            var bh={ah}?Math.max(document.body.scrollHeight,document.documentElement.scrollHeight)*dpr+16:{h};\
            t.window.setSize(new t.window.Size(bw,bh));\
            var dpr=window.devicePixelRatio||1;var ox={ax}?screen.availWidth*dpr-bw:{x};var oy={ay}?screen.availHeight*dpr-bh:{y};\
            t.window.setPosition(new t.window.Position(ox,oy))}}catch(e){{}}}}\
            document.readyState==='loading'?document.addEventListener('DOMContentLoaded',function(){{setTimeout(r,80)}}):r();\
            if(typeof MutationObserver!=='undefined'){{new MutationObserver(function(){{r()}})\
            .observe(document.body,{{childList:true,subtree:true,attributes:true}})}}}})();</script>",
            aw = auto_w, w = width as i32,
            ah = auto_h, h = height as i32,
            ax = auto_x, x = x as i32,
            ay = auto_y, y = y as i32,
        );
        if let Some(pos) = html_content.rfind("</body>") {
            let mut s = html_content;
            s.insert_str(pos, &script);
            s
        } else {
            html_content + &script
        }
    } else {
        html_content
    };

    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let file_name = format!("shortcutflow_html_{}.html", timestamp);
    let html_path = temp_dir.join(&file_name);
    {
        let mut file = std::fs::File::create(&html_path)
            .with_context(|| format!("HtmlWindow: 创建临时文件失败: {:?}", html_path))?;
        file.write_all(html_content.as_bytes())
            .with_context(|| "HtmlWindow: 写入 HTML 内容失败")?;
    }

    let file_url = format!("file:///{}", html_path.to_string_lossy().replace('\\', "/"));
    let url: url::Url = file_url
        .parse()
        .with_context(|| format!("HtmlWindow: 无法解析文件 URL: {}", file_url))?;

    let label = config
        .get("window_label")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("html_window_{}", timestamp));

    log::info!(
        "HtmlWindow: 创建窗口 '{}' ({}x{} @ {},{}, blocking={}, close_aborts={}, auto=w:{},h:{},x:{},y:{})",
        label, init_w as u32, init_h as u32, init_x as i32, init_y as i32,
        blocking, close_aborts, auto_w, auto_h, auto_x, auto_y,
    );

    let window = tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::External(url))
        .title(title)
        .inner_size(init_w, init_h)
        .position(init_x, init_y)
        .resizable(resizable)
        .always_on_top(always_on_top)
        .focused(focus)
        .build()
        .with_context(|| "HtmlWindow: 创建窗口失败")?;

    if !blocking {
        // 非阻塞模式：注册关闭事件监听，若 close_aborts 则关闭窗口时取消整条流
        if close_aborts {
            if let (Some(flow_cancelled), Some(flow_cancel_notify)) =
                (ctx.flow_cancelled.clone(), ctx.flow_cancel_notify.clone())
            {
                let label_for_log = label.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { .. } = event {
                        log::info!(
                            "HtmlWindow: 非阻塞窗口 '{}' 手动关闭，取消整条流",
                            label_for_log
                        );
                        flow_cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
                        flow_cancel_notify.notify_one();
                    }
                });
            }
        }
        let cleanup_path = html_path.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            let _ = std::fs::remove_file(&cleanup_path);
        });
        log::info!("HtmlWindow: 非阻塞窗口 '{}' — 已创建，用户可自行关闭", label);
        return Ok(DataEnvelope::default());
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let tx_opt = std::sync::Mutex::new(Some(tx));

    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { .. } = event {
            if let Some(tx) = tx_opt.lock().unwrap().take() {
                let _ = tx.send(());
            }
        }
    });

    log::info!("HtmlWindow: 阻塞模式 — 等待窗口关闭 '{}' ...", label);
    let _ = rx.await;
    let _ = std::fs::remove_file(&html_path);

    if close_aborts {
        if close_aborts_silent {
            log::info!("HtmlWindow: 窗口 '{}' 手动关闭，静默终止流", label);
            return Ok(DataEnvelope::default());
        }
        anyhow::bail!("HtmlWindow: 窗口 '{}' 被手动关闭，流已熔断", label);
    }

    log::info!("HtmlWindow: 窗口 '{}' 已关闭，继续执行", label);
    Ok(DataEnvelope::default())
}
