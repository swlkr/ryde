use ryde::*;

#[router]
fn router() -> Router {
    Router::new().route("/*files", get(get_files))
}

embed_static_files!("examples/static_files/static");

#[main]
async fn main() {
    serve("localhost:9001", router()).await
}
