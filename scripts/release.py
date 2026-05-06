#!/usr/bin/env python3
"""
ShortcutFlow 一键发布
=====================
用法（交互式）：   uv run ./scripts/release.py
用法（指定版本）： uv run ./scripts/release.py 0.2.0

流程：
  0. 统一更新版本号（Cargo.toml / tauri.conf.json / package.json）
  1. 编译 Release（npm run tauri build）
  2. 同步 setup 到 release 目录
  3. 7z 极限压缩打包
  4. 打开 GitHub Releases → 打印发布指南

依赖：7-Zip, Node.js, Rust
"""

import os
import re
import shutil
import subprocess
import sys
import webbrowser
from pathlib import Path
from _DIRS import ROOT_DIR, SETUP_DIR, RELEASE_DIR, FINAL_RELEASE_DIR

# ── 常量 ──────────────────────────────────────────
SEVEN_ZIP = Path("C:/Program Files (x86)/7-Zip/7z.exe")
CARGO_TOML = ROOT_DIR / "src-tauri" / "Cargo.toml"
TAURI_CONF = ROOT_DIR / "src-tauri" / "tauri.conf.json"
PACKAGE_JSON = ROOT_DIR / "package.json"
RELEASE_NSIS = RELEASE_DIR / "bundle" / "nsis"
SETUP_SUBDIRS = ["BuiltinComponents", "CustomComponents", "ShortcutFlows"]
GIT_REPO = "https://github.com/2bitbit/ShortcutFlow"


# ── 工具 ──────────────────────────────────────────
def log(emoji: str, msg: str):
    print(f"  {emoji} {msg}")


def section(msg: str):
    print(f"\n{'─'*52}\n  {msg}\n{'─'*52}")


def die(msg: str):
    print(f"\n  ❌ {msg}")
    sys.exit(1)


def run(cmd: list[str], cwd=None, shell=False):
    r = subprocess.run(cmd, cwd=cwd, shell=shell)
    if r.returncode != 0:
        die(f"命令失败: {' '.join(cmd)}")


# ── 版本号读写 ─────────────────────────────────────
def read_version() -> str:
    """从 Cargo.toml 读取当前版本"""
    t = CARGO_TOML.read_text("utf-8")
    m = re.search(r'^version\s*=\s*"([^"]+)"', t, re.MULTILINE)
    if not m:
        die("未在 Cargo.toml 中找到 version 字段")
    assert m
    return m.group(1)


def bump_version(new_ver: str):
    """把三处版本号统一更新为新版本"""
    old_ver = read_version()
    if new_ver == old_ver:
        log("ℹ️", f"版本未变 ({new_ver})，跳过更新")
        return

    log("⏳", f"统一更新版本号: {old_ver} → {new_ver}")

    # Cargo.toml: version = "x.y.z"
    _replace(CARGO_TOML, rf'^(version\s*=\s*)"{old_ver}"', rf'\1"{new_ver}"')
    log("✅", "Cargo.toml")

    # tauri.conf.json: "version": "x.y.z"
    _replace(TAURI_CONF, rf'("version"\s*:\s*)"{old_ver}"', rf'\1"{new_ver}"')
    log("✅", "tauri.conf.json")

    # package.json: "version": "x.y.z"
    _replace(PACKAGE_JSON, rf'("version"\s*:\s*)"{old_ver}"', rf'\1"{new_ver}"')
    log("✅", "package.json")


def _replace(path: Path, pattern: str, repl: str):
    text = path.read_text("utf-8")
    new_text, n = re.subn(pattern, repl, text, flags=re.MULTILINE)
    if n != 1:
        die(f"{path.name}: 期望替换 1 处，实际 {n} 处（版本号可能不唯一）")
    path.write_text(new_text, "utf-8")


# ── 主流程 ─────────────────────────────────────────
def main():
    # -1. 安全检查：不允许有未提交的改动，防止版本号改了但构建失败时无法回滚
    result = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=ROOT_DIR, capture_output=True, text=True
    )
    if result.stdout.strip():
        die(
            "存在未提交的改动，请先 commit 或 stash：\n"
            + result.stdout.strip()[:500]
        )

    # 0. 版本号
    current = read_version()

    # 命令行参数 > 交互式输入
    if len(sys.argv) > 1:
        new_ver = sys.argv[1]
    else:
        print(f"\n  当前版本: {current}")
        new_ver = input("  请输入新版本号 (回车跳过): ").strip()

    if new_ver and new_ver != current:
        bump_version(new_ver)
        current = new_ver

    tag = f"v{current}"
    archive = FINAL_RELEASE_DIR / f"ShortcutFlow_{current}.7z"

    print("\n  ╔═══════════════════════════════╗")
    print(f"  ║  ShortcutFlow 发布 {tag:<12} ║")
    print("  ╚═══════════════════════════════╝")

    # 1. 编译
    section("1/4  编译 Release")
    log("⏳", "npm run tauri build")
    run(["npm", "run", "tauri", "build"], cwd=ROOT_DIR, shell=True)
    nsis = sorted(RELEASE_NSIS.glob("*.exe"))
    if not nsis:
        die(f"未找到 NSIS 安装包 ({RELEASE_NSIS})")
    log("✅", f"安装包: {nsis[0].name}")

    # 2. 同步
    section("2/4  同步 setup → release")
    for d in SETUP_SUBDIRS:
        src, dst = SETUP_DIR / d, RELEASE_DIR / d
        shutil.rmtree(dst, ignore_errors=True)
        shutil.copytree(src, dst)
        log("✅", d)

    # 3. 打包
    section("3/4  7z 高压打包")
    FINAL_RELEASE_DIR.mkdir(parents=True, exist_ok=True)
    archive.unlink(missing_ok=True)

    items = [str(RELEASE_DIR / d) for d in SETUP_SUBDIRS] + [str(nsis[0])]
    log("⏳", "7z -mx=9 -mmt=on（排除 __debug*）")
    run(
        [
            str(SEVEN_ZIP),
            "a",
            "-t7z",
            "-mx=9",
            "-mmt=on",
            "-r",
            "-x!__debug*",
            str(archive),
            *items,
        ]
    )
    mb = archive.stat().st_size / (1024 * 1024)
    log("✅", f"{archive.name} ({mb:.1f} MB)")

    # 4. 指引
    section("4/4  手动发布到 GitHub")
    release_url = f"{GIT_REPO}/releases/new"

    print(f"""
  📦 {archive}

  🔧 手动操作:
     0. 确认无误后推送到 origin 分支 (git push origin main或其他分支)

     1. 打开:  {release_url}

     2. 填写:
        • Tag:  {tag}（不存在则 Create new tag）
        • Title: ShortcutFlow {current}

     3. 上传 {archive.name}

     4. 粘贴描述 → Publish

  📋 描述模板:
  ────────────────────────────────────
  ## ShortcutFlow {current}

  **安装**
  1. 下载 {archive.name}
  2. 运行 ShortcutFlow_{current}_x64-setup.exe
  3. 审慎地：解压 BuiltinComponents / CustomComponents / ShortcutFlows 到安装根目录，选择性地覆盖

  **更新内容**
  （在此填写）
  ────────────────────────────────────
""")
    webbrowser.open(release_url)
    os.startfile(str(FINAL_RELEASE_DIR))
    print("  🎉 完成！\n")


if __name__ == "__main__":
    main()
