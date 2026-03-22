use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("provider error: {0}")]
    ProviderError(String),
    #[error("client error: {0}")]
    HttpError(#[from] rig::http_client::Error),
    #[error("prompt error: {0}")]
    PromptError(#[from] rig::completion::PromptError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
