use async_trait::async_trait;
use anyhow::Result;
use crate::domain::models::{DevContainerTemplate, ImageConfiguration};

/// Dev Containerテンプレート取得のための抽象リポジトリ（ポート）
/// DIPによりDomain層からInfrastructure層への依存を逆転させる
#[async_trait]
pub trait DevContainerTemplateRepository: Send + Sync {
    /// 利用可能なDev Containerテンプレート一覧を取得する
    async fn fetch_templates(&self) -> Result<Vec<DevContainerTemplate>>;
    
    /// 指定されたテンプレートのイメージ設定情報を取得する
    async fn fetch_template_info(&self, template_name: &str) -> Result<ImageConfiguration>;
}

// DevContainerTemplateRepositoryの実装をテストで使用するためのモックトレイト
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;

    pub struct MockDevContainerTemplateRepository {
        templates: Vec<DevContainerTemplate>,
        template_infos: HashMap<String, ImageConfiguration>,
    }

    impl MockDevContainerTemplateRepository {
        pub fn new() -> Self {
            Self {
                templates: Vec::new(),
                template_infos: HashMap::new(),
            }
        }

        pub fn with_templates(mut self, templates: Vec<DevContainerTemplate>) -> Self {
            self.templates = templates;
            self
        }

        pub fn with_template_info(mut self, name: String, info: ImageConfiguration) -> Self {
            self.template_infos.insert(name, info);
            self
        }
    }

    #[async_trait]
    impl DevContainerTemplateRepository for MockDevContainerTemplateRepository {
        async fn fetch_templates(&self) -> Result<Vec<DevContainerTemplate>> {
            Ok(self.templates.clone())
        }

        async fn fetch_template_info(&self, template_name: &str) -> Result<ImageConfiguration> {
            self.template_infos
                .get(template_name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_name))
        }
    }
} 