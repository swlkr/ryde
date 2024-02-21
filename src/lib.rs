extern crate self as ryde;
pub use axum::extract::*;
pub use axum::response::*;
pub use axum::*;
pub use css::css;
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

    pub fn render(self) -> Html {
        Html(html::render((doctype(), html((self.head, self.body)))))
    }
}

pub fn document() -> Document {
    Document::new()
}

pub fn render(element: Element) -> Html {
    Html(html::render(element))
}

#[cfg(test)]
mod tests {
    use ryde::*;

    fn it_works() {}
}
