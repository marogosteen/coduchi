use serde::{Deserialize, Serialize};

/// テンプレート情報を表すドメインモデル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub name: String,
    pub path: String,
}

impl Template {
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
    }
}

/// テンプレートの詳細情報を表すドメインモデル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateInfo {
    pub image: String,
    pub image_variant: String,
}

impl TemplateInfo {
    pub fn new(image: String, image_variant: String) -> Self {
        Self {
            image,
            image_variant,
        }
    }

    /// テンプレート情報から最終的なDockerイメージ文字列を生成
    pub fn resolve_image(&self) -> String {
        self.image.replace("${VARIANT}", &self.image_variant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = Template::new("ubuntu".to_string(), "src/ubuntu".to_string());
        assert_eq!(template.name, "ubuntu");
        assert_eq!(template.path, "src/ubuntu");
    }

    #[test]
    fn test_template_info_resolve_image() {
        let template_info = TemplateInfo::new(
            "mcr.microsoft.com/devcontainers/base:${VARIANT}".to_string(),
            "ubuntu-22.04".to_string(),
        );
        
        let resolved = template_info.resolve_image();
        assert_eq!(resolved, "mcr.microsoft.com/devcontainers/base:ubuntu-22.04");
    }

    #[test]
    fn test_template_info_no_variant() {
        let template_info = TemplateInfo::new(
            "ubuntu:latest".to_string(),
            "latest".to_string(),
        );
        
        let resolved = template_info.resolve_image();
        assert_eq!(resolved, "ubuntu:latest");
    }
} 