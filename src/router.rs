extern crate self as router;

pub use ryde_macros::{routes, url};

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        self,
        body::Body,
        extract::{Path, Query},
        http::{Request, StatusCode},
        response::IntoResponse,
        Router,
    };
    use http_body_util::BodyExt;
    use serde::Deserialize;
    use tower::ServiceExt;

    routes!(
        ("/", get(index)),
        ("/login", get(login_form).post(login).patch(login)),
        ("/abc", get(abc)),
        ("/xyz/:xyz", get(xyz))
    );

    async fn index() -> impl IntoResponse {
        url!(index)
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
        let path = url!(abc);
        let query_string = format!("?abc={}", params.abc.unwrap_or_default());

        format!("{}{}", path, query_string)
    }

    async fn xyz(Path(s): Path<String>) -> impl IntoResponse {
        url!(xyz, s) // -> "/xyz/abc"
    }

    #[tokio::test]
    async fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        let router = routes();

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

        routes!(("/", get(index)) with Arc<AppState>);

        struct AppState {
            #[allow(unused)]
            a: String,
        }

        async fn index(State(_s): State<Arc<AppState>>) -> String {
            url!(index)
        }

        let app = routes().with_state(Arc::new(AppState { a: "".into() }));

        assert_eq!(
            (StatusCode::OK, "/".into()),
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
