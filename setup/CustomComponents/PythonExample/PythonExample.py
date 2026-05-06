import sys
import json
import os
import base64
from pathlib import Path

input_raw = sys.stdin.read().strip()

data_envolope = json.loads(input_raw)
metadata, payload = data_envolope["metadata"], data_envolope["payload"]
save_path, img_base64 = Path(metadata["save_path"]).expanduser(), payload

os.makedirs(os.path.dirname(save_path), exist_ok=True)

with open(save_path, "wb") as f:
    f.write(base64.b64decode(img_base64))

print(f"Python接管成功：提取屏幕图像并静默存档到了 {save_path}！")
