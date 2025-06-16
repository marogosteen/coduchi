use anyhow::Result;
use std::path::Path;
use crate::domain::models::{ComposeConfig, GeneratedFile};

/// ファイル操作のための抽象リポジトリ（ポート）
/// DIPによりDomain層からInfrastructure層への依存を逆転させる
pub trait FileRepository: Send + Sync {
    /// 生成されたファイルを設定に基づいてディスクに書き込む
    /// 設定内の出力ディレクトリ（.devcontainer/）に自動でファイルを配置する
    fn write_files(&self, config: &ComposeConfig, files: Vec<GeneratedFile>) -> Result<Vec<String>>;
    
    /// 既存ファイルの上書き確認を行う
    fn confirm_overwrite_if_needed(&self, config: &ComposeConfig) -> Result<bool>;
    
    /// ファイルが存在するかチェック
    fn file_exists(&self, path: &Path) -> bool;
    
    /// 指定ディレクトリに存在する対象ファイルの一覧を取得
    fn get_existing_files(&self, dir: &Path, target_files: &[&str]) -> Vec<String>;
}

// FileRepositoryの実装をテストで使用するためのモック
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashSet;

    pub struct MockFileRepository {
        existing_files: HashSet<String>,
        should_confirm_overwrite: bool,
        write_should_fail: bool,
    }

    impl MockFileRepository {
        pub fn new() -> Self {
            Self {
                existing_files: HashSet::new(),
                should_confirm_overwrite: true,
                write_should_fail: false,
            }
        }

        pub fn with_existing_files(mut self, files: Vec<String>) -> Self {
            self.existing_files = files.into_iter().collect();
            self
        }

        pub fn with_overwrite_confirmation(mut self, should_confirm: bool) -> Self {
            self.should_confirm_overwrite = should_confirm;
            self
        }

        pub fn with_write_failure(mut self, should_fail: bool) -> Self {
            self.write_should_fail = should_fail;
            self
        }
    }

    impl FileRepository for MockFileRepository {
        fn write_files(&self, _config: &ComposeConfig, files: Vec<GeneratedFile>) -> Result<Vec<String>> {
            if self.write_should_fail {
                return Err(anyhow::anyhow!("Mock write failure"));
            }
            Ok(files.into_iter().map(|f| f.filename).collect())
        }

        fn confirm_overwrite_if_needed(&self, config: &ComposeConfig) -> Result<bool> {
            if config.force {
                return Ok(true);
            }

            let target_files = ["devcontainer.json", "compose.yaml", "Dockerfile"];
            let output_dir = config.output_dir();
            let has_existing = target_files
                .iter()
                .any(|&file| {
                    let full_path = output_dir.join(file).to_string_lossy().to_string();
                    self.existing_files.contains(&full_path)
                });

            if has_existing {
                Ok(self.should_confirm_overwrite)
            } else {
                Ok(true)
            }
        }

        fn file_exists(&self, path: &Path) -> bool {
            self.existing_files.contains(&path.to_string_lossy().to_string())
        }

        fn get_existing_files(&self, dir: &Path, target_files: &[&str]) -> Vec<String> {
            target_files
                .iter()
                .filter_map(|&file| {
                    let full_path = dir.join(file);
                    if self.file_exists(&full_path) {
                        Some(file.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        }
    }
} 