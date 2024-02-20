extern crate self as ryde;
pub use axum::extract::*;
pub use axum::*;
pub use router::{router, Routes};
pub use serde::*;
pub use static_files::{self, StaticFiles};
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
    ($ip:expr, $ident:ident) => {
        server($ip, $ident::router())
    };
}

#[cfg(test)]
mod tests {
    use ryde::*;

    fn it_works() {}
}
