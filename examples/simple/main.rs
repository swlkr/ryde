use ryde::*;

#[router]
fn router() -> Router {
    Router::new().route("/", get(get_slash))
}

#[main]
async fn main() {
    serve("::1:9001", router()).await
}

async fn get_slash() -> Html {
    html! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>ryde with rust</title>
            </head>
            <body>
                <div>you are here {url!(get_slash)}</div>
            </body>
        </html>
    }
}
