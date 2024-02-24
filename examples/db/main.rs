use ryde::*;

#[router]
enum Route {
    #[get("/")]
    Index,
    #[post("/todos")]
    CreateTodo,
    #[allow(unused)]
    #[get("/todos/:id/edit")]
    EditTodo(i64),
    #[post("/todos/:id/edit")]
    ChangeTodo(i64),
    #[post("/todos/:id/delete")]
    DestroyTodo(i64),
}

db!(
    "create table if not exists todos (
        id integer primary key,
        content text not null,
        created_at integer not null default(unixepoch())
    )",
    "create unique index if not exists todos_content_ix on todos(content)",
    (insert_todo, "insert into todos (content) values (?)"),
    (update_todo, "update todos set content = ? where id = ?"),
    (delete_todo, "delete from todos where id = ?"),
    (todo, "select * from todos where id = ?"),
    (
        todos,
        "select * from todos order by created_at desc limit 30"
    ),
);

#[main]
async fn main() {
    serve!("localhost:3000", Route)
}

async fn index() -> Result<Response> {
    let todos = todos().await?;

    Ok(render(index_view(None, todos)))
}

fn index_view(error: Option<Error>, todos: Vec<Todos>) -> Element {
    div((
        h1("todos"),
        match error {
            Some(err) => match err {
                Error::UniqueConstraintFailed(_) => div("todo with that name already exists"),
                _ => div(()),
            },
            None => div(()),
        },
        div((todo_list(todos), todo_form(None))),
    ))
}

async fn create_todo(Form(todo): Form<InsertTodo>) -> Result<Response> {
    let result = insert_todo(todo.content)
        .await
        .map_err(|err| Error::from(err));
    let todos = todos().await?;

    Ok(match result {
        Ok(_) => redirect_to(Route::Index),
        Err(err) => render(index_view(Some(err), todos)),
    })
}

async fn edit_todo(Path(id): Path<i64>) -> Result<Response> {
    let todo = todo(id).await?.last().cloned();
    Ok(render(div((h1("edit todo"), todo_form(todo)))))
}

async fn change_todo(Path(id): Path<i64>, Form(todo): Form<UpdateTodo>) -> Result<Response> {
    let result = update_todo(todo.content, id)
        .await
        .map_err(|err| Error::from(err));
    let todos = todos().await?;

    Ok(match result {
        Ok(_) => redirect_to(Route::Index),
        Err(err) => render(index_view(Some(err), todos)),
    })
}

async fn destroy_todo(Path(id): Path<i64>) -> Result<Response> {
    let _ = delete_todo(id).await?;

    Ok(redirect_to(Route::Index))
}

fn todo_form(todo: Option<Todo>) -> Element {
    let route = match todo.as_ref() {
        Some(t) => Route::ChangeTodo(t.id),
        None => Route::CreateTodo,
    };
    let (id, content) = match todo {
        Some(t) => (t.id, t.content),
        None => (0, "".into()),
    };
    form((
        input()
            .type_("text")
            .name("content")
            .attr("autofocus", "")
            .value(content),
        input().type_("hidden").name("id").value(id),
        input().type_("submit").value("add"),
    ))
    .action(route)
    .method("POST")
}

fn delete_todo_form(id: i64) -> Element {
    form(input().type_("submit").value("delete"))
        .action(Route::DestroyTodo(id))
        .method("POST")
}

fn todo_list(todos: Vec<Todos>) -> Element {
    ul(todos.into_iter().map(todo_list_item).collect::<Vec<_>>())
}

fn todo_list_item(todo: Todos) -> Element {
    let css = css!("display: flex", "gap: 1rem");

    li((
        div((todo.id, ", ", todo.content, ", ", todo.created_at)),
        a("edit").href(Route::EditTodo(todo.id)),
        delete_todo_form(todo.id),
    ))
    .css(css)
}

fn render(element: Element) -> Response {
    document().head(()).body(element).render()
}

type Result<T> = std::result::Result<T, Error>;
