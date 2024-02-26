use ryde::*;

route!((get, "/*files", static_files));

fn main() {
    serve!("localhost:3000")
}

async fn static_files(uri: Uri) -> Response {
    serve_static_files!("examples/static_files/static", uri)
}
