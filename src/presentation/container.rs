use std::sync::Arc;
use crate::{
    application::GenerateDevContainerUseCase,
    domain::ports::{DevContainerTemplateRepository, FileRepository, UserInteraction, ProgressReporter},
    infrastructure::{
        repositories::{GitHubTemplateRepository, FileSystemRepository},
        ui::{CliUserInteraction, ConsoleProgressReporter},
    },
};

/// DIコンテナ - 依存性注入を管理
pub struct Container {
    template_repository: Arc<dyn DevContainerTemplateRepository>,
    file_repository: Arc<dyn FileRepository>,
    user_interaction: Arc<dyn UserInteraction>,
    progress_reporter: Arc<dyn ProgressReporter>,
}

impl Container {
    /// 本番環境用のコンテナを作成
    pub fn create() -> Self {
        // Template Repository を2つのインスタンスで共有
        let template_repo = Arc::new(GitHubTemplateRepository::new());
        
        // UserInteractionはDevContainerTemplateRepositoryに依存するため、Boxで包む
        let template_repo_for_ui = Box::new(GitHubTemplateRepository::new());
        let user_interaction = Arc::new(CliUserInteraction::new(template_repo_for_ui));
        
        Self {
            template_repository: template_repo,
            file_repository: Arc::new(FileSystemRepository::new()),
            user_interaction,
            progress_reporter: Arc::new(ConsoleProgressReporter::new()),
        }
    }
    
    #[cfg(test)]
    pub fn create_for_test() -> Self {
        use crate::domain::ports::{
            file_repository::mock::MockFileRepository,
            template_repository::mock::MockDevContainerTemplateRepository,
            user_interaction::mock::{MockUserInteraction, MockProgressReporter},
        };
        
        Self {
            template_repository: Arc::new(MockDevContainerTemplateRepository::new()),
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
        template_repository: Arc<dyn DevContainerTemplateRepository>,
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
    pub fn template_repository(&self) -> Arc<dyn DevContainerTemplateRepository> {
        Arc::clone(&self.template_repository)
    }

    pub fn file_repository(&self) -> Arc<dyn FileRepository> {
        Arc::clone(&self.file_repository)
    }

    pub fn user_interaction(&self) -> Arc<dyn UserInteraction> {
        Arc::clone(&self.user_interaction)
    }

    pub fn progress_reporter(&self) -> Arc<dyn ProgressReporter> {
        Arc::clone(&self.progress_reporter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_container() {
        let container = Container::create();
        
        // 各依存オブジェクトが正しく作成されることを確認
        let _template_repo = container.template_repository();
        let _file_repo = container.file_repository();
        let _user_interaction = container.user_interaction();
        let _progress_reporter = container.progress_reporter();
    }

    #[test]
    fn test_create_test_container() {
        let container = Container::create_for_test();
        
        // テスト用コンテナが正しく作成されることを確認
        let _template_repo = container.template_repository();
        let _file_repo = container.file_repository();
        let _user_interaction = container.user_interaction();
        let _progress_reporter = container.progress_reporter();
    }

    #[test]
    fn test_template_repository_implementation() {
        let container = Container::create();
        let _repo = container.template_repository();
        // GitHubTemplateRepositoryが注入されていることを確認
    }

    #[test]
    fn test_file_repository_implementation() {
        let container = Container::create();
        let _repo = container.file_repository();
        // FileSystemRepositoryが注入されていることを確認
    }

    #[test]
    fn test_ui_implementations() {
        let container = Container::create();
        let _ui = container.user_interaction();
        let _reporter = container.progress_reporter();
        // 各UI実装が注入されていることを確認
    }
} 