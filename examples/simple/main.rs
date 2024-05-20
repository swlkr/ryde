use ryde::*;

#[main]
async fn main() {
    serve("::1:9001", router()).await
}

async fn get_slash() -> String {
    url!(get_slash)
}

#[router]
fn router() -> Router {
    Router::new().route("/", get(get_slash))
}
