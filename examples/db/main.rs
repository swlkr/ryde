use ryde::*;

db!(
    create_todos = "create table if not exists todos (
        id integer primary key,
        content text not null,
        created_at integer not null default(unixepoch())
    )",
    create_todos_content_ix =
        "create unique index if not exists todos_content_ix on todos(content)",
    insert_todo = "insert into todos (content) values (?) returning todos.*",
    update_todo = "update todos set content = ? where id = ? returning todos.*",
    delete_todo = "delete from todos where id = ?",
    todo = "select todos.* from todos where id = ? limit 1",
    todos = "select todos.* from todos order by created_at desc limit 30",
);

routes!(
    ("/", get(index)),
    ("/todos", post(todos_create)),
    ("/todos/:id/edit", get(todos_edit).post(todos_update)),
    ("/todos/:id/delete", get(todos_delete))
);

#[main]
async fn main() {
    create_todos().await.unwrap();
    create_todos_content_ix().await.unwrap();
    serve("::1:3000", routes()).await;
}

async fn index() -> Result<Response> {
    let todos = todos().await?;

    Ok(render(index_view("", todos)))
}

async fn todos_create(Form(todo): Form<InsertTodo>) -> Result<Response> {
    let result = insert_todo(todo.content).await;

    let res = match is_unique!(result)? {
        true => redirect_to(url!(index)),
        false => {
            let todos = todos().await?;
            render(index_view("todo already exists", todos))
        }
    };

    Ok(res)
}

async fn todos_edit(Path(id): Path<i64>) -> Result<Response> {
    let todo = todo(id).await?;
    Ok(render(div((h1("edit todo"), todo_form(todo)))))
}

async fn todos_update(Path(id): Path<i64>, Form(todo): Form<UpdateTodo>) -> Result<Response> {
    let result = update_todo(todo.content, id).await;

    let res = match is_unique!(result)? {
        true => redirect_to(url!(index)),
        false => {
            let todos = todos().await?;
            render(index_view("todo already exists", todos))
        }
    };

    Ok(res)
}

async fn todos_delete(Path(id): Path<i64>) -> Result<Response> {
    let _ = delete_todo(id).await?;

    Ok(redirect_to(url!(index)))
}

fn todo_form(todo: Option<Todo>) -> Element {
    let route = match &todo {
        Some(t) => url!(todos_update, t.id),
        None => url!(todos_create),
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
        .action(url!(todos_delete, id))
        .method("POST")
}

fn todo_list(todos: Vec<Todos>) -> Element {
    ul(todos.into_iter().map(todo_list_item).collect::<Vec<_>>())
}

fn index_view(msg: &'static str, todos: Vec<Todos>) -> Element {
    div((
        h1("todos"),
        div((todo_list(todos), todo_form(None))),
        div(msg),
    ))
}

fn todo_list_item(todo: Todos) -> Element {
    let css = css!("display: flex", "gap: 1rem");

    li((
        div((todo.id, ", ", todo.content, ", ", todo.created_at)),
        a("edit").href(url!(todos_edit, todo.id)),
        delete_todo_form(todo.id),
    ))
    .css(css)
}

fn render(element: Element) -> Response {
    document().head(()).body(element).render()
}
