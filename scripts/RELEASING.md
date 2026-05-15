# ShortcutFlow 发布流程

## 设计原则

版本号是代码的一部分，应该在写功能代码时**提前 bump**，和功能改动落在同一个 commit。release 脚本只做纯出包，不碰版本号。

新流程: 写代码 → bump_version.py → commit(含版本号) → tag → release.py 出包

## 完整流程

### 1. 功能开发

正常写代码，改任何东西。

### 2. Bump 版本号

```bash
uv run scripts/bump_version.py
```

这会同步更新下面三个文件的版本号：
- `src-tauri/Cargo.toml` 
- `src-tauri/tauri.conf.json` 
- `package.json`

验证一致性：
```bash
uv run scripts/bump_version.py --check
```

### 3. 提交

```bash
git add -A
git commit -m "v新版本"
```

此时版本号 bump 已经在 commit 里了。`git show HEAD` 应该既能看到功能改动，也能看到版本号变化。

### 4. 打 Tag

```bash
git tag -a v新版本 -m "v新版本"
```

Tag 指向的 commit 就包含了完整的功能代码 + 版本号。

### 5. 出包

```bash
uv run scripts/release.py
```

release.py 会自动：
1. 检查工作区干净（无未提交改动）
2. 检查版本号三处一致
3. 检查新版本的 tag 指向 HEAD
4. `npm run tauri build` 编译
5. 同步 `setup/` → `target/release/`
6. 7z 极限压缩打包
7. 打开 GitHub Releases 页面并打印上传指引

### 6. 推送 + 上传

```bash
git push --follow-tags
# 然后在浏览器中上传 .7z 到 GitHub Releases
```

## 涉及文件

| 文件 | 用途 |
|------|------|
| `scripts/bump_version.py` | 更新三处版本号 |
| `scripts/release.py` | 构建 → 同步 → 打包 → 指引 |
| `RELEASING.md` | 本文档 |
