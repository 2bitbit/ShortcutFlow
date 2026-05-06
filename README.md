<div align="center">
  <img src="logo.ico" width="128" height="128"> <h1>ShortcutFlow</h1>
  <div align="center">
    <img src="https://img.shields.io/badge/Tauri-2.0-blue?style=flat-square&logo=tauri" alt="Tauri" />
    <img src="https://img.shields.io/badge/Rust-1.90+-orange?style=flat-square&logo=rust" alt="Rust" />
    <img src="https://img.shields.io/badge/Vue.js-3.0-green?style=flat-square&logo=vuedotjs" alt="Vue" />
    <img src="https://img.shields.io/badge/Platform-Windows-blue?style=flat-square"alt="Windows" />
    <img src="https://img.shields.io/badge/Platform-Windows-blue?style=flat-square"alt="Windows" />
  </div>

  <div>
      <span style="display: inline-block; padding: 4px 8px; background-color: #35b439ff; color: #ffffff; font-family: monospace; font-size: 12px; font-weight: bold; border-radius: 4px; border: 1px solid #333; user-select: none;">&lt;/&gt; HUMAN REVIEWED</span>
      <span style=" padding: 6px 8px; background-color: rgba(233, 233, 233, 0.81); color: #0415fd; font-family: monospace; font-size: 12px; font-weight: bold; border-radius: 4px; border: 1px solid #333; user-select: none;">&lt;/&gt; <img src="./herseek_small.png"> HERSEEK COBUILD</span>
  </div>
  
</div>


>[!Warning]
>项目处于原型开发中，敬请期待。
>欠个demo.gif


超级大鲁棒！！！

# 安装指导
前往_____________________下载最近压缩包（含3个目录和一个安装程序），安装到特定目录后，把压缩包内的目录放到根目录下（这会覆盖原有的文件），安装即完成。


---
**ShortcutFlow** 是一个极其轻量、高性能、高拓展性的工作流编排框架，自动化你的日常所需，显著提升效率。它可以让你将繁琐的日常键鼠操作、剪贴板处理、甚至是与大语言模型 (LLM) 的多步交互，统统编排为可视化节点，并允许**绑定到一个全局快捷键上瞬间触发**。

无论是工作中的“一键智能截图转 LaTeX”，还是“绕过网页禁止粘贴的硬件级打字模拟”，全都在一念之间。

<div align=center>
<img src="./public/demo.gif" width=60%>
</div>

> [!NOTE]
> 当前只支持 windows 系统
> 欢迎积极提交 PR

## 🌟 核心特性

- 🎯 **全局劫持，一键唤起**：将编排好的复杂工作流绑定到系统级快捷键，即使在后台静默运行也能随时响应。
- 🧱 **自由的可视化编排**：内置优雅的深色模式流编辑器。如同拼接乐高一般，自由设定数据的“流入”与“流出”，随时调整每个节点的启动延时。
- 🧠 **原生的 AI 互联**：自带极简的 OpenAI 格式节点，可将其作为管线枢纽，无缝承接来自上文的剪贴板或截图数据，输出结果。
- 🔌 **语言无关的无边界拓展**：嫌默认组件不够？通过开放式的 Shell 节点组件，你可以用 Python、Rust、Node 等任何你喜欢的语言编写本地脚本，将其直接桥接入自动化管线！
- 🗄️ **极客风日志诊断**：内置沉浸式日志查看面板，完美捕获底层 Rust 引擎的每一步节点执行状态与错漏追踪。

> [!WARNING] 初次启动会有卡顿，那是正在初始化。

## 🧩 开箱即用的内置节点

ShortcutFlow 预装了多种针对桌面环境深度优化的原子节点：

| 节点组件                 | 连通性约定  | 描述说明                                                                               |
| ------------------------ | ----------- | -------------------------------------------------------------------------------------- |
| 📸 **ScreenCapture**     | *产生数据*  | 唤起系统级框选截图，并将其转化为 Base64 长文本推入管线。                               |
| 📋 **ReadClipboard**     | *产生数据*  | 静默读取系统剪贴板当前的纯文本内容作为初始数据源。                                     |
| ⌨️ **Typing**             | *依赖输入*  | 真实的键盘模拟器。接收流数据，以硬件驱动级别逐个字符敲击输出（专治禁用粘贴的网页）。   |
| 🤖 **LLM**               | *输入/产出* | 系统大脑枢纽。接收任意文本或图像流，发送给大模型进行结构化，并返回纯净推理结果。       |
| 📥 **Paste**             | *依赖输入*  | 流程终结者。拦截上游传导的数据并覆盖到剪贴板，向失焦处触发完美的原生 `Ctrl+V`。        |
| 🖥️ **Shell**              | *自由支配*  | 极其灵活的执行器。将上游数据压入 `stdin` 并执行任意工作区的本地可执行命令（60s 硬超时）。 |
| 🔔 **Popup**             | *依赖输入*  | 消息广播者。利用操作系统的原生系统级弹窗，醒目地通知上游数据内容。                     |
| 🌐 **HtmlWindow**        | *依赖输入*  | 弹出自定义 HTML 窗口。支持阻塞/非阻塞、置顶、自定义尺寸位置。                          |
| 🔀 **Router**            | *输入/产出* | 条件路由器。按正则/字符数等条件匹配 payload，命中后执行对应子流。                       |
| ⌨️ **KeyListener**        | *产生数据*  | 阻塞等待用户按下指定快捷键，返回对应 metadata/payload 给下游。                         |
| 🌍 **HttpRequest**       | *产生数据*  | 发射 HTTP 请求（GET/POST/PUT/DELETE），将响应体传递给下游节点。                        |
| 🔄 **CallFlow**          | *输入/产出* | 执行子流程：通过 `flow_id` 触发并运行另一个已配置的工作流。                             |
| 🔍 **Regex**             | *输入/产出* | 对 payload 按正则表达式替换。支持捕获组、反向引用。                                     |
| 🖱️ **SimulateKey**        | *产生数据*  | 按键序列模拟器。可编排复杂按键组合（如 `Ctrl+C`）的序列执行。                           |
| 📋 **Copy**              | *产生数据*  | 向焦点区触发模拟 `Ctrl+C` 复制文本或图片（Base64）到剪贴板。                            |
| 📝 **WriteClipboard**    | *依赖输入*  | 将上游 payload 写入系统剪贴板。                                                        |
| 🧹 **ClearClipboard**    | *产生数据*  | 清空系统剪贴板内容。                                                                   |

## 🚀 热门玩法示例

- **「公式识别与自动输入」**： `ScreenCapture` 截图 ➔ `LLM` 提示词剥离为 LaTeX ➔ `Paste` 将公式源码一键粘贴进你的论文或笔记。
- **「绕过限制死区打字」**： `ReadClipboard` 获取剪切板资料 ➔ `Typing` 模拟人手按键敲击，突破任何前端防粘贴系统。
- **「极速外部归档」**： `Copy` 模拟系统复制 ➔ `Shell` 呼叫你的私有 Python 脚本自动整理到本地知识库 ➔ `Popup` 右下角弹窗提示成功。


<div align="center">
  <sub>Built with ❤️ by Hackers, for Hackers.</sub>
</div>

