#![allow(non_snake_case)]

use ryde::*;

#[router]
fn router() -> Router {
    Router::new()
        .route("/", get(get_slash))
        .route("/protected", get(get_protected))
}

#[main]
async fn main() {
    serve("::1:9001", router()).await
}

async fn get_slash(cx: Cx) -> Result<Html> {
    cx.render(html! { <div>"hello you are here: " <span>{url!(get_slash)}</span></div> })
}

// curl localhost:9001/protected -> 404
// curl localhost:9001/protected?auth -> 404
// curl localhost:9001/protected?auth=true -> 200
async fn get_protected(cx: Cx, _user: User) -> Result<Html> {
    cx.render(html! { <div>this route is protected</div> })
}

fn View(elements: Elements) -> Component {
    html! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <title>errors</title>
            </head>
            <body>{elements}</body>
        </html>
    }
}

struct Cx;

impl Cx {
    fn render(self, elements: Elements) -> Result<Html> {
        Ok(html! { <View>{elements}</View> })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Cx
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        Ok(Cx {})
    }
}

struct User;

#[derive(Serialize, Deserialize)]
struct AuthQuery {
    auth: Option<bool>,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let Query(AuthQuery { auth }) = Query::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::NotFound)?;
        match auth {
            Some(true) => Ok(User {}),
            Some(false) | None => Err(Error::NotFound),
        }
    }
}
