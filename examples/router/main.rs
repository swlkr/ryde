use ryde::*;

#[main]
async fn main() {
    serve!("localhost:3000", Routes).await.unwrap()
}

async fn index() -> String {
    Routes::Index.to_string()
}

#[router]
enum Routes {
    #[get("/")]
    Index,
}
