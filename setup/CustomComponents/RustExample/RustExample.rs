#!/usr/bin/env cargo

---
[package]
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.22"
---

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Read};
use std::fs;
use std::path::PathBuf;
use base64::Engine;

/// ShortcutFlow 的标准数据封套，与引擎保持一致
#[derive(Debug, Serialize, Deserialize)]
struct DataEnvelope {
    metadata: Option<Value>,
    payload: Option<Value>,
}

fn expanduser(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_default();
        if path == "~" {
            home
        } else {
            format!("{}/{}", home, &path[2..])
        }
    } else {
        path.to_string()
    }
}

fn main() {
    let mut input_raw = String::new();
    io::stdin().read_to_string(&mut input_raw).expect("无法读取 stdin");

    let envelope: DataEnvelope =
        serde_json::from_str(&input_raw).expect("无法解析输入的 DataEnvelope JSON");

    // 从 metadata 中提取 save_path
    let save_path = envelope
        .metadata
        .as_ref()
        .and_then(|m| m.get("save_path"))
        .and_then(|v| v.as_str())
        .unwrap_or("output.png");

    // 从 payload 中提取 base64 图片数据
    let img_b64 = envelope
        .payload
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let resolved = expanduser(save_path);
    let path = PathBuf::from(&resolved);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("无法创建目标目录");
    }

    let img_bytes = base64::engine::general_purpose::STANDARD
        .decode(img_b64)
        .expect("base64 解码失败");

    fs::write(&path, img_bytes).expect("无法写入图片文件");

    println!("Rust接管成功：提取屏幕图像并静默存档到了 {}！", resolved);
}
