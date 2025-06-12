pub mod cli;
pub mod template;
pub mod generator;
pub mod error;

pub use cli::Cli;
pub use error::Error;
pub use template::{Template, TemplateInfo};
pub use generator::{generate_devcontainer_json, generate_compose_yaml, generate_dockerfile};

pub type Result<T> = std::result::Result<T, Error>;
