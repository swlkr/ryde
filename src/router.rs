extern crate self as router;

pub use ryde_macros::{router, routes, url};

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        self,
        body::Body,
        extract::{Path, Query, Request},
        http::StatusCode,
        response::IntoResponse,
        routing::get,
        Router,
    };
    use http_body_util::BodyExt;
    use serde::Deserialize;
    use tower::ServiceExt;

    #[router]
    fn router() -> Router {
        Router::new()
            .route("/", get(get_slash))
            .route("/login", get(login_form).post(login).patch(login))
            .route("/abc", get(abc))
            .route("/xyz/:xyz", get(xyz))
    }

    async fn get_slash() -> impl IntoResponse {
        url!(get_slash)
    }

    async fn login_form() -> impl IntoResponse {
        url!(login_form)
    }

    async fn login() -> impl IntoResponse {
        url!(login)
    }

    #[derive(Debug, Deserialize)]
    struct Abc {
        abc: Option<u8>,
    }

    async fn abc(Query(params): Query<Abc>) -> impl IntoResponse {
        // TODO: let path = url!(abc, abc = params.abc.unwrap_or_default());
        let path = url!(abc);
        let query_string = format!("?abc={}", params.abc.unwrap_or_default());

        format!("{}{}", path, query_string)
    }

    async fn xyz(Path(s): Path<String>) -> impl IntoResponse {
        url!(xyz, s) // -> "/xyz/abc"
    }

    #[tokio::test]
    async fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        let router = router();

        assert_eq!(
            (StatusCode::OK, "/".into()),
            make_request(&router, "GET", "/").await
        );
        assert_eq!(
            (StatusCode::OK, "/login".into()),
            make_request(&router, "GET", "/login").await
        );
        assert_eq!(
            (StatusCode::OK, "/login".into()),
            make_request(&router, "POST", "/login").await
        );
        assert_eq!(
            (StatusCode::NOT_FOUND, "".into()),
            make_request(&router, "GET", "/nope").await
        );
        assert_eq!(
            (StatusCode::OK, "/abc?abc=0".into()),
            make_request(&router, "GET", "/abc").await
        );
        assert_eq!(
            (StatusCode::OK, "/abc?abc=1".into()),
            make_request(&router, "GET", "/abc?abc=1").await
        );
        assert_eq!(
            (StatusCode::OK, "/xyz/abc".into()),
            make_request(&router, "GET", "/xyz/abc").await
        );

        Ok(())
    }

    #[tokio::test]
    async fn state_works() -> Result<(), Box<dyn std::error::Error>> {
        use axum::extract::State;
        use std::sync::Arc;

        #[router]
        fn router() -> Router {
            Router::default()
                .route("/", get(index))
                .with_state(Arc::new(AppState("state".into())))
        }

        struct AppState(String);

        async fn index(State(s): State<Arc<AppState>>) -> String {
            format!("get / with {}", s.0)
        }

        let app = router();

        assert_eq!(
            (StatusCode::OK, "get / with state".into()),
            make_request(&app, "GET", "/").await
        );

        Ok(())
    }

    fn request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    async fn make_request(app: &Router, method: &str, uri: &str) -> (StatusCode, String) {
        let response = app.clone().oneshot(request(method, uri)).await.unwrap();
        (
            response.status(),
            String::from_utf8(
                response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes()
                    .to_vec(),
            )
            .unwrap(),
        )
    }
}
