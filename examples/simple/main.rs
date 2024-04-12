use ryde::*;

routes!(("/", get(index)));

#[main]
async fn main() {
    serve("::1:9001", routes()).await
}

async fn index() -> String {
    url!(index)
}
