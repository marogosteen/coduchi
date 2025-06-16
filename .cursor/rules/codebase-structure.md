# Coduchi プロジェクト構造ガイド (オニオンアーキテクチャ + DIP)

## 概要
CoduchiはDev Containerの設定ファイル（devcontainer.json、compose.yaml、Dockerfile）を自動生成するRustクライアントツールです。
**軽量オニオンアーキテクチャ + 依存性逆転の原則（DIP）**を採用し、高い保守性と拡張性を実現しています。

**ファイル出力先**: 指定されたプロジェクトディレクトリ内の`.devcontainer/`サブディレクトリに設定ファイルを生成します。

## 🏗️ アーキテクチャ概要

### 軽量オニオンアーキテクチャ + DIP
```
┌─────────────────────────────────────┐  
│  Infrastructure + Presentation      │  │ 依存方向
│   (Repositories, UI, CLI, DI)       │  │ （上から下へ）
├─────────────────────────────────────┤  │
│        Application Layer            │  │
│         (Use Cases)                 │  │  
├─────────────────────────────────────┤  │
│         Domain Layer                │  │
│   (Models, Ports, Services)         │  ▼
└─────────────────────────────────────┘
```

**依存関係の方向**: 上から下へ（すべてDomainに向かう）
- **Infrastructure/Presentation** → **Application** → **Domain**
- **DIPの実現**: Infrastructure層がDomain層のポート（抽象）に依存

## 📁 ディレクトリ構造詳細

```
src/
├── domain/                    # ドメイン層（ビジネスロジックの中核）
│   ├── models/               # ドメインモデル・エンティティ
│   ├── ports/                # 抽象インターフェース（DIP）
│   └── services/             # ドメインサービス（純粋な業務ロジック）
├── application/              # アプリケーション層（ユースケース）
│   ├── use_cases/           # 具体的なユースケース実装
│   └── services/            # アプリケーションサービス（将来拡張用）
├── infrastructure/          # インフラ層（外部システム依存）
│   ├── repositories/        # データアクセス実装
│   └── ui/                  # ユーザーインターフェース実装
├── presentation/            # プレゼンテーション層（エントリーポイント）
│   ├── cli.rs              # CLI引数定義
│   └── container.rs        # DI容器（依存性注入）
├── lib.rs                  # ライブラリエントリーポイント
├── main.rs                 # バイナリエントリーポイント
└── error.rs                # レガシー互換エラー定義
```

### 📍 各層の責務

#### **Domain Layer (`src/domain/`)**
**最も内側の層** - 外部依存ゼロ、ビジネスロジックの中核

- **`models/`** - ドメインエンティティとビジネスオブジェクト
  - `ComposeConfig`: Docker Compose設定（CLI引数に基づく）
  - `DevContainerConfig`: Dev Container設定
  - `ComposeConfigBuilder`: 設定構築ビルダーパターン
  - `DevContainerTemplate`, `ImageConfiguration`: Dev Containerテンプレート情報
  - `GeneratedFile`: 生成ファイル表現
  - `GenerationResult`: ファイル生成結果

- **`ports/`** - 抽象インターフェース（DIPの核心）
  - `DevContainerTemplateRepository`: Dev Containerテンプレート取得抽象
  - `FileRepository`: ファイル操作抽象
  - `UserInteraction`: ユーザー対話抽象
  - `ProgressReporter`: 進捗報告抽象

- **`services/`** - ドメインサービス
  - `DevContainerGenerator`: ファイル生成の純粋ロジック

#### **Application Layer (`src/application/`)**
**ユースケースの実装** - ドメインの協調によりビジネス機能を実現

- **`use_cases/`** - 具体的なユースケース
  - `GenerateDevContainerUseCase`: メインの生成ワークフロー
  - リクエスト/レスポンスDTO定義

- **`services/`** - アプリケーションサービス（将来拡張用）

#### **Infrastructure Layer (`src/infrastructure/`)**
**外部システム依存の実装** - ポート（抽象）の具象実装

- **`repositories/`** - データアクセス実装
  - `GitHubTemplateRepository`: GitHub API経由のDev Containerテンプレート取得実装
  - `FileSystemRepository`: ファイルシステム実装

- **`ui/`** - ユーザーインターフェース実装
  - `CliUserInteraction`: CLI対話実装
  - `ConsoleProgressReporter`: コンソール進捗表示

#### **Presentation Layer (`src/presentation/`)**
**エントリーポイントと依存性組み立て**

- **`cli.rs`** - CLI引数パース定義
- **`container.rs`** - DI容器（依存性注入管理）

## 🔄 DIPによる処理フロー

### メイン実行フロー
```
main.rs → lib.rs → Container → UseCase → Domain Services
                        ↓
                 Infrastructure実装注入
```

1. **CLI解析** (`presentation/cli.rs`)
2. **DI容器作成** (`presentation/container.rs`)
3. **依存性注入** (Infrastructure → Domain ports)
4. **ユースケース実行** (`application/use_cases/`)
5. **ドメインサービス協調** (`domain/services/`)
6. **インフラ実装呼び出し** (`infrastructure/`)

