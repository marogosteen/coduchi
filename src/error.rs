use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IOエラー: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTPリクエストエラー: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSONパースエラー: {0}")]
    Json(#[from] serde_json::Error),

    #[error("テンプレートエラー: {0}")]
    Template(String),

    #[error("設定エラー: {0}")]
    Config(String),

    #[error("ユーザー入力エラー: {0}")]
    UserInput(String),
}

impl From<inquire::InquireError> for Error {
    fn from(e: inquire::InquireError) -> Self {
        Error::UserInput(e.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Config(e.to_string())
    }
}
