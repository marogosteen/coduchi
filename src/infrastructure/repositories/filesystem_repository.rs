use anyhow::Result;
use colored::*;
use inquire::Confirm;
use std::path::Path;
use crate::domain::{
            models::{ComposeConfig, GeneratedFile},
    ports::FileRepository,
};

/// ローカルファイルシステム操作を行うリポジトリ実装
pub struct FileSystemRepository;

impl FileSystemRepository {
    pub fn new() -> Self {
        Self
    }
}

impl FileRepository for FileSystemRepository {
    /// 生成されたファイルを設定に基づいてディスクに書き込む
    /// .devcontainer/ディレクトリが存在しない場合は自動的に作成する
    fn write_files(&self, config: &ComposeConfig, files: Vec<GeneratedFile>) -> Result<Vec<String>> {
        let output_dir = config.output_dir();
        
        // .devcontainer/ディレクトリが存在しない場合は作成
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }
        
        let mut written_files = Vec::new();
        
        for file in files {
            let file_path = output_dir.join(&file.filename);
            std::fs::write(&file_path, &file.content)?;
            written_files.push(file.filename);
        }
        
        Ok(written_files)
    }

    /// 既存ファイルの上書き確認を行う
    /// .devcontainer/ディレクトリ内のファイルをチェックする
    fn confirm_overwrite_if_needed(&self, config: &ComposeConfig) -> Result<bool> {
        if config.force {
            return Ok(true);
        }
        
        let target_files = ["devcontainer.json", "compose.yaml", "Dockerfile"];
        let output_dir = config.output_dir();
        let existing_files = self.get_existing_files(&output_dir, &target_files);
        
        if existing_files.is_empty() {
            return Ok(true);
        }
        
        let message = format!(
            "以下のファイルが既に存在します（{}内）：\n{}\n上書きしますか？",
            output_dir.display(),
            existing_files.join("\n")
        );
        
        let should_continue = Confirm::new(&message)
            .with_default(false)
            .prompt()?;
        
        if !should_continue {
            println!("{}", "処理を中止しました。".yellow());
        }
        
        Ok(should_continue)
    }

    /// ファイルが存在するかチェック
    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// 指定ディレクトリに存在する対象ファイルの一覧を取得
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

impl Default for FileSystemRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;
    use crate::domain::models::{ComposeConfig, ComposeConfigBuilder, GeneratedFile};

    #[test]
    fn test_write_files() {
        let repository = FileSystemRepository::new();
        let temp_dir = TempDir::new().unwrap();
        let config = ComposeConfig::new(
            temp_dir.path().to_path_buf(),
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            "ubuntu:latest".to_string(),
            false,
        );
        let files = vec![
            GeneratedFile::new("test.txt".to_string(), "test content".to_string()),
            GeneratedFile::new("test2.txt".to_string(), "test content 2".to_string()),
        ];

        let result = repository.write_files(&config, files);
        assert!(result.is_ok());

        let written_files = result.unwrap();
        assert_eq!(written_files.len(), 2);
        assert!(written_files.contains(&"test.txt".to_string()));
        assert!(written_files.contains(&"test2.txt".to_string()));

        // ファイルが実際に作成されているか確認（.devcontainer/ディレクトリ内）
        let output_dir = config.output_dir();
        assert!(repository.file_exists(&output_dir.join("test.txt")));
        assert!(repository.file_exists(&output_dir.join("test2.txt")));
    }

    #[test]
    fn test_confirm_overwrite_with_force() {
        let repository = FileSystemRepository::new();
        let temp_dir = TempDir::new().unwrap();
        let config = ComposeConfig::new(
            temp_dir.path().to_path_buf(),
            "test".to_string(),          // name
            "test".to_string(),          // container_name
            "test".to_string(),          // dir_name
            "test".to_string(),          // image_name
            "ubuntu:latest".to_string(), // base_image
            true,                        // force = true
        );
        
        // .devcontainer/ディレクトリにファイルを作成
        let output_dir = config.output_dir();
        std::fs::create_dir_all(&output_dir).unwrap();
        File::create(output_dir.join("devcontainer.json")).unwrap();
        
        // forceフラグがtrueの場合は確認せずにtrue
        assert!(repository.confirm_overwrite_if_needed(&config).unwrap());
    }
    
    #[test]
    fn test_confirm_overwrite_no_existing_files() {
        let repository = FileSystemRepository::new();
        let temp_dir = TempDir::new().unwrap();
        let config = ComposeConfig::new(
            temp_dir.path().to_path_buf(),
            "test".to_string(),          // name
            "test".to_string(),          // container_name
            "test".to_string(),          // dir_name
            "test".to_string(),          // image_name
            "ubuntu:latest".to_string(), // base_image
            false,                       // force = false
        );
        
        // 既存ファイルがない場合はtrue
        assert!(repository.confirm_overwrite_if_needed(&config).unwrap());
    }

    #[test]
    fn test_file_exists() {
        let repository = FileSystemRepository::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        assert!(!repository.file_exists(&file_path));
        
        File::create(&file_path).unwrap();
        assert!(repository.file_exists(&file_path));
    }

    #[test]
    fn test_get_existing_files() {
        let repository = FileSystemRepository::new();
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join(".devcontainer");
        std::fs::create_dir_all(&output_dir).unwrap();
        
        // いくつかのファイルを作成
        File::create(output_dir.join("devcontainer.json")).unwrap();
        File::create(output_dir.join("compose.yaml")).unwrap();
        // Dockerfileは作成しない
        
        let target_files = ["devcontainer.json", "compose.yaml", "Dockerfile"];
        let existing = repository.get_existing_files(&output_dir, &target_files);
        
        assert_eq!(existing.len(), 2);
        assert!(existing.contains(&"devcontainer.json".to_string()));
        assert!(existing.contains(&"compose.yaml".to_string()));
        assert!(!existing.contains(&"Dockerfile".to_string()));
    }

    #[test]
    fn test_confirm_overwrite_if_needed_with_no_conflict() {
        let _repository = FileSystemRepository::new();
        let temp_dir = TempDir::new().unwrap();
        let _config = ComposeConfigBuilder::new(temp_dir.path().to_path_buf())
            .with_force(false)
            .build("ubuntu:latest".to_string());
        
        // ... existing code ...
    }
} 