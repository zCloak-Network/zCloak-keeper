use crate::server::config::{ServerConfig, Config};
use crate::server::task::ServerSand;
use crate::error::ServerResult;

#[async_trait::async_trait]
pub trait ServerComponent<C: ServerConfig, R> {
    fn restore<T: ServerSand>() -> ServerResult<Self>
    where
        Self: Sized,
    {
        Self::restore_with_namespace::<T>(Config::default_namespace().to_string())
    }

    fn restore_with_namespace<T: ServerSand>(namespace: String) -> ServerResult<Self>
    where
        Self: Sized;
    
    async fn component(&self) -> anyhow::Result<R>;
    fn config(&self) -> &C;
}