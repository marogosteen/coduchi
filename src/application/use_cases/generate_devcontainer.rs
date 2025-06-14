use anyhow::Result;
use std::sync::Arc;
use crate::{
    domain::{
        models::ConfigBuilder,
        ports::{TemplateRepository, FileRepository, UserInteraction, ProgressReporter},
        services::DevContainerGenerator,
    },
};

/// Dev Container生成のリクエスト
#[derive(Debug)]
pub struct GenerateDevContainerRequest {
    pub dir: std::path::PathBuf,
    pub name: Option<String>,
    pub base_image: Option<String>,
    pub force: bool,
}

/// Dev Container生成のレスポンス
#[derive(Debug)]
pub struct GenerateDevContainerResponse {
    pub success: bool,
    pub generated_files: Vec<String>,
    pub message: String,
}

/// Dev Container生成のユースケース
/// DIPによりポート（抽象）に依存し、具象実装には依存しない
pub struct GenerateDevContainerUseCase {
    template_repo: Arc<dyn TemplateRepository>,
    file_repo: Arc<dyn FileRepository>,
    user_interaction: Arc<dyn UserInteraction>,
    progress_reporter: Arc<dyn ProgressReporter>,
}

impl GenerateDevContainerUseCase {
    pub fn new(
        template_repo: Arc<dyn TemplateRepository>,
        file_repo: Arc<dyn FileRepository>,
        user_interaction: Arc<dyn UserInteraction>,
        progress_reporter: Arc<dyn ProgressReporter>,
    ) -> Self {
        Self {
            template_repo,
            file_repo,
            user_interaction,
            progress_reporter,
        }
    }

    /// メインの実行処理
    pub async fn execute(&self, request: GenerateDevContainerRequest) -> Result<GenerateDevContainerResponse> {
        // Step 1: 設定の構築
        let config_builder = ConfigBuilder::new(request.dir)
            .with_name(request.name)
            .with_base_image(request.base_image)
            .with_force(request.force);

        // Step 2: 必要に応じてベースイメージを対話的に選択
        let base_image = if config_builder.needs_base_image() {
            self.select_base_image_interactively().await?
        } else {
            config_builder.get_base_image().unwrap().clone()
        };

        let config = config_builder.build(base_image);

        // Step 3: 既存ファイルの上書き確認
        let should_continue = self.file_repo.confirm_overwrite_if_needed(&config)?;
        if !should_continue {
            return Ok(GenerateDevContainerResponse {
                success: false,
                generated_files: Vec::new(),
                message: "ユーザーが処理をキャンセルしました".to_string(),
            });
        }

        // Step 4: 設定ファイルの生成
        let generated_files = DevContainerGenerator::generate_all_files(&config);

        // Step 5: ファイルの書き込み
        let written_files = self.file_repo.write_files(&config.dir, generated_files)?;

        // Step 6: 進捗報告
        for filename in &written_files {
            self.progress_reporter.report_file_generated(filename);
        }
        self.progress_reporter.report_completion();

        Ok(GenerateDevContainerResponse {
            success: true,
            generated_files: written_files,
            message: "Dev Container設定ファイルの生成が完了しました".to_string(),
        })
    }

