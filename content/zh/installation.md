---
title: 安装
description: 在 Windows、macOS 或 Linux 上安装 SoundNinja。
order: 2
---

# 安装

从[下载页](/download)或 [GitHub Releases](https://github.com/marcus-universe/SoundNinja/releases/latest) 获取构建。SoundNinja 处于早期开发，请将构建视为测试版。

## Windows

- 64 位 Windows 的 **安装程序 (.exe)** 或 **MSI (.msi)**
- 落地页目标：Windows 10 及更新
- 应用 README 也列出 Windows 8 及以上

运行安装程序，然后从开始菜单启动 SoundNinja。

## macOS

- 适用于 Apple Silicon 和 Intel 的 **通用 .dmg**
- 落地页目标：macOS 10.14 及更新
- 应用 README 列出 macOS 10.15 及以上

打开磁盘映像，将 SoundNinja 拖到“应用程序”。

## Linux

- Ubuntu / Debian 的 **Debian (.deb)**
- 其他发行版的 **AppImage**
- 发布后，Arch 用户可从 AUR 安装 `soundninja-bin`
- 已在 Ubuntu 20.04 上测试

桌面构建需要 WebKit 和 GTK 软件包。在 Debian/Ubuntu 上通常是 `libwebkit2gtk`、`libgtk-3-0` 以及 app-indicator 库。

## 更新

在应用中打开 **帮助 → 检查更新**。也可以在设置中让 SoundNinja 启动时检查。

## 从源码构建

需要自定义构建？克隆[仓库](https://github.com/marcus-universe/SoundNinja)，安装 [Tauri 前置条件](https://v2.tauri.app/start/prerequisites/)，然后：

```bash
bun install
bun run tauri:serve
```

`npm`、`pnpm` 或 `deno` 也可以。生产构建：`bun run tauri:build`。
