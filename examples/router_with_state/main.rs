use ryde::*;

#[main]
async fn main() {
    serve("::1:9001", router()).await
}

async fn get_slash(State(st): State<Arc<St>>) -> String {
    dbg!(&st.0);
    url!(get_slash)
}

#[derive(Debug)]
struct St(String);

#[router]
fn router() -> Router {
    Router::new()
        .route("/", get(get_slash))
        .with_state(Arc::new(St("".into())))
}
