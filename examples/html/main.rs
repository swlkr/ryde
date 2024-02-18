use ryze::*;

#[main]
async fn main() {
    serve!("::1:3000").await.unwrap()
}
