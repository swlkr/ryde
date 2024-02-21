use ryde::*;

#[main]
async fn main() {
    serve!("localhost:3000", Route).await.unwrap()
}

async fn index() -> Result<Html> {
    let rows = query!(UserCount).await?;

    render(div((h1(Route::Index), user_list(rows))))
}

fn user_list(rows: Vec<UserCount>) -> Element {
    ul(rows.iter().map(user_list_item))
}

fn user_list_item(row: &UserCount) -> Element {
    li((row.user_id.unwrap_or("n/a"), ": ", row.count.unwrap_or(0)))
}

fn h1(s: Route) -> Element {
    let class = css!(
        "color: var(--gray-950)",
        "hover|color: var(--gray-850)",
        "dark:color: var(--gray-500)",
        "dark:hover:color: var(--gray-300)".
    );

    ryde::h1(s).css(class)
}

fn render(element: Element) -> Result<Html> {
    Ok(document().head(render!(StaticFiles)).body(element).render())
}

#[router]
enum Route {
    #[get("/")]
    Index,
    #[embed]
    Files,
}
