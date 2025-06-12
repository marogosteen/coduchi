use crate::Result;
use std::path::PathBuf;
use colored::*;

pub fn generate_devcontainer_json(dir: &PathBuf, name: &str, dir_name: &str) -> Result<()> {
    let content = format!(
        r#"{{
  "name": "{}",
  "dockerComposeFile": "compose.yaml",
  "workspaceFolder": "/workspaces/{}",
  "service": "app",
  "customizations": {{
    "vscode": {{
      "extensions": []
    }}
  }}
}}"#,
        name, dir_name
    );

    std::fs::write(dir.join("devcontainer.json"), content)?;
    println!("{}", "devcontainer.jsonを生成しました。".green());
    Ok(())
}

pub fn generate_compose_yaml(dir: &PathBuf, name: &str, dir_name: &str) -> Result<()> {
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
        name, name, dir_name
    );

    std::fs::write(dir.join("compose.yaml"), content)?;
    println!("{}", "compose.yamlを生成しました。".green());
    Ok(())
}

pub fn generate_dockerfile(dir: &PathBuf, base_image: &str) -> Result<()> {
    let content = format!("FROM {}", base_image);
    std::fs::write(dir.join("Dockerfile"), content)?;
    println!("{}", "Dockerfileを生成しました。".green());
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
}
