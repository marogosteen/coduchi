use async_trait::async_trait;
use anyhow::Result;
use crate::domain::models::{Template, TemplateInfo};

/// テンプレート取得のための抽象リポジトリ（ポート）
/// DIPによりDomain層からInfrastructure層への依存を逆転させる
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    /// 利用可能なテンプレート一覧を取得する
    async fn fetch_templates(&self) -> Result<Vec<Template>>;
    
    /// 指定されたテンプレートの詳細情報を取得する
    async fn fetch_template_info(&self, template_name: &str) -> Result<TemplateInfo>;
}

// TemplateRepositoryの実装をテストで使用するためのモックトレイト
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;

    pub struct MockTemplateRepository {
        templates: Vec<Template>,
        template_infos: HashMap<String, TemplateInfo>,
    }

    impl MockTemplateRepository {
        pub fn new() -> Self {
            Self {
                templates: Vec::new(),
                template_infos: HashMap::new(),
            }
        }

        pub fn with_templates(mut self, templates: Vec<Template>) -> Self {
            self.templates = templates;
            self
        }

        pub fn with_template_info(mut self, name: String, info: TemplateInfo) -> Self {
            self.template_infos.insert(name, info);
            self
        }
    }

    #[async_trait]
    impl TemplateRepository for MockTemplateRepository {
        async fn fetch_templates(&self) -> Result<Vec<Template>> {
            Ok(self.templates.clone())
        }

        async fn fetch_template_info(&self, template_name: &str) -> Result<TemplateInfo> {
            self.template_infos
                .get(template_name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_name))
        }
    }
} 