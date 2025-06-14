use std::sync::Arc;
use crate::{
    application::GenerateDevContainerUseCase,
    domain::ports::{TemplateRepository, FileRepository, UserInteraction, ProgressReporter},
    infrastructure::{
        GitHubTemplateRepository, FileSystemRepository, 
        CliUserInteraction, ConsoleProgressReporter,
    },
};

/// 依存性注入コンテナ
/// DIPによる依存関係の解決と実装の組み立てを行う
pub struct Container {
    template_repository: Arc<dyn TemplateRepository>,
    file_repository: Arc<dyn FileRepository>,
    user_interaction: Arc<dyn UserInteraction>,
    progress_reporter: Arc<dyn ProgressReporter>,
}

impl Container {
    /// 本番環境用のコンテナを構築
    pub fn new_production() -> Self {
        let template_repo = Arc::new(GitHubTemplateRepository::new());
        let file_repo = Arc::new(FileSystemRepository::new());
        
        // UserInteractionはTemplateRepositoryに依存するため、Boxで包む
        let template_repo_for_ui = Box::new(GitHubTemplateRepository::new());
        let user_interaction = Arc::new(CliUserInteraction::new(template_repo_for_ui));
        
        let progress_reporter = Arc::new(ConsoleProgressReporter::new());

        Self {
            template_repository: template_repo,
            file_repository: file_repo,
            user_interaction,
            progress_reporter,
        }
    }

    /// テスト環境用のコンテナを構築
    #[cfg(test)]
    pub fn new_for_testing() -> Self {
        use crate::domain::ports::{
            template_repository::mock::MockTemplateRepository,
            file_repository::mock::MockFileRepository,
            user_interaction::mock::{MockUserInteraction, MockProgressReporter},
        };

        Self {
            template_repository: Arc::new(MockTemplateRepository::new()),
            file_repository: Arc::new(MockFileRepository::new()),
            user_interaction: Arc::new(MockUserInteraction::new()),
            progress_reporter: Arc::new(MockProgressReporter::new()),
        }
    }

    /// ユースケースインスタンスを作成
    pub fn create_use_case(&self) -> GenerateDevContainerUseCase {
        GenerateDevContainerUseCase::new(
            self.template_repository.clone(),
            self.file_repository.clone(),
            self.user_interaction.clone(),
            self.progress_reporter.clone(),
        )
    }

    /// カスタム実装でのコンテナ構築（将来の拡張用）
    pub fn new_with_custom(
        template_repository: Arc<dyn TemplateRepository>,
        file_repository: Arc<dyn FileRepository>,
        user_interaction: Arc<dyn UserInteraction>,
        progress_reporter: Arc<dyn ProgressReporter>,
    ) -> Self {
        Self {
            template_repository,
            file_repository,
            user_interaction,
            progress_reporter,
        }
    }

    // Getters（必要に応じて個別にアクセス可能）
    pub fn template_repository(&self) -> Arc<dyn TemplateRepository> {
        self.template_repository.clone()
    }

    pub fn file_repository(&self) -> Arc<dyn FileRepository> {
        self.file_repository.clone()
    }

    pub fn user_interaction(&self) -> Arc<dyn UserInteraction> {
        self.user_interaction.clone()
    }

    pub fn progress_reporter(&self) -> Arc<dyn ProgressReporter> {
        self.progress_reporter.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_container() {
        let container = Container::new_production();
        let _use_case = container.create_use_case();
        // 簡単なインスタンス化テスト
    }

    #[test]
    fn test_create_test_container() {
        let container = Container::new_for_testing();
        let _use_case = container.create_use_case();
        // 簡単なインスタンス化テスト
    }

    #[test]
    fn test_template_repository_implementation() {
        let container = Container::new_production();
        // 簡単なインスタンス化テスト
        let _template_repo = container.template_repository();
    }

    #[test]
    fn test_file_repository_implementation() {
        let container = Container::new_production();
        // 簡単なインスタンス化テスト
        let _file_repo = container.file_repository();
    }

    #[test]
    fn test_ui_implementations() {
        let container = Container::new_production();
        let _use_case = container.create_use_case();
    }
} 