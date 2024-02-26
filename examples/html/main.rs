use ryde::*;

route!((get, "/", index), (get, "/*files", static_files),);

fn main() {
    serve!("::1:3000")
}

async fn index() -> Response {
    render(div((h1(Route::Index), p("ryde with rust! 🐎"))))
}

fn h1(route: Route) -> Element {
    let css = css!(
        "font-size: var(--font-size-2)",
        "line-height: var(--line-height-2)",
        "color: var(--gray-950)",
        "background: var(--amber-500)",
        "dark:color: var(--amber-300)",
        "dark:background: var(--gray-950)"
    );

    ryde::h1(format!("You are here {}", route)).css(css)
}

fn p(s: &'static str) -> Element {
    let css = css!(
        "font-size: 16px",
        "font-family: sans-serif",
        "color: var(--gray-950)",
        "background: var(--amber-500)",
        "dark:color: var(--amber-300)",
        "dark:background: var(--gray-950)"
    );

    ryde::p(s).css(css)
}

async fn static_files(uri: Uri) -> Response {
    serve_static_files!(uri)
}

fn render(element: Element) -> Response {
    document()
        .head(render_static_files!())
        .body(element)
        .render()
}

embed_static_files!("examples/html/static");
