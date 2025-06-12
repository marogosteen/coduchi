use anyhow::Result;
use clap::Parser;
use colored::*;
use inquire::{Select, Confirm};
use std::path::PathBuf;

use coduchi::cli::Cli;
use coduchi::generator::{generate_devcontainer_json, generate_compose_yaml, generate_dockerfile};
use coduchi::template::{fetch_templates, fetch_template_info};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // ディレクトリ名とコンテナ名を取得
    let dir_name = cli.get_dir_name();
    let container_name = cli.get_container_name();

    // 既存ファイルの確認
    if !cli.force {
        let files = ["devcontainer.json", "compose.yaml", "Dockerfile"];
        let existing_files: Vec<_> = files
            .iter()
            .filter(|&&file| cli.dir.join(file).exists())
            .map(|&file| file)
            .collect();

        if !existing_files.is_empty() {
            let message = format!(
                "以下のファイルが既に存在します：\n{}\n上書きしますか？",
                existing_files.join("\n")
            );
            let ans = Confirm::new(&message)
                .with_default(false)
                .prompt()?;

            if !ans {
                println!("{}", "処理を中止しました。".yellow());
                return Ok(());
            }
        }
    }

    // ベースイメージの選択
    let base_image = if let Some(image) = cli.base_image {
        image
    } else {
        println!("{}", "テンプレート一覧を取得中...".blue());
        let templates = fetch_templates(None).await?;
        
        let template_names: Vec<String> = templates.iter()
            .map(|t| t.name.clone())
            .collect();
        
        let selected_template = Select::new("テンプレートを選択してください", template_names)
            .prompt()?;
        
        let template = templates.iter()
            .find(|t| t.name == selected_template)
            .unwrap();
        
        let template_info = fetch_template_info(&template.name, None).await?;
        
        let mut variants = template_info.options.image_variant.proposals;
        variants.insert(0, template_info.options.image_variant.default);
        
        let selected_variant = Select::new("イメージバリアントを選択してください", variants)
            .prompt()?;
        
        template_info.image.replace("${VARIANT}", &selected_variant)
    };

    // 設定ファイルの生成
    generate_devcontainer_json(&cli.dir, &container_name, &dir_name)?;
    generate_compose_yaml(&cli.dir, &container_name, &dir_name)?;
    generate_dockerfile(&cli.dir, &base_image)?;

    println!("{}", "設定ファイルの生成が完了しました。".green());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generate_devcontainer_json() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().to_path_buf();
        
        generate_devcontainer_json(&dir, "test-container", "test-dir").unwrap();
        
        let content = fs::read_to_string(dir.join("devcontainer.json")).unwrap();
        assert!(content.contains("\"name\": \"test-container\""));
        assert!(content.contains("\"workspaceFolder\": \"/workspaces/test-dir\""));
    }

    #[test]
    fn test_generate_compose_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().to_path_buf();
        
        generate_compose_yaml(&dir, "test-container", "test-dir").unwrap();
        
        let content = fs::read_to_string(dir.join("compose.yaml")).unwrap();
        assert!(content.contains("image: test-container:latest"));
        assert!(content.contains("container_name: test-container"));
        assert!(content.contains("/workspaces/test-dir"));
    }

    #[test]
    fn test_generate_dockerfile() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().to_path_buf();
        
        generate_dockerfile(&dir, "ubuntu:latest").unwrap();
        
        let content = fs::read_to_string(dir.join("Dockerfile")).unwrap();
        assert_eq!(content, "FROM ubuntu:latest");
    }

    #[test]
    fn test_cli_parse() {
        let args = vec!["coduchi", "--name", "test", "--dir", "test-dir", "--force"];
        let cli = Cli::parse_from(args);
        
        assert_eq!(cli.name, Some("test".to_string()));
        assert_eq!(cli.dir, PathBuf::from("test-dir"));
        assert!(cli.force);
    }
}
