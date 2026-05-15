use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::{Context, Result};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub async fn execute(app: tauri::AppHandle, ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx
        .input_data
        .metadata
        .with_context(|| "KeyListener: 未收到配置数据")?;

    let timeout_secs = config
        .get("timeout_secs")
        .and_then(|v| v.as_u64())
        .unwrap_or(120);

    // keys 已被引擎层自动解析为数组（见 flow_executor.rs 合并逻辑）
    let keys_array: Vec<Value> = config
        .get("keys")
        .and_then(|v| v.as_array())
        .cloned()
        .with_context(|| "KeyListener: keys 配置缺失或格式错误")?;

    if keys_array.is_empty() {
        anyhow::bail!("KeyListener: keys 配置为空，至少需要定义一个按键");
    }

    // 解析每个按键映射
    struct KeyMapping {
        shortcut: Shortcut,
        shortcut_str: String,
        metadata: Option<Value>,
        payload: Option<Value>,
    }

    let mut mappings = Vec::new();
    for entry in &keys_array {
        let key_str = entry
            .get("key")
            .and_then(|v| v.as_str())
            .with_context(|| format!("KeyListener: 按键项缺少 'key' 字段: {}", entry))?;

        // 将字符串解析为 Shortcut；通过 JSON 反序列化
        let shortcut: Shortcut = serde_json::from_value(Value::String(key_str.to_string()))
            .with_context(|| format!("KeyListener: 无法解析快捷键 '{}'", key_str))?;

        let metadata = entry.get("metadata").cloned();
        let payload = entry.get("payload").cloned();

        mappings.push(KeyMapping {
            shortcut,
            shortcut_str: key_str.to_string(),
            metadata,
            payload,
        });
    }

    log::info!(
        "KeyListener: 注册 {} 个按键监听 (超时: {}s)...",
        mappings.len(),
        timeout_secs
    );
    for m in &mappings {
        log::info!("  - {}", m.shortcut_str);
    }

    // 共享状态：被触发的按键索引（usize::MAX = Esc 取消）
    let triggered: Arc<Mutex<Option<usize>>> = Arc::new(Mutex::new(None));
    let cancelled: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    // 注册所有快捷键
    let gs = app.global_shortcut();

    // RAII 守卫：无论函数如何退出（正常/key/超时/Esc/流被外部取消），必定清理所有快捷键
    struct ShortcutGuard {
        app: tauri::AppHandle,
        registered: Vec<Shortcut>,
    }
    impl Drop for ShortcutGuard {
        fn drop(&mut self) {
            let gs = self.app.global_shortcut();
            for sc in &self.registered {
                let _ = gs.unregister(*sc);
            }
        }
    }
    let mut guard = ShortcutGuard {
        app: app.clone(),
        registered: Vec::new(),
    };

    for (i, mapping) in mappings.iter().enumerate() {
        let triggered_clone = triggered.clone();
        let idx = i;
        let shortcut_str = mapping.shortcut_str.clone();

        gs.on_shortcut(mapping.shortcut, move |_app, _sc, event| {
            if event.state == ShortcutState::Pressed {
                log::info!("KeyListener: 按键 '{}' 被按下", shortcut_str);
                let mut guard = triggered_clone.lock().unwrap();
                if guard.is_none() {
                    *guard = Some(idx);
                }
            }
        })
        .with_context(|| format!("KeyListener: 注册快捷键 '{}' 失败", mapping.shortcut_str))?;
        guard.registered.push(mapping.shortcut);
    }

    // Esc 处理：若已在 keys 中则由用户配置接管，否则注册内置取消逻辑
    let has_esc_key = mappings.iter().any(|m| m.shortcut_str.eq_ignore_ascii_case("Escape"));
    if !has_esc_key {
        let esc: Shortcut = serde_json::from_value(Value::String("Escape".to_string()))
            .with_context(|| "KeyListener: 无法解析 Esc")?;
        let cancelled_clone = cancelled.clone();
        gs.on_shortcut(esc, move |_app, _sc, event| {
            if event.state == ShortcutState::Pressed {
                log::info!("KeyListener: Esc 取消（不在 keys 中）");
                *cancelled_clone.lock().unwrap() = true;
            }
        })
        .with_context(|| "KeyListener: 注册 Esc 快捷键失败")?;
        guard.registered.push(esc);
    } else {
        log::info!("KeyListener: Esc 在 keys 中，跳过内置 Esc 取消逻辑");
    }

    // 窗口关闭监听：若配置了 window_label，则监视该窗口的关闭事件作为取消信号
    let window_label = config
        .get("window_label")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    if let Some(ref label) = window_label {
        if let Some(window) = app.get_webview_window(label) {
            let cancelled_clone = cancelled.clone();
            let label_for_log = label.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    log::info!("KeyListener: 窗口 '{}' 关闭，取消监听", label_for_log);
                    *cancelled_clone.lock().unwrap() = true;
                }
            });
        } else {
            log::warn!("KeyListener: window_label '{}' 指定的窗口不存在，跳过窗口关闭监听", label);
        }
    }

    // 等待触发或超时（timeout_secs==0 时自动设为 300s 上限，防止取消流时快捷键永久泄漏）
    let effective_timeout = if timeout_secs == 0 { 300 } else { timeout_secs };
    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(effective_timeout);
    let mut result = None;
    let mut esc_cancelled = false;
    loop {
        if *cancelled.lock().unwrap() {
            esc_cancelled = true;
            break;
        }
        let idx = { *triggered.lock().unwrap() };
        if let Some(i) = idx {
            result = Some(i);
            break;
        }
        if tokio::time::Instant::now() >= deadline {
            log::warn!("KeyListener: 超时 ({}s)，取消监听", effective_timeout);
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    let triggered_idx = result;

    // Esc 取消：立即报错退出
    if esc_cancelled {
        anyhow::bail!("KeyListener: 用户按 Esc 取消");
    }

    // 返回结果
    match triggered_idx {
        Some(idx) => {
            let m = &mappings[idx];
            log::info!(
                "KeyListener: 返回按键 '{}' 的 metadata 和 payload",
                m.shortcut_str
            );
            Ok(DataEnvelope {
                metadata: m.metadata.clone(),
                payload: m.payload.clone(),
            })
        }
        None => {
            anyhow::bail!(
                "KeyListener: 超时 ({}s)，未检测到任何已注册的按键",
                effective_timeout
            );
        }
    }
}
