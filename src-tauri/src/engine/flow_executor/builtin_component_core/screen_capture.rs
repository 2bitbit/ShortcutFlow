use crate::models::DataEnvelope;
use anyhow::Result;
use serde_json::Value;

pub async fn execute() -> Result<DataEnvelope> {
    let result = tokio::task::spawn_blocking(capture_with_minifb)
        .await
        .map_err(|e| anyhow::anyhow!(e))?
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(png_bytes) = result {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        Ok(DataEnvelope {
            metadata: None,
            payload: Some(Value::String(STANDARD.encode(&png_bytes))),
        })
    } else {
        anyhow::bail!("用户取消了截图或超时未操作")
    }
}

fn capture_with_minifb() -> Result<Option<Vec<u8>>, String> {
    use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
    use xcap::Monitor;

    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;
        SetProcessDPIAware();
    }

    let monitor = Monitor::all()
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .ok_or("No monitor")?;
    let image = monitor.capture_image().map_err(|e| e.to_string())?;

    let width = image.width() as usize;
    let height = image.height() as usize;
    let raw_pixels = image.into_raw();

    let mut original_bg = vec![0u32; width * height];
    let mut dark_bg = vec![0u32; width * height];

    for (i, chunk) in raw_pixels.chunks_exact(4).enumerate() {
        let (r, g, b) = (chunk[0] as u32, chunk[1] as u32, chunk[2] as u32);
        let color = (r << 16) | (g << 8) | b;
        original_bg[i] = color;
        let dark_color = ((r / 2) << 16) | ((g / 2) << 8) | (b / 2);
        dark_bg[i] = dark_color;
    }

    let mut window = Window::new(
        "Screenshot Region Selector",
        width,
        height,
        WindowOptions {
            borderless: true,
            title: false,
            topmost: true,
            resize: false,
            transparency: true,
            none: true,
            ..WindowOptions::default()
        },
    )
    .map_err(|e| e.to_string())?;

    window.update_with_buffer(&dark_bg, width, height).unwrap();

    #[cfg(target_os = "windows")]
    {
        let hwnd = window.get_window_handle();
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GWL_STYLE, GetWindowLongPtrW, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE,
                SetForegroundWindow, SetWindowLongW, SetWindowPos, WS_CAPTION, WS_THICKFRAME,
            };
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            SetWindowLongW(
                hwnd,
                GWL_STYLE,
                ((style as u32) & !WS_CAPTION & !WS_THICKFRAME) as i32,
            );
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED,
            );
            SetForegroundWindow(hwnd);
        }
    }

    window.set_position(0, 0);
    window.set_target_fps(120);

    let mut start_pos: Option<(f32, f32)> = None;
    let mut end_pos: Option<(f32, f32)> = None;
    let mut is_drawing_rectangle = false;
    let mut final_rect = None;

    let mut current_buffer = dark_bg.clone();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mouse_pos = window.get_mouse_pos(MouseMode::Clamp);
        let left_down = window.get_mouse_down(MouseButton::Left);

        if !left_down && is_drawing_rectangle {
            if let (Some(s), Some(e)) = (start_pos, end_pos) {
                let rx = s.0.min(e.0) as u32;
                let ry = s.1.min(e.1) as u32;
                let rw = (s.0 - e.0).abs() as u32;
                let rh = (s.1 - e.1).abs() as u32;
                if rw > 5 && rh > 5 {
                    final_rect = Some((rx, ry, rw, rh));
                }
            }
            break;
        }
        if left_down {
            if !is_drawing_rectangle {
                start_pos = mouse_pos;
                is_drawing_rectangle = true;
            }
            end_pos = mouse_pos;
        }

        current_buffer.copy_from_slice(&dark_bg);
        if is_drawing_rectangle
            && let (Some(s), Some(e)) = (start_pos, end_pos) {
                draw_rectangle(s, e, &mut current_buffer, width, height, &original_bg);
            }
        window
            .update_with_buffer(&current_buffer, width, height)
            .unwrap();
    }

    drop(window);

    if let Some((x, y, w, h)) = final_rect {
        let mut rgba_image = image::RgbaImage::from_raw(width as u32, height as u32, raw_pixels)
            .ok_or("Image err")?;
        let cropped = image::imageops::crop(&mut rgba_image, x, y, w, h).to_image();

        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        cropped
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;

        Ok(Some(png_bytes))
    } else {
        Ok(None)
    }
}

fn draw_rectangle(
    start_pos: (f32, f32),
    end_pos: (f32, f32),
    current_buffer: &mut [u32],
    width: usize,
    height: usize,
    original_bg: &[u32],
) {
    let rx = start_pos.0.min(end_pos.0) as usize;
    let ry = start_pos.1.min(end_pos.1) as usize;
    let rw = (start_pos.0 - end_pos.0).abs() as usize;
    let rh = (start_pos.1 - end_pos.1).abs() as usize;

    for y in ry..=(ry + rh).min(height - 1) {
        let row_idx = y * width;
        let start_idx = row_idx + rx;
        let end_idx = row_idx + (rx + rw).min(width - 1);
        current_buffer[start_idx..=end_idx].copy_from_slice(&original_bg[start_idx..=end_idx]);
    }

    let border_color = 0x00_00_FF_00;
    let border_thickness = 2;

    for y in 0..border_thickness {
        if ry + y < height {
            let row_top = (ry + y) * width;
            for x in rx..=(rx + rw).min(width - 1) {
                current_buffer[row_top + x] = border_color;
            }
        }
        if ry + rh >= y && ry + rh - y < height {
            let row_bottom = (ry + rh - y) * width;
            for x in rx..=(rx + rw).min(width - 1) {
                current_buffer[row_bottom + x] = border_color;
            }
        }
    }

    for x in 0..border_thickness {
        if rx + x < width {
            for y in ry..=(ry + rh).min(height - 1) {
                current_buffer[y * width + rx + x] = border_color;
            }
        }
        if rx + rw >= x && rx + rw - x < width {
            for y in ry..=(ry + rh).min(height - 1) {
                current_buffer[y * width + rx + rw - x] = border_color;
            }
        }
    }
}
