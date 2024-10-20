# ryde

ryde is a single person, single file  web framework for rust

# Install

```sh
cargo add --git https://github.com/swlkr/ryde
```

# Quickstart

```rust
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
```

# More examples

Clone the repo and check out the rest of examples!
