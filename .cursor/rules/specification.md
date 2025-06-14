# coduchi 仕様書

## 概要
VSCode Dev Containerの設定ファイルを自動生成するCLIツール

## 開発言語・アーキテクチャ
- **言語**: Rust
- **アーキテクチャ**: 軽量オニオンアーキテクチャ + 依存性逆転の原則（DIP）
- **設計パターン**: Domain-Driven Design（DDD）、SOLID原則準拠
- **テスト戦略**: 各層の単体テスト + 統合テスト、モック活用

### 技術スタック
- **CLI**: `clap` v4.0+ (derive feature)
- **エラーハンドリング**: `anyhow` + `thiserror`
- **非同期処理**: `tokio` + `async-trait`
- **HTTP通信**: `reqwest` (GitHub API)
- **対話式UI**: `inquire` v0.6+
- **カラー出力**: `colored` v2.0+
- **JSON処理**: `serde` + `serde_json`
- **Base64**: `base64` (テンプレート解析)

### アーキテクチャ設計原則
1. **依存性逆転**: 高レベルモジュールが低レベルモジュールに依存しない
2. **関心の分離**: 各層が明確な責務を持つ
3. **テスタビリティ**: すべての外部依存をモック可能
4. **拡張性**: 新機能追加時の影響範囲を最小化
5. **保守性**: コードの理解・変更が容易

### 層構造
```
Domain (models, ports, services)    # ビジネスロジック中核
    ↑
Application (use_cases)             # ワークフロー実装
    ↑  
Infrastructure (repositories, ui)   # 外部システム連携
    ↑
Presentation (cli, container)       # エントリーポイント・DI
```

## コマンド仕様
```bash
coduchi [options]
```

### オプション
- `-d, --dir <path>`: 設定ファイルを生成するディレクトリを指定（デフォルト: カレントディレクトリ）
- `-n, --name <name>`: devcontainer.jsonのnameプロパティを指定（デフォルト: ディレクトリ名）
- `--base-image <image>`: ベースイメージを指定（指定しない場合は対話式で選択）
- `-f, --force`: 既存ファイルの上書きを確認せずに実行

### オプションの詳細

#### `--force` オプション
- 既存のファイル（devcontainer.json、compose.yaml、Dockerfile）を強制的に上書きします
- 指定しない場合、既存ファイルが存在すると上書きしません
- ショートカット: `-f`

#### ディレクトリ指定（`--dir`）
- 設定ファイルを生成するディレクトリを指定します
- デフォルトはコマンドを実行したディレクトリ（cwd）です
- ショートカット: `-d`
- workspaceFolderは指定したディレクトリ名に基づいて自動設定されます
  - 例: `--dir myapp` → `"workspaceFolder": "/workspaces/myapp"`

#### 名前指定（`--name`）
- devcontainer.jsonのnameプロパティを指定します
- デフォルトは`--dir`で指定したディレクトリ名です
- ショートカット: `-n`

### 使用例

#### オプションを指定した場合
```bash
$ coduchi --name "My Dev Container" --dir my-directory
```

生成されるdevcontainer.json:
```json
{
  "name": "My Dev Container",
  "workspaceFolder": "/workspaces/my-directory"
}
```

#### デフォルト使用の場合
```bash
$ mkdir myapp
$ cd myapp
$ coduchi
```

生成されるdevcontainer.json:
```json
{
  "name": "myapp",
  "workspaceFolder": "/workspaces/myapp"
}
```

## 実装アーキテクチャ

### ワークフロー実装（Application Layer）
**GenerateDevContainerUseCase**が以下のフローを実装：

1. **設定構築** - `ConfigBuilder`による段階的な設定構築
2. **ベースイメージ選択** - `TemplateRepository`ポート経由
3. **上書き確認** - `FileRepository`ポート経由  
4. **ファイル生成** - `DevContainerGenerator`ドメインサービス
5. **書き込み実行** - `FileRepository`ポート経由
6. **進捗報告** - `ProgressReporter`ポート経由

### 外部依存の抽象化（Domain Ports）
```rust
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    async fn fetch_templates(&self) -> Result<Vec<Template>>;
}

#[async_trait] 
pub trait FileRepository: Send + Sync {
    fn write_files(&self, dir: &Path, files: Vec<GeneratedFile>) -> Result<Vec<String>>;
    fn confirm_overwrite_if_needed(&self, config: &AppConfig) -> Result<bool>;
}

#[async_trait]
pub trait UserInteraction: Send + Sync {
    async fn select_base_image(&self, templates: Vec<Template>) -> Result<String>;
    fn show_progress(&self, message: &str);
}

#[async_trait]
pub trait ProgressReporter: Send + Sync {
    fn report_file_generated(&self, filename: &str);
    fn report_completion(&self);
}
```

