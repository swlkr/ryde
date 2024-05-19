#![allow(non_snake_case)]

use ryde::*;

db!(
    create_todos = "create table if not exists todos (
        id integer primary key,
        content text not null,
        created_at integer not null default(unixepoch())
    )" as Todo,
    create_todos_content_ix =
        "create unique index if not exists todos_content_ix on todos(content)",
    insert_todo = "insert into todos (content) values (?) returning *" as Todo,
    update_todo = "update todos set content = ? where id = ? returning *" as Todo,
    delete_todo = "delete from todos where id = ? returning *" as Todo,
    todo = "select todos.* from todos where id = ? limit 1" as Todo,
    todos = "select todos.* from todos order by created_at desc limit 30",
);

routes!(
    ("/", get(get_slash)),
    ("/todos", post(post_todos)),
    ("/todos/:id/edit", get(get_todos_edit).post(post_todos_edit)),
    ("/todos/:id/delete", post(post_todos_delete)),
    ("/*files", get(get_files))
);

embed_static_files!("examples/static_files/static");

#[main]
async fn main() -> Result<()> {
    let _ = create_todos().await?;
    let _ = create_todos_content_ix().await?;

    serve("::1:9001", routes()).await;

    Ok(())
}

async fn get_slash() -> Result<Html> {
    let todos = todos().await?;

    Ok(html! {
        <View>
            <GetSlash error=None todos=todos/>
        </View>
    })
}

async fn post_todos(cx: Cx, Form(todo): Form<Todo>) -> Result<Response> {
    let result = insert_todo(todo.content.clone()).await;

    match result {
        Ok(_) => Ok(redirect_to!(get_slash)),
        Err(Error::UniqueConstraintFailed(_)) => {
            let todos = todos().await?;
            Ok(cx.render(html! { <GetSlash error=Some(format!("{} already exists", todo.content)) todos=todos/> }))
        }
        Err(_) => return Err(Error::InternalServer),
    }
}

async fn get_todos_edit(Path(id): Path<i64>) -> Result<Html> {
    let todo = todo(id).await?.ok_or(Error::NotFound)?;

    Ok(html! {
        <View>
            <h1>Edit todo</h1>
            <TodoForm todo=Some(todo)/>
        </View>
    })
}

async fn post_todos_edit(
    cx: Cx,
    Path(id): Path<i64>,
    Form(todo): Form<Todo>,
) -> Result<Response> {
    let result = update_todo(todo.content.clone(), id).await;

    match result {
        Ok(_) => Ok(redirect_to!(get_slash)),
        Err(Error::UniqueConstraintFailed(_)) => {
            let todos = todos().await?;
            Ok(cx.render(html! { <GetSlash error=Some(format!("{} already exists", todo.content)) todos=todos/> }))
        }
        Err(_) => return Err(Error::InternalServer),
    }
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
            <input type="hidden" name=name.created_at value=0/>
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

fn TodoList(todos: Vec<Todos>) -> Component {
    html! { <table>{todos.iter().map(TodoListRow)}</table> }
}

fn GetSlash(error: Option<String>, todos: Vec<Todos>) -> Component {
    html! {
        <div>
            <h1>todos</h1>
            <div>
                <TodoForm todo=None/>
                <TodoList todos=todos/>
                <div>{error}</div>
            </div>
        </div>
    }
}

fn TodoListRow(todo: &Todos) -> Component {
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
    fn render(&self, elements: Elements) -> Response {
        html! { <View>{elements}</View> }.into_response()
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
