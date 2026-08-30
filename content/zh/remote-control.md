---
title: 远程控制
description: 通过本地网络用 Bitfocus Companion 触发 SoundNinja。
order: 7
---

# 远程控制

SoundNinja 可以在本地网络上监听，让 Bitfocus Companion（或任何 HTTP/WebSocket 客户端）播放和停止音效。

## 启用远程服务器

1. 打开 **设置 → 远程**。
2. 启用远程服务器。默认端口为 **7331**。
3. 可选：设置令牌。客户端必须以 Bearer 令牌或 `?token=` 发送。
4. 复制 `http://IP:PORT` URL，或从 **设置 → 关于** 复制系统 IP。

Windows 可能在服务器首次启动时显示防火墙提示。请允许专用网络访问。

## Bitfocus Companion

1. 安装 [Bitfocus Companion](https://bitfocus.io/companion)。
2. 从官方文件夹下载 SoundNinja 模块：[companion-module-soundninja](https://github.com/marcus-universe/SoundNinja/tree/main/companion-module-soundninja)。尚无 Companion Store 软件包 — 这个文件夹就是下载内容。
3. 在 Companion 中打开 **Developer → Modules**，或将文件夹放到 Companion 的自定义模块路径。
4. 在模块文件夹中运行 `npm install`。
5. 添加一个 **Sound Ninja** 连接。粘贴电脑 IP、端口，以及你设置的令牌。

## 动作

- **Trigger Sound** — 从实时下拉列表选择，或输入 8 位音效 ID（可使用 Companion 变量）
- **Stop Sound** — 按 ID 停止一个正在播放的音效
- **Stop All** — 停止所有正在播放的音效

在 SoundNinja 中，从按钮上下文菜单或多选 ID 芯片复制音效 ID。

## 反馈和变量

- **Sound Playing** — 该音效 ID 处于活动状态时为 true（用于按钮颜色）
- `$(soundninja:connected)` — `true` / `false`
- `$(soundninja:playing_count)` — 正在播放的音效数量
- `$(soundninja:last_triggered)` — 上次触发的音效 ID

## HTTP API（高级）

基础路径：`/api/v1`

- `GET /info` — 应用名称、版本、协议、是否需要令牌
- `GET /sounds` — 音效列表（id、name、tabs、active）
- `GET /state` — 音效以及当前正在播放的 ID
- `POST /trigger` 带 `{ "id" }` — 播放音效
- `GET /trigger/:id` — 播放（便于在浏览器中测试）
- `POST /stop` 带可选 `{ "id" }` — 停止一个或全部
- `GET /ws` — 实时状态；接收 trigger/stop 命令
