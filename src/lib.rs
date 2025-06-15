// ライブラリクレートのルートモジュール
// 軽量オニオンアーキテクチャ + DIP の実装

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

// レガシー互換性のために一時的に残存
pub mod error;

// 明示的なパブリックAPIの公開（名前衝突を避ける）
pub use domain::{models, ports, services as domain_services};
pub use application::{use_cases, services as application_services};
pub use infrastructure::{repositories, ui};
pub use presentation::{cli, container};
pub use error::Error;

// 結果型のエイリアス
pub type Result<T> = std::result::Result<T, anyhow::Error>;

// メイン実行関数（バックワード互換性のため）
pub async fn execute_generation(cli: presentation::Cli) -> anyhow::Result<()> {
    let container = presentation::Container::create();
    let use_case = application::use_cases::GenerateDevContainerUseCase::new(
        container.template_repository(),
        container.file_repository(),
        container.user_interaction(),
        container.progress_reporter(),
    );
    let request = cli.to_request();
    
    let response = use_case.execute(request).await?;
    
    if !response.success {
        return Err(anyhow::anyhow!("{}", response.message));
    }
    
    Ok(())
} 