use ryde::*;

routes!(("/*files", get(files_handler)));

serve_static_files!("examples/static_files/static", files_handler);

#[main]
async fn main() {
    serve("localhost:9001", routes()).await
}
