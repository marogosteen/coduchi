pub mod template_repository;
pub mod file_repository;
pub mod user_interaction;

// 明示的なエクスポート（名前衝突を避ける）
pub use template_repository::{DevContainerTemplateRepository};
pub use file_repository::{FileRepository};
pub use user_interaction::{UserInteraction, ProgressReporter};

// テスト用モックを明示的にエクスポート
#[cfg(test)]
pub mod test_doubles {
    pub use super::template_repository::mock as template_mock;
    pub use super::file_repository::mock as file_mock;
    pub use super::user_interaction::mock as interaction_mock;
} 