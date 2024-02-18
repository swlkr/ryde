extern crate self as ryze;
use axum::Router;
use std::sync::OnceLock;
pub use tokio::main;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static ROUTER: OnceLock<Router> = OnceLock::new();

pub async fn serve(ip: &str) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(ip).await?;
    println!("Listening on {}", ip);
    let router = ROUTER.get_or_init(|| Router::new());
    axum::serve(listener, router.clone()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use ryze::*;

    fn it_works() {}
}
