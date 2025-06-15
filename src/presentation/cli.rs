use clap::Parser;
use std::path::PathBuf;

/// Coduchi CLI引数
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 設定ファイルを生成するディレクトリを指定
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// devcontainer.jsonのnameプロパティを指定
    #[arg(short, long)]
    pub name: Option<String>,

    /// compose.yamlのimageフィールドを指定
    #[arg(long)]
    pub image_name: Option<String>,

    /// compose.yamlのcontainer_nameフィールドを指定
    #[arg(long)]
    pub container_name: Option<String>,

    /// ベースイメージを指定
    #[arg(long)]
    pub base_image: Option<String>,

    /// 既存ファイルの上書きを確認せずに実行
    #[arg(short, long)]
    pub force: bool,
}

impl Cli {
    /// CLI引数をApplication層のリクエストに変換する
    pub fn to_request(self) -> crate::application::GenerateDevContainerRequest {
        crate::application::GenerateDevContainerRequest {
            dir: self.dir,
            name: self.name,
            image_name: self.image_name,
            container_name: self.container_name,
            base_image: self.base_image,
            force: self.force,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse() {
        let args = vec!["coduchi", "--name", "test", "--dir", "test-dir", "--force"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.name, Some("test".to_string()));
        assert_eq!(cli.dir, PathBuf::from("test-dir"));
        assert!(cli.force);
    }

    #[test]
    fn test_cli_defaults() {
        let args = vec!["coduchi"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.name, None);
        assert_eq!(cli.dir, PathBuf::from("."));
        assert_eq!(cli.image_name, None);
        assert_eq!(cli.container_name, None);
        assert_eq!(cli.base_image, None);
        assert!(!cli.force);
    }

    #[test]
    fn test_to_request() {
        let cli = Cli {
            dir: PathBuf::from("test-dir"),
            name: Some("test-container".to_string()),
            image_name: Some("my-app:dev".to_string()),
            container_name: Some("dev-instance".to_string()),
            base_image: Some("ubuntu:latest".to_string()),
            force: true,
        };

        let request = cli.to_request();
        assert_eq!(request.dir, PathBuf::from("test-dir"));
        assert_eq!(request.name, Some("test-container".to_string()));
        assert_eq!(request.image_name, Some("my-app:dev".to_string()));
        assert_eq!(request.container_name, Some("dev-instance".to_string()));
        assert_eq!(request.base_image, Some("ubuntu:latest".to_string()));
        assert!(request.force);
    }
} 