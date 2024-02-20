use ryde::*;

#[main]
async fn main() {
    serve!("localhost:3000", Routes).await.unwrap()
}

async fn index() -> Html {
    render(html((head(render!(StaticFiles)), body(h1(Routes::Index)))))
}

#[router]
enum Routes {
    #[get("/")]
    Index,
    #[embed]
    #[folder("examples/static_files/static")]
    #[allow(unused)]
    StaticFiles,
}
