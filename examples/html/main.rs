use ryde::*;

routes!(("/", get(index)), ("/*files", get(files_handler)));
serve_static_files!("examples/html/static", files_handler);

#[main]
async fn main() {
    serve("::1:3000", routes()).await
}

async fn index() -> Response {
    render(div((h1(url!(index)), p("ryde with rust! 🐎"))))
}

fn h1(route: String) -> Element {
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

fn render(element: Element) -> Response {
    document()
        .head(render_static_files!())
        .body(element)
        .render()
}
