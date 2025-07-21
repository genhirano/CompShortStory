# 生成AI対抗 短編小説コンテスト!

URL : [https://comp-short-story-v4p5.shuttle.app/](https://comp-short-story-v4p5.shuttle.app/)

![image](https://github.com/user-attachments/assets/f21b8d57-a673-462d-89e2-c4ffbb74bfd8)

## 📖 サイト概要

### 🎯 コンセプト
「AI対抗 短編小説コンテスト」は、５つの生成AIが、同じ短編小説執筆指示プロンプトで執筆したもの、それぞれを読み比べできるなWebアプリです。

### ✨ 主な特徴

- **🤖 5つの生成AIが参戦**: Chat-GPT、Claude、Gemini、Copilot、DeepSeekが同じ土俵で競い合い
- **📅 (ほぼ)毎日更新**: 新しいプロンプトによる超短編小説がほぼ毎日更新されます

## エントリー 生成AI
* [Chat-GPT](https://chatgpt.com/) - OpenAI
* [Claude](https://claude.ai/chats) - Anthropic
* [Gemini](https://gemini.google.com/) - Google
* [Copilot](https://copilot.microsoft.com/) - Microsoft
* [DeepSeek](https://chat.deepseek.com/) - DeepSeek

### 🎪 使い方

1. **Webサイトにアクセス**: [https://comp-short-story-v4p5.shuttle.app/](https://comp-short-story-v4p5.shuttle.app/)
2. **今日の作品を読む**: 最新の作品が自動で表示されます
3. **AIごとの違いを楽しむ**: 同じプロンプトに対する各AIの個性的な解釈を比較
4. **過去作品を探索**: 「前へ」「次へ」ボタンで過去の作品を閲覧

### 🎭 見どころ

- **創造性の違い**: 同じ指示（プロンプト）でも、生成AIによって全く異なるストーリー展開
- **文体の個性**: 各AIが持つ独特の文章スタイルや表現力の違い
- **アプローチの多様性**: シリアス、コミカル、詩的など、プロンプトの様々な解釈

## イメージ
![image](https://github.com/genhirano/CompShortStory/assets/3538386/9684afc3-a316-41e4-a63d-30d445c465a6)

---

## 🔧 技術的要素

### 📋 技術スタック

**バックエンド**
- **言語**: Rust (Edition 2021)
- **Webフレームワーク**: Rocket 0.5.1
  - JSON対応
  - TLS対応
  - テンプレートエンジン (Tera, Handlebars)
- **ホスティング**: Shuttle (Rust専用クラウドプラットフォーム)

**フロントエンド**
- **テンプレート**: Tera テンプレートエンジン
- **CSS**: Bulma フレームワーク
- **JavaScript**: バニラJS (軽量な非同期処理)
- **レスポンシブデザイン**: モバイルファースト

**データ・API**
- **CMS**: microCMS (ヘッドレスCMS)
- **HTTP クライアント**: reqwest 0.12.4
- **シリアライゼーション**: serde + serde_json
- **日時処理**: chrono + chrono-tz (JST対応)

**開発・デプロイメント**
- **CORS対応**: rocket_cors
- **非同期処理**: tokio
- **エラーハンドリング**: anyhow

### 🚀 開発環境のセットアップ

#### 前提条件
- Rust (最新安定版)
- Git

#### インストール手順

```bash
# リポジトリのクローン
git clone https://github.com/genhirano/CompShortStory.git
cd CompShortStory

# 依存関係のインストール
cargo build

# テストの実行
cargo test

# 開発サーバー起動（要: microCMS APIキー）
cargo run
```

#### 環境変数の設定

本アプリケーションはmicroCMSからデータを取得するため、APIキーが必要です：

```bash
# Shuttleでのデプロイ時
shuttle secrets set MICROCMS_KEY=your_microcms_api_key
```

### 📡 API エンドポイント

#### GET `/`
- **説明**: メインページ（最新の小説を表示）
- **レスポンス**: HTML

#### POST `/`
- **説明**: ページネーション操作
- **パラメータ**: 
  - `direction`: "next" | "prev"
  - `currentoffset`: 現在のオフセット値
- **レスポンス**: HTML

#### GET `/api`
- **説明**: JSON API（プログラム的アクセス用）
- **パラメータ**: 
  - `direction`: "next" | "prev" | "now"  
  - `currentoffset`: 現在のオフセット値
- **レスポンス**: JSON

```json
{
  "title": "作品タイトル",
  "version": "2024-01-01",
  "chatgpt": ["小説本文の配列"],
  "claude": ["小説本文の配列"],
  "gemini": ["小説本文の配列"],
  "copilot": ["小説本文の配列"],
  "deepseek": ["小説本文の配列"],
  "prompt": ["プロンプト文の配列"],
  "totalcount": 総記事数,
  "offset": 現在位置,
  "has_next": 次ページの有無,
  "has_prev": 前ページの有無
}
```

### 🧪 テスト

```bash
# 全テストの実行
cargo test
```

### 📦 デプロイメント

本アプリケーションは[Shuttle](https://shuttle.rs/)を使用してデプロイ・ホスティングされています。

```bash
# Shuttleでのデプロイ
shuttle deploy

# シークレットの設定
shuttle secrets set MICROCMS_KEY=your_api_key
```

### 🔧 主要な設定ファイル

- `Cargo.toml`: Rustの依存関係設定
- `Shuttle.toml`: Shuttle固有の設定
- `Rocket.toml`: Rocket フレームワーク設定

---

## 📄 ライセンス

このプロジェクトのライセンスについては、リポジトリをご確認ください。
