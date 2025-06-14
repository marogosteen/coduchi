use crate::domain::models::{GeneratedFile, AppConfig, DevContainerConfig};

/// Dev Container設定ファイルのジェネレーター（ドメインサービス）
/// 外部依存を持たない純粋なビジネスロジック
pub struct DevContainerGenerator;

impl DevContainerGenerator {
    /// devcontainer.jsonの内容を生成する
    pub fn generate_devcontainer_json(config: &DevContainerConfig) -> GeneratedFile {
        let content = format!(
            r#"{{
  "name": "{}",
  "dockerComposeFile": "compose.yaml",
  "workspaceFolder": "{}",
  "service": "{}",
  "customizations": {{
    "vscode": {{
      "extensions": []
    }}
  }}
}}"#,
            config.name, config.workspace_folder, config.service
        );

        GeneratedFile::new("devcontainer.json".to_string(), content)
    }

    /// compose.yamlの内容を生成する
    pub fn generate_compose_yaml(app_config: &AppConfig) -> GeneratedFile {
        let content = format!(
            r#"services:
  app:
    image: {}:latest
    container_name: {}
    build:
      context: .
      dockerfile: Dockerfile
    volumes:
      - ..:/workspaces/{}:cached
    command: sleep infinity
"#,
            app_config.container_name, 
            app_config.container_name, 
            app_config.dir_name
        );

        GeneratedFile::new("compose.yaml".to_string(), content)
    }

    /// Dockerfileの内容を生成する
    pub fn generate_dockerfile(base_image: &str) -> GeneratedFile {
        let content = format!("FROM {}", base_image);
        GeneratedFile::new("Dockerfile".to_string(), content)
    }

    /// 全ての設定ファイルを一括生成する
    pub fn generate_all_files(app_config: &AppConfig) -> Vec<GeneratedFile> {
        let devcontainer_config = DevContainerConfig::new(
            app_config.container_name.clone(),
            app_config.dir_name.clone(),
        );

        vec![
            Self::generate_devcontainer_json(&devcontainer_config),
            Self::generate_compose_yaml(app_config),
            Self::generate_dockerfile(&app_config.base_image),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_config() -> AppConfig {
        AppConfig::new(
            PathBuf::from("test-dir"),
            "test-container".to_string(),
            "test-dir".to_string(),
            "ubuntu:latest".to_string(),
            false,
        )
    }

    #[test]
    fn test_generate_devcontainer_json() {
        let devcontainer_config = DevContainerConfig::new(
            "test-container".to_string(),
            "test-dir".to_string(),
        );
        let file = DevContainerGenerator::generate_devcontainer_json(&devcontainer_config);

        assert_eq!(file.filename, "devcontainer.json");
        assert!(file.content.contains("\"name\": \"test-container\""));
        assert!(file.content.contains("\"workspaceFolder\": \"/workspaces/test-dir\""));
        assert!(file.content.contains("\"service\": \"app\""));
    }

    #[test]
    fn test_generate_compose_yaml() {
        let config = create_test_config();
        let file = DevContainerGenerator::generate_compose_yaml(&config);

        assert_eq!(file.filename, "compose.yaml");
        assert!(file.content.contains("image: test-container:latest"));
        assert!(file.content.contains("container_name: test-container"));
        assert!(file.content.contains("/workspaces/test-dir"));
    }

    #[test]
    fn test_generate_dockerfile() {
        let file = DevContainerGenerator::generate_dockerfile("ubuntu:latest");

        assert_eq!(file.filename, "Dockerfile");
        assert_eq!(file.content, "FROM ubuntu:latest");
    }

    #[test]
    fn test_generate_all_files() {
        let config = create_test_config();
        let files = DevContainerGenerator::generate_all_files(&config);

        assert_eq!(files.len(), 3);
        assert_eq!(files[0].filename, "devcontainer.json");
        assert_eq!(files[1].filename, "compose.yaml");
        assert_eq!(files[2].filename, "Dockerfile");
        assert_eq!(files[2].content, "FROM ubuntu:latest");
    }
} 