#![allow(non_snake_case)]

use ryde::*;

#[router]
fn router() -> Router {
    Router::new()
        .route("/", get(get_slash))
        .route("/todos", post(post_todos))
        .route("/todos/:id/edit", get(get_todos_edit).post(post_todos_edit))
        .route("/todos/:id/delete", post(post_todos_delete))
        .route("/*files", get(get_files))
}

#[main]
async fn main() -> Result<()> {
    let _ = create_todos().await?;

    serve("::1:9001", router()).await;

    Ok(())
}

async fn get_slash(cx: Cx) -> Result<Html> {
    let todos = todos().await?;

    Ok(cx.render(html! { <GetSlash todos=todos/> }))
}

#[derive(Serialize, Deserialize)]
struct TodoParams {
    content: String,
}

async fn post_todos(Form(form): Form<TodoParams>) -> Result<Response> {
    let _ = insert_todo(form.content).await?;

    Ok(redirect_to!(get_slash))
}

async fn get_todos_edit(cx: Cx, Path(id): Path<i64>) -> Result<Html> {
    let todo = todo(id).await?.ok_or(Error::NotFound)?;

    Ok(cx.render(html! {
        <h1>Edit todo</h1>
        <TodoForm todo=Some(todo)/>
    }))
}

async fn post_todos_edit(Path(id): Path<i64>, Form(form): Form<TodoParams>) -> Result<Response> {
    let _todo = update_todo(form.content, id).await?;

    Ok(redirect_to!(get_slash))
}

async fn post_todos_delete(Path(id): Path<i64>) -> Result<Response> {
    let _ = delete_todo(id).await?;

    Ok(redirect_to!(get_slash))
}

fn TodoForm(todo: Option<Todo>) -> Component {
    let name = Todo::names();
    let action = match todo {
        Some(ref todo) => url!(post_todos_edit, todo.id),
        None => url!(post_todos),
    };
    let todo = todo.unwrap_or_default();

    html! {
        <form method="post" action=action>
            <input type="text" name=name.content autofocus value=todo.content/>
            <input type="hidden" name=name.id value=todo.id/>
            <input type="submit" value="save"/>
        </form>
    }
}

fn DeleteTodoForm(id: i64) -> Component {
    html! {
        <form method="post" action=url!(post_todos_delete, id)>
            <input type="submit" value="delete"/>
        </form>
    }
}

fn TodoList(todos: Vec<Todo>) -> Component {
    html! { <table>{todos.iter().map(TodoListRow)}</table> }
}

fn GetSlash(todos: Vec<Todo>) -> Component {
    html! {
        <div>
            <h1>todos</h1>
            <div>
                <TodoForm todo=None/>
                <TodoList todos=todos/>
            </div>
        </div>
    }
}

fn TodoListRow(todo: &Todo) -> Component {
    html! {
        <tr>
            <td>{todo.id}</td>
            <td>{&todo.content}</td>
            <td>{todo.created_at}</td>
            <td>
                <a href=url!(get_todos_edit, todo.id)>edit</a>
            </td>
            <td>
                <DeleteTodoForm id=todo.id/>
            </td>
        </tr>
    }
}

fn View(elements: Elements) -> Component {
    html! {
        <!DOCTYPE html>
        <html>
            <head>{render_static_files!()}</head>
            <body>{elements}</body>
        </html>
    }
}

struct Cx;

impl Cx {
    fn render(&self, elements: Elements) -> Html {
        html! { <View>{elements}</View> }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Cx
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        Ok(Cx {})
    }
}

db!(
    create_todos = "create table if not exists todos (
        id integer primary key,
        content text unique not null,
        created_at integer not null default(unixepoch())
    )" as Todo,
    insert_todo = "
        insert into todos (content)
        values (?)
        on conflict do
        update set content = excluded.content
        returning *" as Todo,
    update_todo = "update todos set content = ? where id = ? returning *" as Todo,
    delete_todo = "delete from todos where id = ?",
    todo = "select todos.* from todos where id = ? limit 1" as Todo,
    todos = "select todos.* from todos order by created_at desc limit 30" as Vec<Todo>,
);

embed_static_files!("examples/static_files/static");
