from pathlib import Path

ROOT_DIR = Path(__file__).parent.parent

DEBUG_DIR = ROOT_DIR / "src-tauri" / "target" / "debug"
RELEASE_DIR = ROOT_DIR / "src-tauri" / "target" / "release"

SETUP_DIR = ROOT_DIR / "setup"

FINAL_RELEASE_DIR = ROOT_DIR / "final_release"
