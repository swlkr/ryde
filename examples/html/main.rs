use ryde::*;

#[router]
enum Routes {
    #[get("/")]
    Index,
    #[embed]
    #[folder("examples/html/static")]
    #[allow(unused)]
    StaticFiles,
}

#[main]
async fn main() {
    serve!("localhost:3000", Routes)
}

async fn index() -> Html {
    render(div((h1(Routes::Index), p("ryde with rust! 🐎"))))
}

fn h1(route: Routes) -> Element {
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

fn p(s: impl Display) -> Element {
    let css = css!(
        "font-size: 16px",
        "font-family: sans-serif",
        "color: var(--gray-950)",
        "background: var(--amber-500)",
        "dark:color: var(--amber-300)",
        "dark:background: var(--gray-950)"
    );

    ryde::p(s.to_string()).css(css)
}

fn render(element: Element) -> Html {
    document().head(render!(StaticFiles)).body(element).render()
}
