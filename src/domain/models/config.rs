use std::path::PathBuf;

/// アプリケーション設定を表すドメインモデル
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub dir: PathBuf,
    pub container_name: String,
    pub dir_name: String,
    pub base_image: String,
    pub force: bool,
}

impl AppConfig {
    pub fn new(
        dir: PathBuf,
        container_name: String,
        dir_name: String,
        base_image: String,
        force: bool,
    ) -> Self {
        Self {
            dir,
            container_name,
            dir_name,
            base_image,
            force,
        }
    }
}

/// Dev Container設定のドメインモデル
#[derive(Debug, Clone)]
pub struct DevContainerConfig {
    pub name: String,
    pub workspace_folder: String,
    pub service: String,
}

impl DevContainerConfig {
    pub fn new(name: String, dir_name: String) -> Self {
        Self {
            name,
            workspace_folder: format!("/workspaces/{}", dir_name),
            service: "app".to_string(),
        }
    }
}

/// 設定構築のためのビルダー（ドメインサービス的な役割）
pub struct ConfigBuilder {
    dir: PathBuf,
    name: Option<String>,
    base_image: Option<String>,
    force: bool,
}

impl ConfigBuilder {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            name: None,
            base_image: None,
            force: false,
        }
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_base_image(mut self, base_image: Option<String>) -> Self {
        self.base_image = base_image;
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    pub fn build(self, base_image: String) -> AppConfig {
        let dir_name = self.dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        let container_name = self.name.unwrap_or_else(|| dir_name.clone());

        AppConfig::new(
            self.dir,
            container_name,
            dir_name,
            base_image,
            self.force,
        )
    }

    pub fn needs_base_image(&self) -> bool {
        self.base_image.is_none()
    }

    pub fn get_base_image(&self) -> Option<&String> {
        self.base_image.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new(PathBuf::from("test-dir"))
            .with_name(Some("test-container".to_string()))
            .with_force(true)
            .build("ubuntu:latest".to_string());

        assert_eq!(config.dir, PathBuf::from("test-dir"));
        assert_eq!(config.container_name, "test-container");
        assert_eq!(config.dir_name, "test-dir");
        assert_eq!(config.base_image, "ubuntu:latest");
        assert!(config.force);
    }

    #[test]
    fn test_devcontainer_config() {
        let config = DevContainerConfig::new("test-app".to_string(), "test-dir".to_string());
        
        assert_eq!(config.name, "test-app");
        assert_eq!(config.workspace_folder, "/workspaces/test-dir");
        assert_eq!(config.service, "app");
    }
} 