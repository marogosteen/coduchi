use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub image: String,
    pub options: TemplateOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateOptions {
    pub image_variant: ImageVariant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageVariant {
    pub default: String,
    pub proposals: Vec<String>,
}

pub async fn fetch_templates(base_url: Option<&str>) -> Result<Vec<Template>> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/repos/devcontainers/templates/contents/src",
        base_url.unwrap_or("https://api.github.com")
    );
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    let templates: Vec<Template> = response.json().await?;
    Ok(templates)
}

pub async fn fetch_template_info(template: &str, base_url: Option<&str>) -> Result<TemplateInfo> {
    let client = reqwest::Client::new();
    let base = base_url.unwrap_or("https://api.github.com");
    
    // devcontainer.jsonの取得
    let devcontainer_url = format!(
        "{}/repos/devcontainers/templates/contents/src/{}/.devcontainer/devcontainer.json",
        base, template
    );
    let devcontainer: HashMap<String, serde_json::Value> = client
        .get(&devcontainer_url)
        .header("User-Agent", "coduchi")
        .send()
        .await?
        .json()
        .await?;

    // devcontainer-template.jsonの取得
    let template_url = format!(
        "{}/repos/devcontainers/templates/contents/src/{}/devcontainer-template.json",
        base, template
    );
    let template_info: HashMap<String, serde_json::Value> = client
        .get(&template_url)
        .header("User-Agent", "coduchi")
        .send()
        .await?
        .json()
        .await?;

    let image = devcontainer.get("image")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Image not found in devcontainer.json"))?
        .to_string();

    let options = template_info.get("options")
        .and_then(|v| v.get("imageVariant"))
        .ok_or_else(|| anyhow::anyhow!("Image variant options not found"))?;

    let image_variant = ImageVariant {
        default: options.get("default")
            .and_then(|v| v.as_str())
            .unwrap_or("latest")
            .to_string(),
        proposals: options.get("proposals")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default(),
    };

    Ok(TemplateInfo {
        image,
        options: TemplateOptions { image_variant },
    })
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
                "name": "test-template",
                "type": "dir",
                "path": "test-template"
            }
        ]);

        let _m = server
            .mock("GET", "/repos/devcontainers/templates/contents/src")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let templates = fetch_templates(Some(&server.url())).await.unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "test-template");
        assert_eq!(templates[0].path, "test-template");
    }

    #[tokio::test]
    async fn test_fetch_template_info() {
        let mut server = Server::new_async().await;
        
        let devcontainer_json = json!({
            "image": "mcr.microsoft.com/devcontainers/base:ubuntu"
        });

        let template_json = json!({
            "options": {
                "imageVariant": {
                    "default": "ubuntu-22.04",
                    "proposals": [
                        "ubuntu-20.04",
                        "ubuntu-22.04"
                    ]
                }
            }
        });

        let _m1 = server
            .mock("GET", "/repos/devcontainers/templates/contents/src/test-template/.devcontainer/devcontainer.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(devcontainer_json.to_string())
            .create();

        let _m2 = server
            .mock("GET", "/repos/devcontainers/templates/contents/src/test-template/devcontainer-template.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(template_json.to_string())
            .create();

        let info = fetch_template_info("test-template", Some(&server.url())).await.unwrap();
        assert_eq!(info.image, "mcr.microsoft.com/devcontainers/base:ubuntu");
        assert_eq!(info.options.image_variant.default, "ubuntu-22.04");
        assert_eq!(info.options.image_variant.proposals, vec!["ubuntu-20.04", "ubuntu-22.04"]);
    }
}
