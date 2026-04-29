<div align="center">
  
# 🚀 AI-SSH
**现代化、智能化的跨平台 SSH 终端与多机运维工具**

![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?style=flat-square&logo=tauri)
![Vue.js](https://img.shields.io/badge/Vue.js-3.x-4FC08D?style=flat-square&logo=vuedotjs)
![Rust](https://img.shields.io/badge/Rust-1.70+-000000?style=flat-square&logo=rust)
![Tailwind CSS](https://img.shields.io/badge/Tailwind-CSS-38B2AC?style=flat-square&logo=tailwind-css)
![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)

[特性](#-核心特性) •
[快速开始](#-快速开始) •
[项目架构](#-项目架构) •
[使用指南](#-使用指南) •
[贡献](#-贡献指南)

</div>

## 📖 项目简介

AI-SSH 是一个基于 **Tauri + Vue 3 + Rust** 构建的下一代智能化服务器管理工具。它不仅拥有极具现代感和毛玻璃质感（Glassmorphism）的惊艳 UI 界面，还深度集成了 **DeepSeek AI** 技术。无论是常规的 SSH 终端交互、SFTP 文件管理，还是复杂的单机/多机自动化运维脚本分发，AI-SSH 都能通过 AI 的辅助让您的工作效率倍增。

## ✨ 核心特性

- 🎨 **极致的现代 UI 体验**：全方位深度定制的毛玻璃暗色主题，告别传统的生硬弹窗，拥有流畅的动画和交互体验。
- ⚡ **原生级性能**：底层采用 Rust 驱动，占用极低的内存与 CPU，极速启动。
- 🤖 **AI 深度赋能 (DeepSeek & RAG)**：
  - **本地私有知识库 (RAG)**：支持检索增强生成（RAG），结合本地知识文档，提供更精准、符合您业务上下文的 AI 回答，保护数据隐私。
  - **脚本智能分析**：编写或阅读运维脚本时，一键使用 AI 分析脚本功能、预估执行后果。
  - **高危操作阻断**：AI 自动识别脚本中的明文密码或 Token，并在执行前给出高危警告。
  - **智能对话助手**：内建 AI ChatBox，为您提供实时的 Linux 命令咨询与排障支持。
- 🖥️ **跨平台终端集成**：基于 `xterm.js` 的丝滑 Web 终端，支持全键盘事件拦截与多标签页切换。
- 📂 **SFTP 可视化文件管理**：直观的服务器目录树，支持上传、下载及权限修改等常规文件操作。
- 🚀 **强大的多机自动化执行**：
  - 支持 **单机模式** / **多机模式** 快速切换（支持脚本卡片右键悬浮菜单操作）。
  - **自动唤醒机制**：多机并发执行时，针对未连接的服务器可一键自动连接并无缝执行。
  - 结构化的执行日志展示。 

---

## 🚀 快速开始

### 环境依赖

在开始之前，请确保您的开发环境中已经安装了以下依赖：
- [Node.js](https://nodejs.org/en/) (v16+)
- [Rust](https://www.rust-lang.org/tools/install) (稳定版)
- C++ 构建工具 (Windows 需安装 Visual Studio C++ build tools)

### 安装与运行

1. **克隆项目**
   ```bash
   git clone https://github.com/your-username/ai-ssh.git
   cd ai-ssh
   ```

2. **安装前端依赖**
   ```bash
   npm install
   ```

3. **运行开发环境**
   ```bash
   npm run tauri dev
   ```

4. **构建生产版本 (安装包)**
   ```bash
   npm run tauri build
   ```
   > 构建完成后，安装包将生成在 `src-tauri/target/release/bundle/` 目录下（支持 MSI 和 NSIS 安装包）。

---

## 🛠️ 项目架构

本项目采用经典的 Tauri 混合架构设计：

- **Frontend (Vue 3 + Vite + Tailwind CSS)**:
  - 负责所有视图层、状态管理 (Pinia) 与现代化的 UI 交互。
  - 核心组件：`TerminalView` (终端)、`OpsView` (运维脚本中心)、`FileBrowser` (文件管理)、`AiChatBox` (AI 对话)。
- **Backend (Rust + Tauri API)**:
  - 处理底层 SSH/SFTP 长连接 (使用 `ssh2` 库)。
  - SQLite 本地数据库交互，负责服务器信息和脚本的安全持久化。
  - PTY 进程调用与原生操作系统接口交互。

---

## 💡 使用指南

### 1. 添加与连接服务器
在左侧边栏点击 **+** 按钮添加服务器信息。填写 Host、Port 和凭证信息后即可建立 SSH 连接。
*支持对已保存的服务器右键点击进行“一键复制”或“删除”操作。*

### 2. AI 智能运维
- 进入系统设置，配置您的 **DeepSeek API Key**。
- 在“我的脚本”中编写 Shell 脚本时，点击 **AI 智能生成描述** 按钮即可体验自动补全与高危检测。

### 3. 多机并发脚本分发
- 在“我的脚本”页面，使用**右键菜单**点击任意脚本，选择 **多机执行**。
- 在右侧展开的目标选择器中勾选任意数量的服务器（无需预先连接）。
- 点击“一键运行”，AI-SSH 会自动处理连接握手并下发脚本。

---

## 🤝 贡献指南

我们非常欢迎来自社区的贡献！如果您有好的想法或发现了 Bug，请随意提交 Issue 或 Pull Request：

1. Fork 本仓库
2. 创建您的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交您的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 将您的更改推送到分支 (`git push origin feature/AmazingFeature`)
5. 发起一个 Pull Request

---

## 📄 开源协议

本项目基于 [MIT License](LICENSE) 协议开源。欢迎自由使用与二次开发。

<div align="center">
  <i>如果 AI-SSH 帮助到了您，请给本项目点个 ⭐️ Star！</i>
</div>
