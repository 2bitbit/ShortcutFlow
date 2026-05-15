#!/usr/bin/env python3
"""
ShortcutFlow 发布打包
=====================
自 v1.0.4 起，版本号 bump 独立为 bump_version.py。
本脚本只负责构建 → 同步 → 打包 → 输出发布指引。

前置条件（由开发者自行完成）：
  1. bump_version.py 0.x.x   （更新版本号）
  2. git commit               （提交，版本号随功能一起入 commit）
  3. git tag v0.x.x           （打 tag）

用法:
    uv run scripts/release.py          # 从 Cargo.toml 读取版本号
    uv run scripts/release.py 0.2.0    # 指定版本号（用于回退等场景）

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


# ── 版本号读取 ─────────────────────────────────────
def read_version() -> str:
    """从 Cargo.toml 读取当前版本"""
    t = CARGO_TOML.read_text("utf-8")
    m = re.search(r'^version\s*=\s*"([^"]+)"', t, re.MULTILINE)
    if not m:
        die("未在 Cargo.toml 中找到 version 字段")
    return m.group(1)


def check_versions_consistent():
    """验证三处版本号一致"""
    versions = {}
    for path, pattern in [
        (CARGO_TOML, r'^version\s*=\s*"([^"]+)"'),
        (TAURI_CONF, r'"version"\s*:\s*"([^"]+)"'),
        (PACKAGE_JSON, r'"version"\s*:\s*"([^"]+)"'),
    ]:
        t = path.read_text("utf-8")
        m = re.search(pattern, t, re.MULTILINE)
        v = m.group(1) if m else "??? 未找到"
        versions[path.name] = v

    unique = set(versions.values())
    if len(unique) != 1:
        print("❌ 三处版本号不一致：")
        for name, ver in versions.items():
            print(f"   {name}: {ver}")
        die("请先运行 uv run scripts/bump_version.py --check 检查")


# ── 主流程 ─────────────────────────────────────────
def main():
    # 0. 版本号
    file_version = read_version()

    if len(sys.argv) > 1:
        cli_version = sys.argv[1]
        if cli_version != file_version:
            die(
                f"命令行版本 ({cli_version}) 与 Cargo.toml ({file_version}) 不一致。\n"
                f"如需更换版本号，请先运行 uv run scripts/bump_version.py {cli_version}"
            )
    current = file_version

    check_versions_consistent()
    tag = f"v{current}"
    archive = FINAL_RELEASE_DIR / f"ShortcutFlow_{current}.7z"

    # -1. 安全检查
    result = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=ROOT_DIR, capture_output=True, text=True
    )
    if result.stdout.strip():
        die(
            "存在未提交的改动，请先 commit 或 stash：\n"
            + result.stdout.strip()[:500]
        )

    # 检查 tag 是否存在且指向 HEAD
    tag_result = subprocess.run(
        ["git", "tag", "--points-at", "HEAD"],
        cwd=ROOT_DIR, capture_output=True, text=True
    )
    if tag not in tag_result.stdout.strip().split("\n"):
        print(f"  ⚠️  当前 HEAD 上没有 tag {tag}")
        print(f"  HEAD 上的 tag: {tag_result.stdout.strip() or '(无)'}")
        answer = input(f"  是否继续？[y/N] ").strip().lower()
        if answer != "y":
            die("已取消。请先: git tag -a {tag} -m '{tag}'")

    print("\n  ╔═══════════════════════════════╗")
    print(f"  ║  ShortcutFlow 发布 {tag:<12} ║")
    print("  ╚═══════════════════════════════╝")

    # 1. 编译
    section("1/4  编译 Release")
    log("⏳", "npm run tauri build")
    run(["npm", "run", "tauri", "build"], cwd=ROOT_DIR, shell=True)
    nsis_file = RELEASE_NSIS / f"ShortcutFlow_{current}_x64-setup.exe"
    if not nsis_file.exists():
        die(f"未找到 NSIS 安装包 ({nsis_file})")
    log("✅", f"安装包: {nsis_file.name}")

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

    items = [str(RELEASE_DIR / d) for d in SETUP_SUBDIRS] + [str(nsis_file)]
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
     0. 确认无误后推送到 origin (git push --follow-tags)

     1. 打开:  {release_url}

     2. 填写:
        • Tag:  {tag}（已存在，从下拉框选择）
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
