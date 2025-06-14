use async_trait::async_trait;
use anyhow::Result;
use crate::domain::models::Template;

/// ユーザーインタラクションのための抽象サービス（ポート）
/// DIPによりDomain層からInfrastructure層への依存を逆転させる
#[async_trait]
pub trait UserInteraction: Send + Sync {
    /// テンプレート一覧からベースイメージを対話的に選択する
    async fn select_base_image(&self, templates: Vec<Template>) -> Result<String>;
    
    /// ユーザーに確認を求める
    fn confirm(&self, message: &str) -> Result<bool>;
    
    /// 進捗情報を表示する
    fn show_progress(&self, message: &str);
    
    /// 成功メッセージを表示する
    fn show_success(&self, message: &str);
    
    /// エラーメッセージを表示する
    fn show_error(&self, message: &str);
    
    /// 警告メッセージを表示する
    fn show_warning(&self, message: &str);
    
    /// 情報メッセージを表示する
    fn show_info(&self, message: &str);
}

/// プログレス報告のための抽象サービス（ポート）
pub trait ProgressReporter: Send + Sync {
    /// ファイル生成完了を報告
    fn report_file_generated(&self, filename: &str);
    
    /// 全体の完了を報告
    fn report_completion(&self);
    
    /// エラーを報告
    fn report_error(&self, message: &str);
    
    /// 情報メッセージを報告
    fn report_info(&self, message: &str);
    
    /// 警告メッセージを報告
    fn report_warning(&self, message: &str);
}

// UserInteractionの実装をテストで使用するためのモック
#[cfg(test)]
pub mod mock {
    use super::*;

    pub struct MockUserInteraction {
        selected_base_image: Option<String>,
        confirmation_response: bool,
        messages: Vec<String>,
    }

    impl MockUserInteraction {
        pub fn new() -> Self {
            Self {
                selected_base_image: None,
                confirmation_response: true,
                messages: Vec::new(),
            }
        }

        pub fn with_base_image_selection(mut self, base_image: String) -> Self {
            self.selected_base_image = Some(base_image);
            self
        }

        pub fn with_confirmation_response(mut self, response: bool) -> Self {
            self.confirmation_response = response;
            self
        }

        pub fn get_messages(&self) -> &Vec<String> {
            &self.messages
        }
    }

    #[async_trait]
    impl UserInteraction for MockUserInteraction {
        async fn select_base_image(&self, templates: Vec<Template>) -> Result<String> {
            if let Some(ref base_image) = self.selected_base_image {
                Ok(base_image.clone())
            } else if !templates.is_empty() {
                Ok(format!("ubuntu:latest")) // デフォルト値
            } else {
                Err(anyhow::anyhow!("No templates available"))
            }
        }

        fn confirm(&self, _message: &str) -> Result<bool> {
            Ok(self.confirmation_response)
        }

        fn show_progress(&self, _message: &str) {}
        fn show_success(&self, _message: &str) {}
        fn show_error(&self, _message: &str) {}
        fn show_warning(&self, _message: &str) {}
        fn show_info(&self, _message: &str) {}
    }

    pub struct MockProgressReporter {
        reported_files: Vec<String>,
        completed: bool,
    }

    impl MockProgressReporter {
        pub fn new() -> Self {
            Self {
                reported_files: Vec::new(),
                completed: false,
            }
        }

        pub fn get_reported_files(&self) -> &Vec<String> {
            &self.reported_files
        }

        pub fn is_completed(&self) -> bool {
            self.completed
        }
    }

    impl ProgressReporter for MockProgressReporter {
        fn report_file_generated(&self, _filename: &str) {}
        fn report_completion(&self) {}
        fn report_error(&self, _message: &str) {}
        fn report_info(&self, _message: &str) {}
        fn report_warning(&self, _message: &str) {}
    }
} 