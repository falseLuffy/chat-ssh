<div align="center">

# AI-SSH
**现代化、智能化的跨平台 SSH 终端与服务器运维管理工具**

![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?style=flat-square&logo=tauri)
![Vue.js](https://img.shields.io/badge/Vue.js-3.x-4FC08D?style=flat-square&logo=vuedotjs)
![Rust](https://img.shields.io/badge/Rust-1.70+-000000?style=flat-square&logo=rust)
![Tailwind CSS](https://img.shields.io/badge/Tailwind-CSS-38B2AC?style=flat-square&logo=tailwind-css)
![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)

[特性](#-核心特性) • [快速开始](#-快速开始) • [使用指南](#-使用指南)

</div>

## 项目简介

AI-SSH 是一个基于 **Tauri + Vue 3 + Rust** 的跨平台 SSH 客户端与服务器运维管理工具。它集成了交互式终端、SFTP 文件管理、脚本分发执行、服务器监控（CPU/内存/磁盘/网络）、Docker 管理、系统服务管理、进程管理、防火墙管理等运维功能，并深度集成 **DeepSeek AI** 提供智能辅助。

## 核心特性

### SSH 终端与连接管理
- 支持密码认证的 SSH 连接管理，多服务器会话并存
- 基于 `xterm.js` 的交互式 Web 终端，支持全键盘事件、粘贴与清屏
- **Xshell 会话导入**：支持 `.xsh` 和 `.xts` 文件格式，自动解码中文编码并解密 DPAPI 加密密码

### SFTP 文件管理
- 可视化远程目录浏览，目录/文件优先排序展示
- 支持文件上传、下载、删除操作
- 操作日志记录与冲突处理

### 自动化脚本引擎
- **脚本编辑器**：在线编写、保存 Shell 脚本，支持交互式脚本模式
- **单机/多机执行**：支持右键菜单快速切换单机或多机执行模式
- **自动唤醒**：多机执行时对未连接的服务器自动连接并发起执行
- **执行日志**：结构化的执行结果展示，区分成功/失败

### 服务器监控面板
- **实时概览**：CPU 负载、内存占用、磁盘空间、运行时间、操作系统、主机名
- **可视化图表**：基于 ECharts 的 CPU 仪表盘、内存/磁盘饼图、CPU 负载趋势图、实时网络流量图
- **自动轮询**：每 5 秒刷新一次系统状态

### Docker 管理
- 查看所有容器列表（运行中/已停止）
- 容器操作：启动、停止、重启、暂停、取消暂停、删除
- 搜索过滤

### 系统服务管理
- 查看所有 systemd 服务及运行状态
- 服务操作：启动、停止、重启、启用、禁用
- 搜索过滤

### 进程管理
- 查看当前 CPU/内存占用最高的进程
- 一键终止进程（带确认）

### 防火墙管理
- 自动检测 UFW（Ubuntu/Debian）或 firewalld（CentOS/RHEL）
- 查看所有防火墙规则
- 添加/删除端口规则

### 服务器智能巡检
- **规则引擎**：支持阈值检查（CPU/内存/磁盘）、服务状态检查、Docker 状态检查、自定义命令输出检查
- **巡检报告**：记录每次巡检结果，包含各检查项状态（pass/warning/critical/error）
- **定时调度**：支持 cron 表达式配置定期巡检
- **AI 总结**：自动调用 DeepSeek AI 生成巡检分析报告与修复建议

### AI 智能辅助（DeepSeek）
- **AI 命令生成**：通过自然语言描述生成对应的 Shell 指令
- **AI 风险审查**：执行前分析指令风险等级、潜在后果与改进建议
- **AI 脚本分析**：自动分析脚本用途、描述执行后果、检测明文密码/Token 泄露
- **AI 服务器诊断**：结合实时系统状态进行智能故障诊断
- **AI 对话助手**：内建 ChatBox，支持本地知识库（RAG），提供个性化运维咨询
- **本地知识库**：记录常用命令与知识，增强 AI 回答的准确性

### 安全特性
- **主密码保护**：脚本编辑、删除、执行等敏感操作需验证主密码
- **高危操作阻断**：AI 识别到密码泄露或高风险操作时给出明确警告
- **执行确认机制**：执行脚本前展示风险描述并二次确认

---

## 快速开始

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

4. **构建生产版本**
   ```bash
   npm run tauri build
   ```

---

## 使用指南

### 1. 服务器管理
在左侧边栏点击 **+** 添加服务器信息（主机、端口、用户名、密码），连接成功后即可在终端中操作。

### 2. 监控与运维
连接服务器后切换至"管理"标签页，可查看系统概览、Docker 容器、系统服务、进程与防火墙规则。

### 3. 脚本分发
在"运维脚本"页面编写或选择脚本，支持右键菜单快速切换单机/多机执行模式。

### 4. 智能巡检
在"管理 > 巡检"页面创建巡检规则，支持手动执行或设置定时任务。

### 5. AI 辅助
在设置中配置 DeepSeek API Key 后，可在终端中使用 AI 命令生成、AI 对话等功能。

---

## 项目架构

- **Frontend (Vue 3 + Vite + Tailwind CSS + Pinia)**：视图层、状态管理、UI 交互
- **Backend (Rust + Tauri)**：SSH/SFTP 连接（ssh2 库）、SQLite 持久化、PTY 进程管理、AI API 调用

---

## 许可证

本项目基于 [MIT License](LICENSE) 协议开源。

<div align="center">
  <i>如果此项目帮助到了您，请给一个 ⭐️ Star！</i>
</div>
