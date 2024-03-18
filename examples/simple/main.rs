use ryde::*;

route!((get, "/", index));

listen!("::1:3000");

async fn index() -> Response {
    res!(Route::Index)
}