    /// ベースイメージを対話的に選択する
    async fn select_base_image_interactively(&self) -> Result<String> {
        self.user_interaction.show_progress("テンプレート一覧を取得中...");
        
        let templates = self.template_repo.fetch_templates().await?;
        
        if templates.is_empty() {
            return Err(anyhow::anyhow!("利用可能なテンプレートが見つかりませんでした"));
        }

        let base_image = self.user_interaction.select_base_image(templates).await?;
        
        self.user_interaction.show_success(&format!("選択されたベースイメージ: {}", base_image));
        
        Ok(base_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::domain::{
        models::Template,
        ports::{
            template_repository::mock::MockTemplateRepository,
            file_repository::mock::MockFileRepository,
            user_interaction::mock::{MockUserInteraction, MockProgressReporter},
        },
    };

    #[tokio::test]
    async fn test_execute_with_base_image() {
        // Arrange
        let template_repo: Arc<dyn TemplateRepository> = Arc::new(MockTemplateRepository::new());
        let file_repo: Arc<dyn FileRepository> = Arc::new(MockFileRepository::new());
        let user_interaction: Arc<dyn UserInteraction> = Arc::new(MockUserInteraction::new());
        let progress_reporter: Arc<dyn ProgressReporter> = Arc::new(MockProgressReporter::new());

        let use_case = GenerateDevContainerUseCase::new(
            template_repo,
            file_repo,
            user_interaction,
            progress_reporter,
        );

        let request = GenerateDevContainerRequest {
            dir: PathBuf::from("test-dir"),
            name: Some("test-container".to_string()),
            base_image: Some("ubuntu:latest".to_string()),
            force: true,
        };

        // Act
        let response = use_case.execute(request).await.unwrap();

        // Assert
        assert!(response.success);
        assert_eq!(response.generated_files.len(), 3);
        assert!(response.generated_files.contains(&"devcontainer.json".to_string()));
        assert!(response.generated_files.contains(&"compose.yaml".to_string()));
        assert!(response.generated_files.contains(&"Dockerfile".to_string()));
    }

    #[tokio::test]
    async fn test_execute_with_template_selection() {
        // Arrange
        let templates = vec![
            Template::new("ubuntu".to_string(), "src/ubuntu".to_string()),
            Template::new("node".to_string(), "src/node".to_string()),
        ];

        let template_repo: Arc<dyn TemplateRepository> = Arc::new(
            MockTemplateRepository::new()
                .with_templates(templates)
        );
        let file_repo: Arc<dyn FileRepository> = Arc::new(MockFileRepository::new());
        let user_interaction: Arc<dyn UserInteraction> = Arc::new(
            MockUserInteraction::new()
                .with_base_image_selection("mcr.microsoft.com/devcontainers/base:ubuntu-22.04".to_string())
        );
        let progress_reporter: Arc<dyn ProgressReporter> = Arc::new(MockProgressReporter::new());

        let use_case = GenerateDevContainerUseCase::new(
            template_repo,
            file_repo,
            user_interaction,
            progress_reporter,
        );

        let request = GenerateDevContainerRequest {
            dir: PathBuf::from("test-dir"),
            name: Some("test-container".to_string()),
            base_image: None, // テンプレート選択が必要
            force: true,
        };

        // Act
        let response = use_case.execute(request).await.unwrap();

        // Assert
        assert!(response.success);
        assert_eq!(response.generated_files.len(), 3);
    }

    #[tokio::test]
    async fn test_execute_cancelled_by_user() {
        // Arrange
        let template_repo: Arc<dyn TemplateRepository> = Arc::new(MockTemplateRepository::new());
        
        // 既存ファイルがある状況を作る
        let existing_files = vec![
            "test-dir/devcontainer.json".to_string(),
        ];
        
        let file_repo: Arc<dyn FileRepository> = Arc::new(
            MockFileRepository::new()
                .with_existing_files(existing_files)
                .with_overwrite_confirmation(false) // ユーザーがキャンセル
        );
        let user_interaction: Arc<dyn UserInteraction> = Arc::new(MockUserInteraction::new());
        let progress_reporter: Arc<dyn ProgressReporter> = Arc::new(MockProgressReporter::new());

        let use_case = GenerateDevContainerUseCase::new(
            template_repo,
            file_repo,
            user_interaction,
            progress_reporter,
        );

        let request = GenerateDevContainerRequest {
            dir: PathBuf::from("test-dir"),
            name: Some("test-container".to_string()),
            base_image: Some("ubuntu:latest".to_string()),
            force: false, // 確認が必要
        };

        // Act
        let response = use_case.execute(request).await.unwrap();

        // Assert
        assert!(!response.success);
        assert_eq!(response.generated_files.len(), 0);
        assert!(response.message.contains("キャンセル"));
    }
} 