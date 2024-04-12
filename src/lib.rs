extern crate self as ryde;
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
pub use ryde_css::css;
pub use ryde_db;
pub use ryde_db::{connection, db, rusqlite, tokio_rusqlite, Connection};
pub use ryde_html::{self as html, *};
pub use ryde_router;
pub use ryde_router::{routes, url};
pub use ryde_static_files::{self as static_files, StaticFiles};
pub use serde;
pub use serde::*;
pub use std::fmt::Display;
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

pub fn x_redirect(route: impl Display) -> Response {
    let headers = [("x-location", route.to_string())];
    (http::StatusCode::OK, headers).into_response()
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

#[macro_export]
macro_rules! is_unique {
    ($expr:expr) => {
        is_unique($expr.map_err(Error::from).err())
    };
}

pub fn is_unique(err: Option<Error>) -> Result<bool> {
    let Some(err) = err else {
        return Ok(true);
    };

    match err {
        Error::UniqueConstraintFailed(_) => Ok(false),
        err => Err(err),
    }
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
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
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
macro_rules! serve_static_files {
    ($expr:expr) => {
        serve_static_files!($expr, files_handler);
    };

    ($expr:expr, $ident:ident) => {
        #[derive(static_files::StaticFiles)]
        #[folder($expr)]
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
