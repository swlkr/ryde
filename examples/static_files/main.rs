use ryde::*;

route!((get, "/*files", files_handler));

serve_static_files!("examples/static_files/static", files_handler);

fn main() {
    serve!("localhost:3000")
}
