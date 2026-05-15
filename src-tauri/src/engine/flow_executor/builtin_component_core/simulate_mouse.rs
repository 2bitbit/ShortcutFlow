use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::Result;
use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};

pub async fn execute(ctx: ExecutionContext) -> Result<DataEnvelope> {
    let config = ctx.input_data.metadata.unwrap_or_default();
    let interval_ms = config
        .get("interval_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(100);
    let sequence = config.get("sequence").and_then(|v| v.as_array());

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    if let Some(seq) = sequence {
        let seq_len = seq.len();
        for (i, item) in seq.iter().enumerate() {
            let button_type = item
                .get("button_type")
                .and_then(|v| v.as_str())
                .unwrap_or("Left");
            let action = item
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("Click");
            let mut tap_count = item.get("tap_count").and_then(|v| v.as_u64()).unwrap_or(1);
            let coords_str = item.get("coords").and_then(|v| v.as_str()).unwrap_or("");

            if button_type == "Move" || button_type == "移动" {
                let parts: Vec<&str> = coords_str.split(',').collect();
                if parts.len() == 2
                    && let (Ok(x), Ok(y)) = (
                        parts[0].trim().parse::<i32>(),
                        parts[1].trim().parse::<i32>(),
                    ) {
                        let is_relative_x =
                            parts[0].trim().starts_with('+') || parts[0].trim().starts_with('-');
                        let is_relative_y =
                            parts[1].trim().starts_with('+') || parts[1].trim().starts_with('-');

                        if is_relative_x || is_relative_y {
                            if let Ok((lx, ly)) = enigo.location() {
                                let final_x = if is_relative_x { lx + x } else { x };
                                let final_y = if is_relative_y { ly + y } else { y };
                                let _ = enigo.move_mouse(final_x, final_y, Coordinate::Abs);
                            } else {
                                // fallback
                                let _ = enigo.move_mouse(x, y, Coordinate::Rel);
                            }
                        } else {
                            let _ = enigo.move_mouse(x, y, Coordinate::Abs);
                        }
                    }
            } else {
                let btn = match button_type {
                    "Right" | "右键" => Button::Right,
                    "Middle" | "中键" => Button::Middle,
                    _ => Button::Left,
                };

                match action {
                    "Press" | "按住" => {
                        let _ = enigo.button(btn, Direction::Press);
                    }
                    "Release" | "释放" => {
                        let _ = enigo.button(btn, Direction::Release);
                    }
                    "Click" | "点按" => {
                        if tap_count < 1 {
                            tap_count = 1;
                        }
                        for _ in 0..tap_count {
                            let _ = enigo.button(btn, Direction::Click);
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
                            let _ = enigo.button(btn, Direction::Click);
                            if tap_count > 1 && interval_ms > 0 {
                                tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms))
                                    .await;
                            }
                        }
                    }
                }
            }

            if i < seq_len - 1 && interval_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;
            }
        }
    }

    Ok(DataEnvelope::default())
}
