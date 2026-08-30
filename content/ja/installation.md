---
title: インストール
description: Windows、macOS、LinuxにSoundNinjaをインストール。
order: 2
---

# インストール

[ダウンロードページ](/download)または[GitHub Releases](https://github.com/marcus-universe/SoundNinja/releases/latest)からビルドを入手してください。SoundNinjaは初期開発中なので、ビルドはベータとして扱ってください。

## Windows

- 64ビットWindows向けの**インストーラー (.exe)** または **MSI (.msi)**
- ランディングページの対象：Windows 10以降
- アプリのREADMEはWindows 8以上も記載

インストーラーを実行し、スタートメニューからSoundNinjaを起動します。

## macOS

- Apple SiliconとIntel向けの**ユニバーサル .dmg**
- ランディングページの対象：macOS 10.14以降
- アプリのREADMEはmacOS 10.15以上を記載

ディスクイメージを開き、SoundNinjaをアプリケーションにドラッグします。

## Linux

- Ubuntu / Debian向けの**Debian (.deb)**
- その他のディストリ向けの**AppImage**
- リリース公開後、ArchユーザーはAURから`soundninja-bin`をインストールできます
- Ubuntu 20.04でテスト済み

デスクトップビルドにはWebKitとGTKのパッケージが必要です。Debian/Ubuntuでは通常、`libwebkit2gtk`、`libgtk-3-0`、app-indicatorライブラリです。

## アップデート

アプリ内で **ヘルプ → アップデートを確認** を開きます。設定で起動時にSoundNinjaが確認するようにもできます。

## ソースからビルド

カスタムビルドが必要なら、[リポジトリ](https://github.com/marcus-universe/SoundNinja)をクローンし、[Tauriの前提条件](https://v2.tauri.app/start/prerequisites/)を入れたうえで：

```bash
bun install
bun run tauri:serve
```

`npm`、`pnpm`、`deno`でも動きます。本番ビルド：`bun run tauri:build`。
