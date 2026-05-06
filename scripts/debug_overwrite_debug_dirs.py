"""
用于在debug时，快速把自己对setup的更新覆盖到debug的目录下。
也用于在 cargo clean 后快速恢复debug的环境。
复制完成后自动从 PRIVATE/ai-api.env 注入 LLM 组件的 API Key。
"""

from pathlib import Path
from _DIRS import DEBUG_DIR
import shutil, json

SETUP_DIR = Path(__file__).parent.parent / "setup"
OLD_DIRS = [
    DEBUG_DIR / "BuiltinComponents",
    DEBUG_DIR / "CustomComponents",
    DEBUG_DIR / "ShortcutFlows",
]

# 遍历直接子目录，直接覆盖
for old_dir in OLD_DIRS:
    shutil.rmtree(old_dir, ignore_errors=True)
    new_dir = SETUP_DIR / old_dir.name
    if new_dir.exists():
        shutil.copytree(new_dir, old_dir, dirs_exist_ok=True)
        print(f"✅ 已覆盖 {old_dir}")
    else:
        print(f"❌ {new_dir} 不存在")

# ── 自动注入 LLM API Key ──
env_path = Path(__file__).parent.parent / "PRIVATE" / "ai-api.env"
llm_json = DEBUG_DIR / "BuiltinComponents" / "LLM" / "LLM.json"

if env_path.exists() and llm_json.exists():
    lines = env_path.read_text(encoding="utf-8").strip().splitlines()
    # 取非空行：base_url, api_key, model
    values = [l.strip() for l in lines if l.strip()]
    if len(values) >= 2:
        api_key = values[1]  # second non-empty line = API key
        with open(llm_json, "r", encoding="utf-8") as f:
            llm = json.load(f)
        llm["default_config"]["api_key"] = api_key
        with open(llm_json, "w", encoding="utf-8") as f:
            json.dump(llm, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print(f"🔑 已向 LLM 组件的默认配置内注入 LLM API Key（来自 PRIVATE/ai-api.env）")
    else:
        print(f"⚠️  PRIVATE/ai-api.env 格式不符，跳过 API Key 注入")
else:
    if not env_path.exists():
        print(f"ℹ️  PRIVATE/ai-api.env 不存在，跳过 API Key 注入")
