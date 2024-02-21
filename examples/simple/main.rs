use ryde::*;

#[main]
async fn main() {
    serve!("localhost:3000", Route)
}

async fn index() -> String {
    Route::Index.to_string()
}

#[router]
enum Route {
    #[get("/")]
    Index,
}
