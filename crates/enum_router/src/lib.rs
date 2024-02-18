pub use enum_router_macros::{router, Routes};
extern crate self as enum_router;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::{Path, Query};
    use axum::http::{Request, StatusCode};
    use axum::response::IntoResponse;
    use axum::Router;
    use enum_router::router;
    use serde::Deserialize;
    use tower::ServiceExt;

    async fn index() -> impl IntoResponse {
        "index"
    }

    async fn login_form() -> impl IntoResponse {
        "login form"
    }

    async fn login() -> impl IntoResponse {
        "login"
    }

    #[derive(Deserialize)]
    struct Abc {
        abc: Option<u8>,
    }

    async fn abc(Query(abc): Query<Abc>) -> impl IntoResponse {
        Route::Abc { abc: abc.abc }.to_string()
    }

    async fn xyz(Path(xyz): Path<String>) -> impl IntoResponse {
        format!("/xyz?xyz={}", xyz)
    }

    #[allow(unused)]
    #[derive(Routes, Debug, PartialEq)]
    pub enum Route {
        #[get("/")]
        Index,
        #[get("/login")]
        LoginForm,
        #[post("/login")]
        Login,
        #[get("/abc")]
        Abc { abc: Option<u8> },
        #[get("/xyz/:xyz")]
        Xyz(String),
    }

    #[tokio::test]
    async fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        let app = Route::router();

        assert_eq!(StatusCode::OK, make_request(&app, "GET", "/").await);
        assert_eq!(StatusCode::OK, make_request(&app, "GET", "/login").await);
        assert_eq!(StatusCode::OK, make_request(&app, "POST", "/login").await);
        assert_eq!(
            StatusCode::NOT_FOUND,
            make_request(&app, "GET", "/nope").await
        );
        assert_eq!(StatusCode::OK, make_request(&app, "GET", "/abc").await);
        assert_eq!(
            StatusCode::OK,
            make_request(&app, "GET", "/abc?abc=123").await
        );

        Ok(())
    }

    #[tokio::test]
    async fn state_works() -> Result<(), Box<dyn std::error::Error>> {
        use axum::extract::State;
        use std::sync::Arc;

        struct AppState {
            #[allow(unused)]
            a: String,
        }

        #[router(Arc<AppState>)]
        #[allow(unused)]
        enum Route {
            #[get("/")]
            Index,
        }

        async fn index(State(_s): State<Arc<AppState>>) -> &'static str {
            "index"
        }

        let app = Route::router().with_state(Arc::new(AppState { a: "".into() }));

        assert_eq!(StatusCode::OK, make_request(&app, "GET", "/").await);

        Ok(())
    }

    fn request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    async fn make_request(app: &Router, method: &str, uri: &str) -> StatusCode {
        app.clone()
            .oneshot(request(method, uri))
            .await
            .unwrap()
            .status()
    }
}
