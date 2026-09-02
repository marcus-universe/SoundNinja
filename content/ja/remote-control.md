---
title: リモートコントロール
description: ローカルネットワーク経由でBitfocus CompanionからSoundNinjaを操作。
order: 7
---

# リモートコントロール

SoundNinjaはローカルネットワークで待ち受け、Bitfocus Companion（または任意のHTTP/WebSocketクライアント）がサウンドを再生・停止できます。

## リモートサーバーを有効化

1. **設定 → リモート** を開きます。
2. リモートサーバーを有効にします。デフォルトポートは **7331** です。
3. 任意：トークンを設定。クライアントはBearerトークンまたは `?token=` で送ります。
4. `http://IP:PORT` URLをコピーするか、**設定 → 情報** からシステムIPをコピーします。

Windowsはサーバー初回起動時にファイアウォールの確認を出すことがあります。プライベートネットワークのアクセスを許可してください。

## Bitfocus Companion

1. [Bitfocus Companion](https://bitfocus.io/companion)をインストールします。
2. [companion-module-soundninja.tgz](https://github.com/marcus-universe/SoundNinja/releases/latest/download/companion-module-soundninja.tgz) をダウンロード。Companion Storeのパッケージはまだありません。
3. Companionで **Modules → Load module package** を開き、`.tgz` を選びます。
4. **Sound Ninja** 接続を追加。PCのIP、ポート、設定したトークンを貼り付けます。

## アクション

- **Trigger Sound** — ライブドロップダウンから選ぶか、8文字のサウンドIDを入力（Companion変数が使えます）
- **Stop Sound** — IDで再生中のサウンドを1つ停止
- **Stop All** — 再生中の全サウンドを停止

SoundNinjaでは、ボタンのコンテキストメニューまたは複数選択のIDチップからサウンドIDをコピーします。

## フィードバックと変数

- **Sound Playing** — そのサウンドIDがアクティブな間はtrue（ボタン色に使う）
- `$(soundninja:connected)` — `true` / `false`
- `$(soundninja:playing_count)` — 再生中のサウンド数
- `$(soundninja:last_triggered)` — 最後にトリガーしたサウンドID

## HTTP API（上級）

ベースパス：`/api/v1`

- `GET /info` — アプリ名、バージョン、プロトコル、トークンが必要か
- `GET /sounds` — サウンド一覧（id、name、tabs、active）
- `GET /state` — サウンドと現在再生中のID
- `POST /trigger` に `{ "id" }` — サウンドを再生
- `GET /trigger/:id` — 再生（ブラウザで簡単にテスト）
- `POST /stop` に任意の `{ "id" }` — 1つまたはすべて停止
- `GET /ws` — ライブ状態。trigger/stopコマンドを受信
