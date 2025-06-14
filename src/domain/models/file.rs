/// 生成されたファイルを表すドメインモデル
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub filename: String,
    pub content: String,
}

impl GeneratedFile {
    pub fn new(filename: String, content: String) -> Self {
        Self { filename, content }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn size(&self) -> usize {
        self.content.len()
    }
}

/// ファイル生成の結果を表すドメインモデル
#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub files: Vec<GeneratedFile>,
    pub success: bool,
    pub message: String,
}

impl GenerationResult {
    pub fn success(files: Vec<GeneratedFile>) -> Self {
        Self {
            files,
            success: true,
            message: "ファイル生成が成功しました".to_string(),
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            files: Vec::new(),
            success: false,
            message,
        }
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_file() {
        let file = GeneratedFile::new(
            "test.txt".to_string(),
            "test content".to_string(),
        );

        assert_eq!(file.filename, "test.txt");
        assert_eq!(file.content, "test content");
        assert!(!file.is_empty());
        assert_eq!(file.size(), 12);
    }

    #[test]
    fn test_generated_file_empty() {
        let file = GeneratedFile::new(
            "empty.txt".to_string(),
            "".to_string(),
        );

        assert!(file.is_empty());
        assert_eq!(file.size(), 0);
    }

    #[test]
    fn test_generation_result_success() {
        let files = vec![
            GeneratedFile::new("file1.txt".to_string(), "content1".to_string()),
            GeneratedFile::new("file2.txt".to_string(), "content2".to_string()),
        ];

        let result = GenerationResult::success(files);
        assert!(result.success);
        assert_eq!(result.file_count(), 2);
    }

    #[test]
    fn test_generation_result_failure() {
        let result = GenerationResult::failure("Error occurred".to_string());
        assert!(!result.success);
        assert_eq!(result.file_count(), 0);
        assert_eq!(result.message, "Error occurred");
    }
} 