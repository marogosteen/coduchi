use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 設定ファイルを生成するディレクトリを指定
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// devcontainer.jsonのnameプロパティを指定
    #[arg(short, long)]
    pub name: Option<String>,

    /// ベースイメージを指定
    #[arg(long)]
    pub base_image: Option<String>,

    /// 既存ファイルの上書きを確認せずに実行
    #[arg(short, long)]
    pub force: bool,
}

impl Cli {
    pub fn get_dir_name(&self) -> String {
        self.dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    pub fn get_container_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.get_dir_name())
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
    fn test_get_dir_name() {
        let cli = Cli {
            dir: PathBuf::from("test-dir"),
            name: None,
            base_image: None,
            force: false,
        };
        
        assert_eq!(cli.get_dir_name(), "test-dir");
    }

    #[test]
    fn test_get_container_name() {
        let cli = Cli {
            dir: PathBuf::from("test-dir"),
            name: Some("custom-name".to_string()),
            base_image: None,
            force: false,
        };
        
        assert_eq!(cli.get_container_name(), "custom-name");
    }
}