### 依存性逆転の実現
```rust
// ❌ 従来: 高レベルが低レベルに依存
UseCase → GitHubRepository (具象)

// ✅ DIP: 高レベルが抽象に依存、低レベルが抽象を実装
UseCase → TemplateRepository (抽象) ← GitHubTemplateRepository (具象)
```

## 📋 主要な型とトレイト

### Domain Layer

#### ドメインモデル (`domain/models/`)
```rust
pub struct ComposeConfig {
    pub dir: PathBuf,
    pub name: String,
    pub container_name: String,
    pub dir_name: String,
    pub image_name: String,
    pub base_image: String,
    pub force: bool,
}

pub struct DevContainerConfig {
    pub name: String,
    pub workspace_folder: String,
}

pub struct ComposeConfigBuilder {
    // ビルダーパターンで安全な設定構築
}

pub struct GeneratedFile {
    pub filename: String,
    pub content: String,
}

pub struct GenerationResult {
    pub files: Vec<GeneratedFile>,
    pub success: bool,
    pub message: String,
}
```

#### ポート定義 (`domain/ports/`)
```rust
#[async_trait]
pub trait DevContainerTemplateRepository: Send + Sync {
    async fn fetch_templates(&self) -> Result<Vec<DevContainerTemplate>>;
}

#[async_trait]
pub trait FileRepository: Send + Sync {
    fn write_files(&self, config: &ComposeConfig, files: Vec<GeneratedFile>) -> Result<Vec<String>>;
    fn confirm_overwrite_if_needed(&self, config: &ComposeConfig) -> Result<bool>;
}

#[async_trait]
pub trait UserInteraction: Send + Sync {
    async fn select_base_image(&self, templates: Vec<DevContainerTemplate>) -> Result<String>;
    fn show_progress(&self, message: &str);
}

#[async_trait]
pub trait ProgressReporter: Send + Sync {
    fn report_file_generated(&self, filename: &str);
    fn report_completion(&self);
}
```

### Application Layer

#### ユースケース (`application/use_cases/`)
```rust
pub struct GenerateDevContainerUseCase {
    template_repo: Arc<dyn DevContainerTemplateRepository>,
    file_repo: Arc<dyn FileRepository>,
    user_interaction: Arc<dyn UserInteraction>,
    progress_reporter: Arc<dyn ProgressReporter>,
}

impl GenerateDevContainerUseCase {
    pub async fn execute(&self, request: GenerateDevContainerRequest) 
        -> Result<GenerateDevContainerResponse> {
        // 純粋にポート（抽象）を通じてビジネスロジックを実行
    }
}
```

### Infrastructure Layer

#### 具象実装 (`infrastructure/repositories/`)
```rust
pub struct GitHubTemplateRepository {
    client: reqwest::Client,
}

#[async_trait]
impl DevContainerTemplateRepository for GitHubTemplateRepository {
    async fn fetch_templates(&self) -> Result<Vec<DevContainerTemplate>> {
        // GitHub API具象実装
    }
}

pub struct FileSystemRepository;

#[async_trait]
impl FileRepository for FileSystemRepository {
    fn write_files(&self, config: &ComposeConfig, files: Vec<GeneratedFile>) -> Result<Vec<String>> {
        // .devcontainer/ディレクトリへのファイルシステム具象実装
        // ディレクトリ自動作成機能を含む
    }
}
```

### Presentation Layer

#### DI容器 (`presentation/container.rs`)
```rust
pub struct Container {
    template_repo: Arc<dyn DevContainerTemplateRepository>,
    file_repo: Arc<dyn FileRepository>,
    user_interaction: Arc<dyn UserInteraction>,
    progress_reporter: Arc<dyn ProgressReporter>,
}

impl Container {
    pub fn create() -> Self {
        // 本番環境用の具象実装を注入
        Self {
            template_repo: Arc::new(GitHubTemplateRepository::new()),
            file_repo: Arc::new(FileSystemRepository::new()),
            user_interaction: Arc::new(CliUserInteraction::new(/* ... */)),
            progress_reporter: Arc::new(ConsoleProgressReporter::new()),
        }
    }
    
    pub fn create_for_test() -> Self {
        // テスト用のモック実装を注入
    }
}
```

## 🎯 DIPアーキテクチャの利点

### 1. **完全な依存性逆転**
- **高レベルモジュール**（UseCase）が**低レベルモジュール**（Infrastructure）に依存しない
- 抽象（ポート）を通じた疎結合設計
- インフラ変更がビジネスロジックに影響しない

### 2. **優れたテスタビリティ**
- すべての外部依存をモック可能
- 各層の単体テストが容易
- 統合テストでの依存制御が簡単

### 3. **高い保守性**
- 各層の責務が明確に分離
- 変更の影響範囲が予測可能
- 新機能追加時の設計指針が明確

