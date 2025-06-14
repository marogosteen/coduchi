use clap::Parser;
use coduchi::{execute_generation, cli::Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    execute_generation(cli).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cli_parse() {
        let args = vec!["coduchi", "--name", "test", "--dir", "test-dir", "--force"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.name, Some("test".to_string()));
        assert_eq!(cli.dir, PathBuf::from("test-dir"));
        assert!(cli.force);
    }
}
