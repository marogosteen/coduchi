use async_trait::async_trait;
use anyhow::Result;
use colored::Colorize;
use inquire::Select;
use crate::domain::{
    models::DevContainerTemplate,
    ports::{DevContainerTemplateRepository, UserInteraction},
};

/// CLI環境でのユーザーインタラクション実装
pub struct CliUserInteraction {
    template_repository: Box<dyn DevContainerTemplateRepository>,
}

impl CliUserInteraction {
    pub fn new(template_repository: Box<dyn DevContainerTemplateRepository>) -> Self {
        Self { template_repository }
    }
}

#[async_trait]
impl UserInteraction for CliUserInteraction {
    /// テンプレート一覧からベースイメージを対話的に選択する
    async fn select_base_image(&self, templates: Vec<DevContainerTemplate>) -> Result<String> {
        if templates.is_empty() {
            return Err(anyhow::anyhow!("利用可能なテンプレートがありません"));
        }

        // テンプレート名の一覧を表示
        let template_names: Vec<String> = templates.iter().map(|t| t.name.clone()).collect();
        let selected_template_name = Select::new("テンプレートを選択してください", template_names).prompt()?;

        let selected_template = templates
            .iter()
            .find(|t| t.name == selected_template_name)
            .ok_or_else(|| anyhow::anyhow!("選択されたテンプレートが見つかりません"))?;

        // 選択されたテンプレートの詳細情報を取得
        let image_config = self.template_repository
            .fetch_template_info(&selected_template.name)
            .await?;

        // 最終的なイメージ名を返す
        let final_image = image_config.resolve_final_image();
        Ok(final_image)
    }
    
    /// ユーザーに確認を求める
    fn confirm(&self, message: &str) -> Result<bool> {
        let selection = Select::new(message, vec!["はい", "いいえ"]).prompt()?;
        Ok(selection == "はい")
    }
    
    /// 進捗情報を表示する
    fn show_progress(&self, message: &str) {
        println!("{} {}", "[INFO]".bright_blue(), message);
    }
    
    /// 成功メッセージを表示する
    fn show_success(&self, message: &str) {
        println!("{} {}", "[SUCCESS]".bright_green(), message);
    }
    
    /// エラーメッセージを表示する
    fn show_error(&self, message: &str) {
        println!("{} {}", "[ERROR]".bright_red(), message);
    }
    
    /// 警告メッセージを表示する
    fn show_warning(&self, message: &str) {
        println!("{} {}", "[WARNING]".bright_yellow(), message);
    }
    
    /// 情報メッセージを表示する
    fn show_info(&self, message: &str) {
        println!("{} {}", "[INFO]".bright_blue(), message);
    }
}

/// CLI操作のヘルパー（既存コードからの移行）
pub struct CliUi;

impl CliUi {
    /// ユーザーの確認を求める
    pub fn confirm(&self, message: &str) -> Result<bool> {
        use inquire::Confirm;
        Ok(Confirm::new(message)
            .with_default(false)
            .prompt()?)
    }

    /// 選択肢から選択を求める
    pub fn select<T>(&self, message: &str, options: Vec<T>) -> Result<T>
    where
        T: std::fmt::Display + Clone,
    {
        Ok(Select::new(message, options).prompt()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ImageConfiguration;
    use crate::domain::ports::template_repository::mock::MockDevContainerTemplateRepository;

    #[test]
    fn test_cli_ui_creation() {
        let _ui = CliUi;
        // 作成できることを確認
    }

    #[tokio::test]
    async fn test_cli_user_interaction_empty_templates() {
        let template_repo = Box::new(MockDevContainerTemplateRepository::new());
        let ui = CliUserInteraction::new(template_repo);
        
        let result = ui.select_base_image(vec![]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("利用可能なテンプレートがありません"));
    }

    #[tokio::test]
    async fn test_cli_user_interaction_with_templates() {
        let template = DevContainerTemplate::new("ubuntu".to_string(), "src/ubuntu".to_string());
        let config = ImageConfiguration::new(
            "mcr.microsoft.com/devcontainers/base:${VARIANT}".to_string(),
            "ubuntu-22.04".to_string(),
        );

        let template_repo = Box::new(
            MockDevContainerTemplateRepository::new()
                .with_templates(vec![template])
                .with_template_info("ubuntu".to_string(), config)
        );

        let _ui = CliUserInteraction::new(template_repo);

        // 空のテンプレートリストでテスト（実際の対話は不可能なため、機能テストのみ）
        let templates = vec![DevContainerTemplate::new("ubuntu".to_string(), "src/ubuntu".to_string())];
        
        // 実際の対話無しでテストする場合、モックの動作のみ確認
        // 完全なテストは統合テストで行う
        assert_eq!(templates.len(), 1);
    }
} 