pub mod template_repository;
pub mod file_repository;
pub mod user_interaction;

// 明示的なエクスポート（名前衝突を避ける）
pub use template_repository::{TemplateRepository};
pub use file_repository::{FileRepository};
pub use user_interaction::{UserInteraction, ProgressReporter};

// テスト用モックを明示的にエクスポート
#[cfg(test)]
pub use template_repository::mock as template_mock;
#[cfg(test)]
pub use file_repository::mock as file_mock;
#[cfg(test)]
pub use user_interaction::mock as ui_mock; 