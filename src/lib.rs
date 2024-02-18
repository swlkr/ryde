extern crate self as ryze;
use axum::Router;
pub use enum_router::{router, Routes};
pub use tokio::main;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn server(ip: &str, router: Router) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(ip).await?;
    println!("Listening on {}", ip);
    axum::serve(listener, router).await?;
    Ok(())
}

#[macro_export]
macro_rules! serve {
    ($ip:expr) => {
        server($ip, axum::Router::new())
    };

    ($ip:expr, $ident:ident) => {
        server($ip, $ident::router())
    };
}

#[cfg(test)]
mod tests {
    use ryze::*;

    fn it_works() {}
}
