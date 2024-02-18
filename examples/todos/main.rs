use ryde::*;

#[main]
async fn main() {
    execute!(
        "create table if not exists todos (id integer primary key, content text not null, updated_at integer not null, created_at integer not null)",
        "create unique index todos_content_ix on todos (text)",
    )
    .await
    .unwrap();
    serve("localhost:9007").await.unwrap()
}

async fn index() -> Result<Html> {
    let rows = query!(
        Row,
        "select count(id) as count, user_id
        from posts
        group by user_id",
    )
    .await?;

    Ok((
        doctype(),
        html((
            head(Files::head()),
            body(heading(Route::Index), user_list(rows)),
        )),
    ))
}

fn user_list(rows: Vec<Row>) -> Element {
    ul(rows.iter().map(user_list_item).collect::<Vec<_>>())
}

fn user_list_item(row: &Row) -> Element {
    li((
        row.get("user_id").unwrap_or("n/a"),
        ":",
        row.get("count").unwrap_or(0),
    ))
}

fn heading(s: Display) -> Element {
    let class = css!(
        "color: --gray-950",
        "hover | color: --gray-850",
        "dark | color: --gray-500",
        "dark:hover | color: --gray-300".
    );

    h1(s).class(class)
}

#[router]
enum Route {
    #[get("/")]
    Index,
    #[embed]
    Files,
}
