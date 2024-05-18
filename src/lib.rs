extern crate self as ryde;

mod db;
mod html;
mod router;

pub use axum;
pub use axum::extract::*;
pub use axum::http;
pub use axum::http::header::*;
pub use axum::http::Uri;
pub use axum::response::*;
pub use axum::*;
pub use axum_extra::extract::*;
pub use axum_extra::headers;
pub use cookie::Cookie;
pub use db::{connection, db, rusqlite, tokio_rusqlite, Connection};
pub use html::{component, escape, html, Component, Elements, Render};
pub use router::{routes, url};
pub use ryde_macros::{env_vars, StaticFiles};
pub use serde;
pub use serde::*;
pub use std::fmt::Display;
pub use tokio;
pub use tokio::*;

pub type Result<T> = std::result::Result<T, Error>;

pub fn server(ip: &str, router: Router) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { serve(ip, router).await });
    Ok(())
}

pub async fn serve(ip: &str, router: Router) {
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    println!("Listening on {}", ip);
    axum::serve(listener, router).await.unwrap();
}

#[macro_export]
macro_rules! render_static_files {
    () => {{
        Assets::render()
    }};
}

pub type Html = Component;

impl IntoResponse for Html {
    fn into_response(self) -> Response {
        axum::response::Html(self.html).into_response()
    }
}

pub fn redirect(s: String) -> Response {
    let headers = [(SET_COOKIE, format!("flash={}", "")), (LOCATION, s.into())];

    (http::StatusCode::SEE_OTHER, headers).into_response()
}

#[macro_export]
macro_rules! redirect_to {
    ($ident:ident $(,$expr:expr)*) => {{
        let _ = $ident;
        redirect(url!($ident, $($expr),*))
    }}
}

#[macro_export]
macro_rules! id {
    ($($idents:ident),*) => {{
        $(let _ = &$idents;)*
        vec![$((stringify!($idents)),)*].join(" ")
    }};
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    DatabaseConnectionClosed,
    DatabaseClose,
    Database(String),
    UniqueConstraintFailed(String),
    Io(String),
    NotFound,
    InternalServer,
    Multipart(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DatabaseConnectionClosed => f.write_str("Error: Database connection closed"),
            Error::DatabaseClose => f.write_str("Error: Database was already closed"),
            Error::Database(e) => f.write_fmt(format_args!("Error: Generic database error {}", e)),
            Error::UniqueConstraintFailed(e) => {
                f.write_fmt(format_args!("Error: Unique constraint failed {}", e))
            }
            Error::Io(e) => f.write_fmt(format_args!("Error: Io error {}", e)),
            Error::NotFound => f.write_str("Error: Not found"),
            Error::InternalServer => f.write_str("Error: Internal server error"),
            Error::Multipart(e) => f.write_fmt(format_args!("Error: {}", e)),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn ser::StdError + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn ser::StdError> {
        self.source()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Error::DatabaseConnectionClosed => (500, "db connection closed".into()),
            Error::DatabaseClose => (500, "db closed".into()),
            Error::Database(err) => (500, err),
            Error::UniqueConstraintFailed(columns) => (500, columns),
            Error::Io(s) => (500, s),
            Error::NotFound => (404, "not found".into()),
            Error::InternalServer => (500, "internal server error".into()),
            Error::Multipart(_s) => (
                422,
                "Unprocessable entity from multipart form request".into(),
            ),
        };
        Response::builder()
            .status(status)
            .body(body.into())
            .unwrap()
    }
}

impl From<tokio_rusqlite::Error> for Error {
    fn from(value: tokio_rusqlite::Error) -> Self {
        match value {
            tokio_rusqlite::Error::ConnectionClosed => Error::DatabaseConnectionClosed,
            tokio_rusqlite::Error::Close(_) => Error::DatabaseClose,
            tokio_rusqlite::Error::Rusqlite(err) => err.into(),
            tokio_rusqlite::Error::Other(err) => Error::Database(err.to_string()),
            _ => unreachable!(),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        match value {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound,
            err => {
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
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value.to_string())
    }
}

impl From<axum_extra::extract::multipart::MultipartError> for Error {
    fn from(value: axum_extra::extract::multipart::MultipartError) -> Self {
        Error::Multipart(value.body_text())
    }
}

#[macro_export]
macro_rules! embed_static_files {
    ($folder:expr) => {
        embed_static_files!($folder, "/", get_files);
    };
    ($folder:expr, $prefix:expr) => {
        embed_static_files!($folder, $prefix, get_files);
    };

    ($folder:expr, $prefix:expr, $ident:ident) => {
        #[derive(ryde::StaticFiles)]
        #[folder($folder)]
        #[prefix($prefix)]
        pub struct Assets;

        pub async fn $ident(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
            match Assets::get(uri.path()) {
                Some((content_type, bytes)) => (
                    axum::http::StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, content_type)],
                    bytes,
                ),
                None => (
                    axum::http::StatusCode::NOT_FOUND,
                    [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    "not found".as_bytes(),
                ),
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::env_vars;

    #[test]
    fn env_works() {
        std::env::set_var("HOME", "SWEET HOME");
        std::env::set_var("HAVE_CAKE", "EAT IT TOO");
        env_vars!(HOME, HAVE_CAKE);
        // loads statics
        env_vars();
        assert_eq!(HOME(), "SWEET HOME");
        // changing has no effect
        std::env::set_var("HOME", "NO PLACE LIKE IT");
        assert_eq!(HOME(), "SWEET HOME");

        // n idents works
        assert_eq!(HAVE_CAKE(), "EAT IT TOO");
    }
}