### 4. **拡張性**
- 新しいポート追加で機能拡張
- 複数のインフラ実装の切り替え可能
- プラグイン的な設計が可能

## 🧪 テスト戦略

### Unit Tests by Layer

#### Domain Layer Tests
```rust
// 純粋関数のテスト（外部依存なし）
#[test]
fn test_compose_config_builder() {
    let config = ComposeConfigBuilder::new(PathBuf::from("test"))
        .with_name(Some("container".to_string()))
        .build("ubuntu:latest".to_string());
    
    assert_eq!(config.container_name, "container");
}
```

#### Application Layer Tests
```rust
// モックを使用したユースケーステスト
#[tokio::test]
async fn test_generate_devcontainer_use_case() {
    let use_case = GenerateDevContainerUseCase::new(
        Arc::new(MockDevContainerTemplateRepository::new()),
        Arc::new(MockFileRepository::new()),
        Arc::new(MockUserInteraction::new()),
        Arc::new(MockProgressReporter::new()),
    );
    
    let response = use_case.execute(request).await.unwrap();
    assert!(response.success);
}
```

#### Infrastructure Layer Tests
```rust
// 具象実装の単体テスト
#[test]
fn test_filesystem_repository() {
    let repo = FileSystemRepository::new();
    let config = ComposeConfig::new(/* ... */);
    let result = repo.write_files(&config, files);
    assert!(result.is_ok());
}
```

### Integration Tests
```rust
// 全体フロー統合テスト
#[tokio::test]
async fn test_full_generation_flow() {
    let container = Container::create_for_test();
    let use_case = container.create_use_case();
    
    let response = use_case.execute(request).await.unwrap();
    assert!(response.success);
}
```

## 📦 依存関係管理

### Layer Dependencies
```toml
[dependencies]
# Domain Layer: 外部crate依存最小限
serde = { version = "1.0", features = ["derive"] }
async-trait = "0.1"

# Infrastructure Layer: 外部システム連携
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.0", features = ["full"] }
inquire = "0.6"
colored = "2.0"

# Application Layer: ドメインとインフラの仲介
anyhow = "1.0"

# Presentation Layer: CLI・DI
clap = { version = "4.0", features = ["derive"] }
```

### Dependency Flow
```
Infrastructure + Presentation
            ↓
        Application
            ↓
          Domain
```

**実際の依存関係**:
- `Infrastructure` → `Domain` (ports)
- `Presentation` → `Application` + `Infrastructure`  
- `Application` → `Domain` (models, ports, services)

## 🚀 将来の拡張ポイント

### 短期的な拡張
- **新しいポート追加**: `ConfigValidator`, `DevContainerTemplateCache`
- **複数インフラ実装**: Azure DevOps, GitLab CI対応
- **設定フォーマット拡張**: YAML, TOML設定サポート

### 長期的な拡張
- **プラグインシステム**: 動的ポート実装ロード
- **イベント駆動**: ドメインイベントによる拡張
- **マイクロサービス**: 各ユースケースの独立サービス化

### 新機能追加のガイドライン
1. **新しいビジネス機能** → `domain/ports/`に抽象定義
2. **外部システム連携** → `infrastructure/`に具象実装
3. **ワークフロー追加** → `application/use_cases/`に追加
4. **UI変更** → `presentation/`層で対応

## 🔧 開発時の指針

### AIペアプログラミング時の参照ポイント

#### 新機能開発
1. **ドメイン設計**: `domain/models/`でエンティティ定義
2. **ポート定義**: `domain/ports/`で抽象インターフェース
3. **ユースケース**: `application/use_cases/`でワークフロー
4. **インフラ実装**: `infrastructure/`で具象実装
5. **DI設定**: `presentation/container.rs`で依存注入

#### バグ修正
1. **ビジネスロジック**: `domain/services/`を確認
2. **ワークフロー**: `application/use_cases/`を確認
3. **外部連携**: `infrastructure/`を確認
4. **エントリーポイント**: `presentation/`を確認

#### リファクタリング
- 各層の責務を明確に保つ
- 依存関係の方向（外→内）を守る
- ポート（抽象）経由の呼び出しを維持

## 📚 アーキテクチャ原則

### SOLID原則の実践
- **S**ingle Responsibility: 各層・クラスが単一責務
- **O**pen/Closed: ポートによる拡張性
- **L**iskov Substitution: トレイト実装の置換可能性
- **I**nterface Segregation: 細分化されたポート定義
- **D**ependency Inversion: **完全なDIP実装**

### クリーンアーキテクチャ準拠
- **依存関係ルール**: 内側の層は外側を知らない
- **境界の明確化**: レイヤー間はポートでのみ通信
- **フレームワーク独立**: ビジネスロジックがフレームワークに依存しない

このオニオンアーキテクチャ+DIPの実装により、Coduchiは**高い保守性・拡張性・テスタビリティ**を実現しています。 