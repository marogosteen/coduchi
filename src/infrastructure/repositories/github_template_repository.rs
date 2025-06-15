use async_trait::async_trait;
use anyhow::Result;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Deserialize;
use crate::domain::{
    models::{DevContainerTemplate, ImageConfiguration},
    ports::DevContainerTemplateRepository,
};

/// GitHub APIのレスポンス内容を表す構造体
#[derive(Debug, Deserialize)]
struct GitHubContent {
    pub content: String,
    pub encoding: String,
}

/// GitHub API経由でDev Containerテンプレート情報を取得するリポジトリ実装
pub struct GitHubTemplateRepository {
    client: reqwest::Client,
    base_url: String,
}

impl GitHubTemplateRepository {
    /// 新しいGitHubテンプレートリポジトリを作成
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.github.com".to_string(),
        }
    }

    /// テスト用のモックURLでGitHubリポジトリを作成
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// devcontainer.jsonファイルを取得する
    async fn fetch_devcontainer_json(&self, template_name: &str) -> Result<GitHubContent> {
        let url = format!(
            "{}/repos/devcontainers/templates/contents/src/{}/.devcontainer/devcontainer.json",
            self.base_url, template_name
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "coduchi")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch template info: {}",
                response.status()
            ));
        }

        response.json().await.map_err(anyhow::Error::from)
    }

    /// Base64エンコードされたdevcontainer.jsonをパースしてイメージ設定情報を抽出する
    fn parse_template_info(&self, content: GitHubContent) -> Result<ImageConfiguration> {
        if content.encoding != "base64" {
            return Err(anyhow::anyhow!("Unexpected content encoding"));
        }

        let decoded = STANDARD.decode(content.content.replace('\n', ""))?;
        let json_str = String::from_utf8(decoded)?;

        // JSONコメント（//）を除去
        let json_str = json_str
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        let json: serde_json::Value = serde_json::from_str(&json_str)?;

        let base_image = json["image"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Image not found in devcontainer.json"))?;

        let variant = json["imageVariant"]
            .as_str()
            .unwrap_or("latest")
            .to_string();

        Ok(ImageConfiguration::new(base_image.to_string(), variant))
    }
}

#[async_trait]
impl DevContainerTemplateRepository for GitHubTemplateRepository {
    /// Dev Containerテンプレート一覧を取得する
    async fn fetch_templates(&self) -> Result<Vec<DevContainerTemplate>> {
        let url = format!("{}/repos/devcontainers/templates/contents/src", self.base_url);

        let response = self.client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "coduchi")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch templates: {}", response.status()));
        }

        let text = response.text().await?;

        #[derive(Deserialize)]
        struct GitHubDirContent {
            name: String,
            path: String,
            #[serde(rename = "type")]
            content_type: String,
        }

        let contents: Vec<GitHubDirContent> = serde_json::from_str(&text)?;
        let templates: Vec<DevContainerTemplate> = contents
            .into_iter()
            .filter(|content| content.content_type == "dir")
            .map(|content| DevContainerTemplate::new(content.name, content.path))
            .collect();

        Ok(templates)
    }

    /// 指定されたテンプレートのイメージ設定情報を取得する
    async fn fetch_template_info(&self, template_name: &str) -> Result<ImageConfiguration> {
        let content = self.fetch_devcontainer_json(template_name).await?;
        self.parse_template_info(content)
    }
}

impl Default for GitHubTemplateRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use serde_json::json;

    #[tokio::test]
    async fn test_fetch_templates() {
        let mut server = Server::new_async().await;

        let mock_response = json!([
            {
                "name": "alpine",
                "path": "src/alpine",
                "type": "dir"
            },
            {
                "name": "ubuntu",
                "path": "src/ubuntu",
                "type": "dir"
            },
            {
                "name": "README.md",
                "path": "src/README.md",
                "type": "file"
            }
        ]);

        let mock = server
            .mock("GET", "/repos/devcontainers/templates/contents/src")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let repository = GitHubTemplateRepository::with_base_url(server.url());
        let templates = repository.fetch_templates().await.unwrap();

        mock.assert_async().await;
        assert_eq!(templates.len(), 2); // ディレクトリのみ
        assert_eq!(templates[0].name, "alpine");
        assert_eq!(templates[1].name, "ubuntu");
    }

    #[tokio::test]
    async fn test_parse_template_info() {
        let repository = GitHubTemplateRepository::new();
        let mock_content = GitHubContent {
            content: base64::engine::general_purpose::STANDARD.encode(
                r#"{
  "image": "mcr.microsoft.com/devcontainers/base:${VARIANT}",
  "imageVariant": "ubuntu-22.04"
}"#
            ),
            encoding: "base64".to_string(),
        };

        let result = repository.parse_template_info(mock_content).unwrap();
        assert_eq!(result.base_image, "mcr.microsoft.com/devcontainers/base:${VARIANT}");
        assert_eq!(result.variant, "ubuntu-22.04");
        assert_eq!(result.resolve_final_image(), "mcr.microsoft.com/devcontainers/base:ubuntu-22.04");
    }

    #[tokio::test]
    async fn test_parse_template_info_with_comments() {
        let repository = GitHubTemplateRepository::new();
        let json_with_comments = r#"{
  // This is a comment
  "image": "ubuntu:${VARIANT}",
  "imageVariant": "20.04"
}"#;

        let mock_content = GitHubContent {
            content: base64::engine::general_purpose::STANDARD.encode(json_with_comments),
            encoding: "base64".to_string(),
        };

        let result = repository.parse_template_info(mock_content).unwrap();
        assert_eq!(result.base_image, "ubuntu:${VARIANT}");
        assert_eq!(result.variant, "20.04");
        assert_eq!(result.resolve_final_image(), "ubuntu:20.04");
    }
} 