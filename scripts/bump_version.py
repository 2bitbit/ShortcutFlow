#!/usr/bin/env python3
"""
版本号统一更新
===============
同步更新三处版本号: Cargo.toml / tauri.conf.json / package.json

用法:
    uv run scripts/bump_version.py 1.0.5    # 指定版本
    uv run scripts/bump_version.py           # 交互式输入
    uv run scripts/bump_version.py --check   # 仅检查三处版本是否一致

应该在写功能代码时先 bump 版本号，再 commit。
release.py 不再负责版本号——它只做构建打包。
"""

import re
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).parent.parent
CARGO_TOML = ROOT_DIR / "src-tauri" / "Cargo.toml"
TAURI_CONF = ROOT_DIR / "src-tauri" / "tauri.conf.json"
PACKAGE_JSON = ROOT_DIR / "package.json"

FILES = {
    "Cargo.toml": (CARGO_TOML, r'^version\s*=\s*"([^"]+)"',
                   r'^(version\s*=\s*)"[^"]+"'),
    "tauri.conf.json": (TAURI_CONF, r'"version"\s*:\s*"([^"]+)"',
                        r'("version"\s*:\s*)"[^"]+"'),
    "package.json": (PACKAGE_JSON, r'"version"\s*:\s*"([^"]+)"',
                     r'("version"\s*:\s*)"[^"]+"'),
}


def read_version(path: Path, pattern: str) -> str:
    text = path.read_text("utf-8")
    m = re.search(pattern, text, re.MULTILINE)
    if not m:
        raise SystemExit(f"❌ 未在 {path.name} 中找到 version 字段")
    return m.group(1)


def check_consistency() -> dict[str, str]:
    """检查三处版本号是否一致，返回 {文件名: 版本号}"""
    versions = {}
    for name, (path, pattern, _) in FILES.items():
        versions[name] = read_version(path, pattern)
    if len(set(versions.values())) != 1:
        print("❌ 三处版本号不一致：")
        for name, ver in versions.items():
            print(f"   {name}: {ver}")
        raise SystemExit(1)
    return versions


def bump(new_ver: str):
    current = check_consistency()
    old_ver = list(current.values())[0]

    if new_ver == old_ver:
        print(f"ℹ️  版本未变 ({new_ver})，无需更新")
        return

    print(f"  版本更新: {old_ver} → {new_ver}")
    for name, (path, _, replace_pattern) in FILES.items():
        text = path.read_text("utf-8")
        new_text, n = re.subn(replace_pattern, rf'\1"{new_ver}"', text,
                              flags=re.MULTILINE)
        if n != 1:
            raise SystemExit(f"❌ {name}: 期望替换 1 处，实际 {n} 处")
        path.write_text(new_text, "utf-8")
        print(f"  ✅ {name}")


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "--check":
        check_consistency()
        print(f"✅ 三处版本号一致: {list(check_consistency().values())[0]}")
        return

    if len(sys.argv) > 1 and not sys.argv[1].startswith("-"):
        new_ver = sys.argv[1]
    else:
        current = check_consistency()
        cur = list(current.values())[0]
        new_ver = input(f"  当前版本: {cur}\n"
                        f"  输入新版本号 (回车取消): ").strip()
        if not new_ver:
            print("  已取消")
            return

    bump(new_ver)
    print(f"\n  下一步: git add -A && git commit -m \"chore: bump to {new_ver}\"")


if __name__ == "__main__":
    main()
