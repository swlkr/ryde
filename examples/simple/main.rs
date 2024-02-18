use ryze::*;

#[main]
async fn main() {
    serve("localhost:3000").await.unwrap()
}