### 依存性注入（Presentation Layer）
```rust
// 本番環境構成
pub fn create() -> Container {
    Container {
        template_repo: Arc::new(GitHubTemplateRepository::new()),
        file_repo: Arc::new(FileSystemRepository::new()),  
        user_interaction: Arc::new(CliUserInteraction::new(/* ... */)),
        progress_reporter: Arc::new(ConsoleProgressReporter::new()),
    }
}

// テスト環境構成  
pub fn create_for_test() -> Container {
    Container {
        template_repo: Arc::new(MockTemplateRepository::new()),
        file_repo: Arc::new(MockFileRepository::new()),
        user_interaction: Arc::new(MockUserInteraction::new()),
        progress_reporter: Arc::new(MockProgressReporter::new()),
    }
}
```

## 生成するファイル

### 1. devcontainer.json
```json
{
  "name": "<name>",
  "dockerComposeFile": "compose.yaml",
  "workspaceFolder": "/workspaces/<dir-name>",
  "service": "app",
  "customizations": {
    "vscode": {
      "extensions": []
    }
  }
}
```

### 2. compose.yaml
```yaml
services:
  app:
    image: <name>:latest
    container_name: <name>
    build:
      context: .
      dockerfile: Dockerfile
    volumes:
      - ..:/workspaces/<dir-name>:cached
    command: sleep infinity
```

### 3. Dockerfile
```dockerfile
FROM <base-image>
```

## ベースイメージ選択

### アーキテクチャ上の実装
- **TemplateRepository**ポートによる抽象化
- **GitHubTemplateRepository**による具象実装
- **UserInteraction**ポートによる対話抽象化  
- **CliUserInteraction**による具象実装

### 選択フローの詳細

1. **Dev Container Template の一覧取得**
   - `GitHubTemplateRepository::fetch_templates()`実装
   - GitHub APIを使用して `https://api.github.com/repos/devcontainers/templates/contents/src` からテンプレート一覧を取得
   - レスポンスモデルは `doc/example/api/list_api_sample.json` を参照

2. **テンプレート選択**
   - `UserInteraction::select_base_image()`ポート経由
   - 取得したテンプレート一覧から `name` フィールドを表示
   - ユーザーに対話形式でテンプレートを選択させる

3. **テンプレート情報の取得**
   - 選択されたテンプレートの `path` フィールドを使用
   - `.devcontainer/devcontainer.json` から `image` フィールドを取得
   - `devcontainer-template.json` から `options.imageVariant` の情報を取得
     - `default` と `proposals` フィールドから利用可能なバリアント一覧を取得

4. **イメージバリアントの選択**
   - 取得したバリアント一覧を表示
   - デフォルトバリアントを明示
   - ユーザーに対話形式でバリアントを選択させる

5. **Dockerfileの生成**
   - `DevContainerGenerator::generate_dockerfile()`ドメインサービス
   - 選択されたテンプレートの `image` 文字列から `$` 以降を選択されたバリアントで置換
   - フォーマット: `FROM <置換後のイメージ文字列>`

## 拡張機能の設定
- 空の配列を設定
- ユーザーが生成後に必要に応じてカスタマイズ可能

## 対話式プロンプト
オプションが指定されていない場合、以下の項目を対話式で確認：

1. ベースイメージの選択
   - 一覧表示
2. 既存ファイルの上書き確認

## エラーハンドリング

### アーキテクチャ上のエラー処理
- **Domain Layer**: `anyhow::Result`による統一的エラー処理
- **Application Layer**: ユースケース内でのエラーハンドリング・変換
- **Infrastructure Layer**: 外部依存固有エラーの抽象化
- **Presentation Layer**: ユーザーフレンドリーなエラーメッセージ表示

### エラーカテゴリ
- **ファイル操作エラー**: 権限・書き込みエラー
- **設定バリデーションエラー**: 不正な設定値
- **API通信エラー**: GitHub API接続・レスポンスエラー
- **ユーザー入力エラー**: 不正な入力・キャンセル
- **システムエラー**: 予期しない内部エラー

## 出力・ログ

### 出力形式
- **成功時**: 生成されたファイルのパスを表示（緑色）
- **エラー時**: 具体的なエラーメッセージを表示（赤色）
- **警告時**: 警告メッセージを表示（黄色）
- **情報時**: 情報メッセージを表示（青色）

### メッセージの形式
- **エラー**: 赤色（例：ファイルの書き込みに失敗しました）
- **警告**: 黄色（例：既存のファイルが存在します）
- **成功**: 緑色（例：設定ファイルの生成が完了しました）
- **情報**: 青色（例：ベースイメージを選択してください）

### 進捗表示
- `ProgressReporter`ポートによる抽象化
- テンプレート取得中の進捗表示
- ファイル生成完了の報告
- 全体完了の通知

## テスト戦略

### 単体テスト
- **Domain Layer**: 純粋関数のテスト（外部依存なし）
- **Application Layer**: モックを使用したユースケーステスト
- **Infrastructure Layer**: 具象実装の個別テスト
- **Presentation Layer**: CLI解析・DI容器テスト

### 統合テスト
- エンドツーエンドのワークフローテスト
- モック環境での全体フロー確認
- エラーケースの網羅的テスト

### モック活用
- すべての外部依存（GitHub API、ファイルシステム、ユーザー入力）をモック化
- テスト環境での確実な動作検証
- CI/CD環境でのテスト実行
