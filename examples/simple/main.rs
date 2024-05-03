use ryde::*;

routes!(("/", get(get_slash)));

#[main]
async fn main() {
    serve("::1:9001", routes()).await
}

async fn get_slash() -> String {
    url!(get_slash)
}
