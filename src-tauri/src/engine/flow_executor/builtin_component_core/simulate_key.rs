use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::Result;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx.input_data.metadata.unwrap_or_default();
    let interval_ms = config
        .get("interval_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(100);
    let sequence = config.get("sequence").and_then(|v| v.as_array());

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    if let Some(seq) = sequence {
        for item in seq {
            let key_str = item.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let action = item
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("Click");
            let mut tap_count = item.get("tap_count").and_then(|v| v.as_u64()).unwrap_or(1);

            if key_str.is_empty() {
                continue;
            }

            // 解析多个按键（例如 ctrl+alt+f）
            let parts: Vec<&str> = key_str.split('+').map(|s| s.trim()).collect();
            let mut keys = Vec::new();

            for p in parts.iter() {
                let k = match p.to_lowercase().as_str() {
                    "ctrl" | "control" => Key::Control,
                    "alt" => Key::Alt,
                    "shift" => Key::Shift,
                    "meta" | "win" | "command" => Key::Meta,
                    "enter" | "return" => Key::Return,
                    "space" => Key::Space,
                    "tab" => Key::Tab,
                    "backspace" => Key::Backspace,
                    "escape" | "esc" => Key::Escape,
                    "up" => Key::UpArrow,
                    "down" => Key::DownArrow,
                    "left" => Key::LeftArrow,
                    "right" => Key::RightArrow,
                    "f1" => Key::F1,
                    "f2" => Key::F2,
                    "f3" => Key::F3,
                    "f4" => Key::F4,
                    "f5" => Key::F5,
                    "f6" => Key::F6,
                    "f7" => Key::F7,
                    "f8" => Key::F8,
                    "f9" => Key::F9,
                    "f10" => Key::F10,
                    "f11" => Key::F11,
                    "f12" => Key::F12,
                    "plus" | "add" => Key::Unicode('+'),
                    "minus" | "sub" => Key::Unicode('-'),
                    "comma" => Key::Unicode(','),
                    "period" | "dot" => Key::Unicode('.'),
                    _ => {
                        let mut chars = p.chars();
                        if let Some(c) = chars.next() {
                            if chars.next().is_none() {
                                // 单个字符
                                Key::Unicode(c)
                            } else {
                                anyhow::bail!(
                                    "SimulateKey: 无法解析按键 '{}'（组合键中只允许单个字符或已知键名，如 ctrl/alt/f1 等）",
                                    p
                                );
                            }
                        } else {
                            anyhow::bail!("SimulateKey: 按键部分为空");
                        }
                    }
                };
                keys.push(k);
            }

            if keys.is_empty() {
                continue;
            }

            match action {
                "Press" | "按住" => {
                    for key in &keys {
                        let _ = enigo.key(*key, Direction::Press);
                    }
                }
                "Release" | "释放" => {
                    for key in &keys {
                        let _ = enigo.key(*key, Direction::Release);
                    }
                }
                "Click" | "点按" => {
                    if tap_count < 1 {
                        tap_count = 1;
                    }
                    for _ in 0..tap_count {
                        // 按下所有修饰键及主键
                        for key in &keys {
                            let _ = enigo.key(*key, Direction::Press);
                        }
                        // 释放所有按键（反向释放）
                        for key in keys.iter().rev() {
                            let _ = enigo.key(*key, Direction::Release);
                        }
                        if tap_count > 1 && interval_ms > 0 {
                            tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms))
                                .await;
                        }
                    }
                }
                _ => {
                    if tap_count < 1 {
                        tap_count = 1;
                    }
                    for _ in 0..tap_count {
                        for key in &keys {
                            let _ = enigo.key(*key, Direction::Press);
                        }
                        for key in keys.iter().rev() {
                            let _ = enigo.key(*key, Direction::Release);
                        }
                        if tap_count > 1 && interval_ms > 0 {
                            tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms))
                                .await;
                        }
                    }
                }
            }
            if interval_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;
            }
        }
    }

    Ok(DataEnvelope::default())
}
