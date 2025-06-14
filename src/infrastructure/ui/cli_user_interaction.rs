use async_trait::async_trait;
use anyhow::Result;
use colored::*;
use inquire::Select;
use crate::domain::{
    models::Template,
    ports::{TemplateRepository, UserInteraction},
};

/// CLI環境でのユーザーインタラクション実装
pub struct CliUserInteraction {
    template_repository: Box<dyn TemplateRepository>,
}

impl CliUserInteraction {
    pub fn new(template_repository: Box<dyn TemplateRepository>) -> Self {
        Self { template_repository }
    }
}

#[async_trait]
impl UserInteraction for CliUserInteraction {
    /// テンプレート一覧からベースイメージを対話的に選択する
    async fn select_base_image(&self, templates: Vec<Template>) -> Result<String> {
        if templates.is_empty() {
            return Err(anyhow::anyhow!("テンプレートが見つかりませんでした"));
        }

        // Step 1: テンプレート選択
        let template_names: Vec<String> = templates.iter().map(|t| t.name.clone()).collect();
        let selected_template_name = Select::new("テンプレートを選択してください", template_names).prompt()?;
        
        let selected_template = templates
            .iter()
            .find(|t| t.name == selected_template_name)
            .ok_or_else(|| anyhow::anyhow!("選択されたテンプレートが見つかりませんでした"))?;

        // Step 2: テンプレート情報の取得
        self.show_progress("テンプレート情報を取得中...");
        let template_info = self.template_repository
            .fetch_template_info(&selected_template.name)
            .await?;

        // Step 3: 最終イメージの生成
        let final_image = template_info.resolve_image();
        
        self.show_success(&format!("選択されたベースイメージ: {}", final_image));
        
        Ok(final_image)
    }
    
    /// ユーザーに確認を求める
    fn confirm(&self, message: &str) -> Result<bool> {
        use inquire::Confirm;
        Ok(Confirm::new(message)
            .with_default(false)
            .prompt()?)
    }
    
    /// 進捗情報を表示する
    fn show_progress(&self, message: &str) {
        println!("{}", message.blue());
    }
    
    /// 成功メッセージを表示する
    fn show_success(&self, message: &str) {
        println!("{}", message.green());
    }
    
    /// エラーメッセージを表示する
    fn show_error(&self, message: &str) {
        println!("{}", format!("エラー: {}", message).red());
    }
    
    /// 警告メッセージを表示する
    fn show_warning(&self, message: &str) {
        println!("{}", message.yellow());
    }
    
    /// 情報メッセージを表示する
    fn show_info(&self, message: &str) {
        println!("{}", message.blue());
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
    use crate::domain::ports::template_repository::mock::MockTemplateRepository;

    #[test]
    fn test_cli_ui_creation() {
        let _ui = CliUi;
        // 作成できることを確認
    }

    #[tokio::test]
    async fn test_cli_user_interaction_empty_templates() {
        let template_repo = Box::new(MockTemplateRepository::new());
        let ui = CliUserInteraction::new(template_repo);
        
        let result = ui.select_base_image(vec![]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("テンプレートが見つかりませんでした"));
    }

    // 注：対話的な機能のテストは実際のユーザー入力を必要とするため、
    // 統合テストまたはモック化されたテストを別途実装する必要があります
} 