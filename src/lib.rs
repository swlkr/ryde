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
pub use ryde_db::db;
pub use ryde_db::rusqlite;
pub use ryde_db::tokio_rusqlite;
pub use ryde_html::{self as html, *};
pub use ryde_router;
pub use ryde_router::{params, route, router, Routes};
pub use ryde_static_files::{self as static_files, StaticFiles};
pub use serde;
pub use serde::*;
pub use std::fmt::Display;
pub use tokio::*;

pub type Result<T> = std::result::Result<T, Error>;

pub fn server(ip: &str, router: Router) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
        println!("Listening on {}", ip);
        axum::serve(listener, router).await.unwrap();
    });
    Ok(())
}

#[macro_export]
macro_rules! serve {
    ($ip:expr) => {
        server($ip, Route::router()).unwrap()
    };
    ($ip:expr, $router:expr) => {
        server($ip, $router).unwrap()
    };
}

#[macro_export]
macro_rules! render_static_files {
    () => {{
        Assets::render()
    }};
}

#[macro_export]
macro_rules! res {
    ($expr:expr) => {{
        impl IntoResponse for Route {
            fn into_response(self) -> Response {
                self.to_string().into_response()
            }
        }

        $expr.into_response()
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

#[macro_export]
macro_rules! listen {
    ($expr:expr) => {
        fn main() {
            serve!($expr);
        }
    };
}

#[macro_export]
macro_rules! serve_static_files {
    ($expr:expr, $ident:ident) => {
        #[derive(static_files::StaticFiles)]
        #[folder($expr)]
        pub struct Assets;

        pub async fn files_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
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
    use ryde::*;

    fn it_works() {}
}
