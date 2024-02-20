use ryde::*;

#[main]
async fn main() {
    serve!("localhost:3000", Routes).await.unwrap()
}

#[router]
enum Routes {
    #[allow(unused)]
    #[embed]
    #[folder("examples/static_files/static")]
    StaticFiles,
}
