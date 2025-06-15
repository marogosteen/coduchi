use serde::{Deserialize, Serialize};

/// Dev Containerテンプレートを表すドメインモデル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DevContainerTemplate {
    pub name: String,
    pub path: String,
}

impl DevContainerTemplate {
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
    }
}

/// Dockerイメージ設定を表すドメインモデル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageConfiguration {
    pub base_image: String,
    pub variant: String,
}

impl ImageConfiguration {
    pub fn new(base_image: String, variant: String) -> Self {
        Self {
            base_image,
            variant,
        }
    }

    /// バリアントを適用した最終的なDockerイメージ名を取得
    pub fn resolve_final_image(&self) -> String {
        self.base_image.replace("${VARIANT}", &self.variant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devcontainer_template_creation() {
        let template = DevContainerTemplate::new("ubuntu".to_string(), "src/ubuntu".to_string());
        assert_eq!(template.name, "ubuntu");
        assert_eq!(template.path, "src/ubuntu");
    }

    #[test]
    fn test_image_configuration_resolve_final_image() {
        let config = ImageConfiguration::new(
            "mcr.microsoft.com/devcontainers/base:${VARIANT}".to_string(),
            "ubuntu-22.04".to_string(),
        );
        
        let resolved = config.resolve_final_image();
        assert_eq!(resolved, "mcr.microsoft.com/devcontainers/base:ubuntu-22.04");
    }

    #[test]
    fn test_image_configuration_no_variant() {
        let config = ImageConfiguration::new(
            "ubuntu:latest".to_string(),
            "latest".to_string(),
        );
        
        let resolved = config.resolve_final_image();
        assert_eq!(resolved, "ubuntu:latest");
    }
} 