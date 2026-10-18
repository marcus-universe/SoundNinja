# Sound Ninja

[English](./README.md) · [Deutsch](./README.de.md) · [Español](./README.es.md) · [Français](./README.fr.md) · **日本語** · [简体中文](./README.zh-Hans.md)

![Sound Ninja Logo Animated](./designs/Logo_Animated.gif)

<div align="center">

[![Website](https://img.shields.io/badge/Website-soundninja.marcus--universe.com-29d4ff?style=for-the-badge)](https://soundninja.marcus-universe.com/)
[![Docs](https://img.shields.io/badge/Docs-User_guide-29d4ff?style=for-the-badge)](https://soundninja.marcus-universe.com/docs)
<br>
![License](https://img.shields.io/github/license/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Version](https://img.shields.io/github/package-json/v/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Stars](https://img.shields.io/github/stars/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Forks](https://img.shields.io/github/forks/marcus-universe/SoundNinja.svg)

</div>
<b>Sound Ninja</b> は、自分だけのサウンドボードを作るための、最大限カスタマイズできる <b>オープンソースのサウンドボードアプリ</b> です。**ライブ配信、友だちとのチャット、ポッドキャスト収録**に向いています。😉👍


## 🥷 なぜ SoundNinja？
SoundNinja の柱は、高い性能と深いカスタマイズです。

既存のサウンドボードアプリの多くは、小さい画面や VR の中では読みにくく、操作もぎこちないです。SoundNinja は、自分の環境に合わせられる、アクセスしやすく、完全にオープンソースな解決策です。

### 🚀 性能が先
Rust の **Tauri v2** と、柔軟な **Nuxt 4** フロントエンドで作られています。SoundNinja はとても軽いです。重い代替アプリと違い、ディスク使用量は小さく、RAM と CPU もほとんど使いません。

### 🎯 速さとカスタマイズのために
大きなライブラリを効率よく整理できるサウンドボードが目標です。必要な音を、必要な瞬間に、ミリ秒で見つけられます。


| 機能                         | 説明                                                      | 状態 |
| ---------------------------- | --------------------------------------------------------- | ------ |
| **サウンドボード**           | お気に入りの音で自分のボードを作る                        | ✅     |
| **タブ**                     | 音をタブで整理する                                        | ✅     |
| **検索バー**                 | 音をすばやく簡単に見つける                                | ✅     |
| **テーマ**                   | 好みの見た目にデザインする                                | ✅     |
| **カスタムプロファイル**     | 設定の違うサウンドボードを複数作る                        | ✅     |
| **レコーダー**               | PC の音声を Sound Ninja の中で直接録音する                | ✅     |
| **GIF と画像**               | 音に画像や GIF を付ける                                   | ✅     |
| **カスタムホットキー**       | ホットキーを自分用に変える                                | ✅     |
| **Companion リモコン**       | Bitfocus Companion から HTTP/WebSocket で音を鳴らす       | ✅     |
| **サウンドボード共有**       | ボードを友だちと共有する                                  | ✅     |
| **インポート / エクスポート** | サウンドボードを読み書きする                             | ✅     |
| **タグ**                     | お気に入りの音をカテゴリでタグ付けする                    | ✅     |
| **MIDI 対応**                | MIDI でサウンドボードを操作する                           | ⛔     |
| **音声検索**                 | 声で音を探す                                              | ⛔     |
| **Twitch チャット操作**      | コミュニティが再生する音を決める                          | ⛔     |
| **AI アシスタント**          | 今の場面に合う音を提案する                                | ⛔     |

## ドキュメント

- [ウェブサイト](https://soundninja.marcus-universe.com/)
- [ユーザーガイド](https://soundninja.marcus-universe.com/docs)
- [ソースからビルド](./docs/build-from-source.md)（英語）
- [開発環境](./docs/development.md)（英語）
- [Companion / Remote](./docs/companion.md)（英語）
- [ステム分離モデル](./docs/stems-model.md)（英語）
- [アプリのアップデーター](./docs/updater.md)（英語）

---
<img src="./designs/download.png" alt="ダウンロード" width="200" />

[![Windows NSIS](https://img.shields.io/badge/Windows-NSIS-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-windows-x64-setup.exe)
<br>
<br>
[![macOS Apple Silicon](https://img.shields.io/badge/macOS-Apple_Silicon-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-macos-arm64.dmg)
[![macOS Intel](https://img.shields.io/badge/macOS-Intel-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-macos-x64.dmg)
<br>
<br>
[![Linux deb](https://img.shields.io/badge/Linux-.deb-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-linux-amd64.deb)
[![Linux AppImage](https://img.shields.io/badge/Linux-AppImage-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-linux-amd64.AppImage)
[![Arch](https://img.shields.io/badge/Arch-pkg.tar.zst-1793D1?style=for-the-badge&logo=archlinux&logoColor=white)](#arch-linux)
<br>
<br>
[![Companion](https://img.shields.io/badge/Companion-Module-111111?style=for-the-badge)](https://github.com/marcus-universe/companion-module-soundninja/releases/latest)
<br>
<br>
[![Build from source](https://img.shields.io/badge/Build-from_source-29d4ff?style=for-the-badge)](./docs/build-from-source.md)
<br>
<br>

アプリのインストーラーは `/releases/latest/download/…` にあります。Companion モジュールの `.tgz` は [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja/releases/latest) のリリースページにあります。

### Arch Linux

ビルド済みの `soundninja-bin` パッケージ（`x86_64` と `aarch64`）は GitHub Releases にあります。[aur.archlinux.org](https://aur.archlinux.org) には**まだない**ので、`yay -S soundninja` では見つかりません。curl でインストールします。

```bash
arch=$(uname -m)   # x86_64 or aarch64
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-bin-[^"]+-'"${arch}"'\.pkg\.tar\.zst' | head -n1)
curl -fLO "$url"
sudo pacman -U ./soundninja-bin-*-"${arch}".pkg.tar.zst
```

または AUR ソースの tarball からビルドします（`makepkg`）。

```bash
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-aur-[^"]+\.tar\.gz' | head -n1)
curl -fLO "$url"
mkdir -p soundninja-bin && tar xf soundninja-aur-*.tar.gz -C soundninja-bin
cd soundninja-bin && makepkg -si
```

**Actions → Release** は、GitHub Release の**下書き**（`vX.Y.Z`）がなければ作り、まだ下書きにないプラットフォームだけを添付します。一つの対象を再ビルドするには **Release Windows**、**Release Linux**、**Release macOS ARM**、**Release macOS Intel** を使います。各アクションはインストーラー（と安定した別名）をその下書きにアップロードします。Companion モジュール: [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja)（`SoundNinja-Companion.tgz` と `SoundNinja-Companion-{Version}.tgz`）。

## 対応プラットフォーム

公式ビルド（特記がない限り x86_64）:

| プラットフォーム | 対応 |
| :------- | :------ |
| Windows 10+ | NSIS インストーラー（WebView2） |
| macOS 10.15+ | アーキテクチャごとの DMG（Apple Silicon、Intel） |
| Linux x86_64 | `.deb`（Debian/Ubuntu、WebKitGTK 4.1）または AppImage。音声は PipeWire または PulseAudio。 |
| Arch Linux | GitHub Releases の `soundninja-bin`（`pacman -U`、x86_64 + aarch64）。aur.archlinux.org にはまだありません。 |
| Linux ARM | `arm64` / `armhf` 向け `.deb`。 |

開発初期です。すべてのディストリビューションではテストしていません。直せるように、問題があれば報告してください。

## クレジット

- [lucidrains/BS-RoFormer](https://github.com/lucidrains/BS-RoFormer) — Band-Split RoPE Transformer。Record Editor の AI ステム分離を支えるアーキテクチャ。MIT ライセンス。
- 重みは viperx が学習。ONNX 書き出しは [ZFTurbo/MSS_ONNX_TensorRT](https://github.com/ZFTurbo/MSS_ONNX_TensorRT)。
- [KLIPY](https://klipy.com) — オンライン GIF 検索（自分の API キー）。
- [GifSnap](https://gifsnap.com) — キー不要の GIF 検索（公開 API、best effort）。リクエストを減らすため、検索結果はローカルにキャッシュします。
