use ryde::*;

route!((get, "/", index));

fn main() {
    serve!("::1:3000")
}

async fn index() -> Response {
    res!(Route::Index)
}
