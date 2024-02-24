extern crate self as ryde;
pub use axum::extract::*;
pub use axum::http;
pub use axum::http::header::*;
pub use axum::response::*;
pub use axum::*;
pub use axum_extra::extract::*;
pub use axum_extra::headers;
pub use cookie::Cookie;
pub use css::css;
pub use db::db;
pub use db::rusqlite;
pub use db::tokio_rusqlite;
pub use html::*;
pub use router::{router, Routes};
pub use serde::*;
pub use static_files::{self, StaticFiles};
pub use std::fmt::Display;
pub use tokio::main;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn server(ip: &str, router: Router) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(ip).await?;
    println!("Listening on {}", ip);
    axum::serve(listener, router).await?;
    Ok(())
}

#[macro_export]
macro_rules! serve {
    ($ip:expr, $ident:ident) => {
        server($ip, $ident::router()).await.unwrap()
    };
}

#[macro_export]
macro_rules! render {
    ($ident:ident) => {
        $ident::render()
    };
}

pub type Html = axum::response::Html<String>;

pub struct Document {
    head: Element,
    body: Element,
}

impl Document {
    fn new() -> Self {
        Self {
            head: head(()),
            body: body(()),
        }
    }

    pub fn head(mut self, children: impl Render + 'static) -> Self {
        self.head = anon_element(children);
        self
    }

    pub fn body(mut self, children: impl Render + 'static) -> Self {
        let styles = styles(&children);
        let inner_head = html::render(self.head).replace("<>", "").into();
        self.head = head((Raw(inner_head), style(Raw(styles))));
        self.body = body(children);
        self
    }

    pub fn render(self) -> Response {
        Html(html::render((doctype(), html((self.head, self.body))))).into_response()
    }
}

pub fn document() -> Document {
    Document::new()
}

pub fn render(element: Element) -> Html {
    Html(html::render(element))
}

pub fn redirect_to(route: impl Display) -> Response {
    let headers = [
        (SET_COOKIE, format!("flash={}", "")),
        (LOCATION, route.to_string()),
    ];

    (http::StatusCode::SEE_OTHER, headers).into_response()
}

pub fn redirect_with_flash(route: impl Display, message: String) -> Response {
    let headers = [
        (SET_COOKIE, format!("flash={}", message)),
        (LOCATION, route.to_string()),
    ];

    (http::StatusCode::SEE_OTHER, headers).into_response()
}

pub enum Error {
    DatabaseConnectionClosed,
    DatabaseClose,
    Database(String),
    UniqueConstraintFailed(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = match self {
            Error::DatabaseConnectionClosed => "db connection closed".into(),
            Error::DatabaseClose => "db closed".into(),
            Error::Database(err) => err,
            Error::UniqueConstraintFailed(columns) => columns,
        };
        Response::builder().status(500).body(body.into()).unwrap()
    }
}

impl From<tokio_rusqlite::Error> for Error {
    fn from(value: tokio_rusqlite::Error) -> Self {
        match value {
            tokio_rusqlite::Error::ConnectionClosed => Error::DatabaseConnectionClosed,
            tokio_rusqlite::Error::Close(_) => Error::DatabaseClose,
            tokio_rusqlite::Error::Rusqlite(err) => {
                // TODO: follow the white rabbit to the actual error for unique constraints
                let s = err.to_string();
                if s.starts_with("UNIQUE constraint failed: ") {
                    Error::UniqueConstraintFailed(
                        s.split(":").map(|s| s.trim()).last().unwrap_or("").into(),
                    )
                } else {
                    Error::Database(s)
                }
            }
            tokio_rusqlite::Error::Other(err) => Error::Database(err.to_string()),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ryde::*;

    fn it_works() {}
}
