# Static files

Static files provides an easy way to declare and embed your static files.

## Declare your static files

This will embed the static files in your binary at compile time with `include_bytes!`.
It will try to find the files starting from the root of your project: `CARGO_MANIFEST_DIR`.

```rust
use ryde::*;

#[main]
async fn main() {
    serve!("localhost:3000", Routes)
}

#[router]
enum Routes {
    #[embed("/static/*file")]
    StaticFiles
}
```

## Render them

```rust
fn render() -> String {
    ryde::render((
        doctype(),
        html((
            head(render!(StaticFiles)),
            body(),
        )),
    ))
}
```

