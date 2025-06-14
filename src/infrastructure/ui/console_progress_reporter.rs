use colored::*;
use crate::domain::ports::ProgressReporter;

/// コンソール出力でプログレス表示を行うレポーター実装
pub struct ConsoleProgressReporter;

impl ConsoleProgressReporter {
    pub fn new() -> Self {
        Self
    }
}

impl ProgressReporter for ConsoleProgressReporter {
    /// ファイル生成完了を報告
    fn report_file_generated(&self, filename: &str) {
        println!("{}", format!("{}を生成しました。", filename).green());
    }

    /// 全体の完了を報告
    fn report_completion(&self) {
        println!("{}", "設定ファイルの生成が完了しました。".green());
    }

    /// エラーを報告
    fn report_error(&self, message: &str) {
        println!("{}", format!("エラー: {}", message).red());
    }

    /// 情報メッセージを報告
    fn report_info(&self, message: &str) {
        println!("{}", message.blue());
    }

    /// 警告メッセージを報告
    fn report_warning(&self, message: &str) {
        println!("{}", message.yellow());
    }
}

impl Default for ConsoleProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_reporter_creation() {
        let reporter = ConsoleProgressReporter::new();
        // メソッドが実際に呼べることを確認
        reporter.report_info("Test message");
    }

    #[test]
    fn test_progress_reporter_default() {
        let reporter = ConsoleProgressReporter::default();
        reporter.report_info("Test default");
    }

    #[test]
    fn test_all_report_methods() {
        let reporter = ConsoleProgressReporter::new();
        
        // すべてのメソッドが呼べることを確認
        reporter.report_file_generated("test.txt");
        reporter.report_completion();
        reporter.report_error("Test error");
        reporter.report_info("Test info");
        reporter.report_warning("Test warning");
        
        // パニックしないことを確認
    }
} 