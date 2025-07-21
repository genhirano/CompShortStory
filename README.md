# AI対抗 短編小説コンテスト!
![image](https://github.com/user-attachments/assets/f21b8d57-a673-462d-89e2-c4ffbb74bfd8)

各AIが同じプロンプトで生成する超短編小説を読み比べできるWebアプリケーション  
URL : [https://comp-short-story-v4p5.shuttle.app/](https://comp-short-story-v4p5.shuttle.app/)

## 📖 サイト概要

### 🎯 コンセプト
「AI対抗 短編小説コンテスト」は、複数の生成AIが全く同じプロンプト（指示文）を与えられた時に、どのような異なる短編小説を生成するかを比較検討できるユニークなWebサービスです。

### ✨ 主な特徴

- **📅 毎日更新**: 新しいプロンプトによる小説が毎日追加されます
- **🤖 5つのAI参戦**: Chat-GPT、Claude、Gemini、Copilot、DeepSeekが同じ土俵で競い合い
- **📱 レスポンシブデザイン**: PCでもスマートフォンでも快適に読める設計
- **🔄 過去作品閲覧**: 前後のボタンで過去の作品群を自由に閲覧可能
- **🎨 読みやすいUI**: 小説の読書に集中できるシンプルで洗練されたデザイン

### 🎪 使い方

1. **Webサイトにアクセス**: [https://comp-short-story-v4p5.shuttle.app/](https://comp-short-story-v4p5.shuttle.app/)
2. **今日の作品を読む**: 最新の作品が自動で表示されます
3. **AIごとの違いを楽しむ**: 同じプロンプトに対する各AIの個性的な解釈を比較
4. **過去作品を探索**: 「前へ」「次へ」ボタンで過去の作品を閲覧
5. **お気に入りを発見**: 気に入った作品や面白い比較結果をチェック

### 🎭 何が面白いのか？

- **創造性の違い**: 同じ指示でも、AIによって全く異なるストーリー展開
- **文体の個性**: 各AIが持つ独特の文章スタイルや表現力の違い
- **アプローチの多様性**: シリアス、コミカル、詩的など、様々な解釈手法
- **日々の発見**: 毎日新しいプロンプトで予想外の作品に出会える

## エントリー 生成AI
* [Chat-GPT](https://chatgpt.com/) - OpenAI
* [Claude](https://claude.ai/chats) - Anthropic
* [Gemini](https://gemini.google.com/) - Google
* [Copilot](https://copilot.microsoft.com/) - Microsoft
* [DeepSeek](https://chat.deepseek.com/) - DeepSeek

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

### 🏗️ アーキテクチャ概要

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│   ユーザー        │◄──►│  Webアプリ     │◄──►│   microCMS      │
│  (ブラウザ)       │    │   (Rocket)    │    │   (API)         │
└─────────────────┘    └──────────────┘    └─────────────────┘
                              │
                              ▼
                      ┌──────────────┐
                      │   Shuttle     │
                      │  (ホスティング) │
                      └──────────────┘
```

**データフロー**:
1. ユーザーがWebページにアクセス
2. RocketアプリがmicroCMS APIから最新の小説データを取得
3. 取得したデータをTeraテンプレートで整形
4. レスポンシブなHTMLとしてユーザーに配信

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

# 特定テストの実行
cargo test test_json
```

### 📦 デプロイメント

本アプリケーションは[Shuttle](https://shuttle.rs/)を使用してデプロイされています。

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

### 🤝 開発への貢献

1. フォークしてクローン
2. フィーチャーブランチを作成
3. 変更をコミット
4. テストが通ることを確認
5. プルリクエストを作成

---

## 📄 ライセンス

このプロジェクトのライセンスについては、リポジトリをご確認ください。
