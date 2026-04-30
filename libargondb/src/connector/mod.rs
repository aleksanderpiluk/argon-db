use std::sync::Arc;

use async_trait::async_trait;

#[async_trait]
pub trait ConnectorHandle: Send + Sync {
    async fn close(self: Box<Self>);
}

#[derive(Debug, Clone)]
pub enum ConnectorError {
    UnexpectedError(Arc<Box<dyn std::error::Error + Send + Sync>>),
}

impl std::fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedError(e) => write!(f, "Unexpected connection error: {}", e),
        }
    }
}

impl std::error::Error for ConnectorError {}
