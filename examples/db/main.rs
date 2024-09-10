#![allow(non_snake_case)]

use ryde::*;

#[router]
fn router(cx: Cx) -> Router {
    Router::new()
        .route("/", get(get_slash))
        .route("/todos", post(todos_create))
        .route("/todos/:id/edit", get(todos_edit).post(todos_update))
        .route("/todos/:id/delete", post(todos_delete))
        .with_state(cx)
}

#[main]
async fn main() -> Result<()> {
    // let db_url = dotenv("DATABASE_URL").expect("DATABASE_URL not found");
    let db = db(":memory:").await?;
    let _ = db.create_todos().await?;
    let cx = Cx { db };
    serve("::1:9001", router(cx)).await;

    Ok(())
}

async fn get_slash(cx: Cx, db: Db) -> Result<Html> {
    let todos = db.todos().await?;

    Ok(cx.render(html! { <GetSlash todos=todos/> }))
}

#[derive(Deserialize)]
struct TodoParams {
    content: String,
}

async fn todos_create(db: Db, Form(form): Form<TodoParams>) -> Result<Response> {
    let _todo = db.insert_todo(form.content).await?;

    Ok(redirect_to!(get_slash))
}

async fn todos_edit(cx: Cx, db: Db, Path(id): Path<i64>) -> Result<Html> {
    let todo = db.todo(id).await?.ok_or(Error::NotFound)?;

    Ok(cx.render(html! {
        <h1>Edit todo</h1>
        <TodoForm todo=Some(todo)/>
    }))
}

async fn todos_update(
    db: Db,
    Path(id): Path<i64>,
    Form(form): Form<TodoParams>,
) -> Result<Response> {
    let _todo = db.update_todo(form.content, id).await?;

    Ok(redirect_to!(get_slash))
}

async fn todos_delete(db: Db, Path(id): Path<i64>) -> Result<Response> {
    let _x = db.delete_todo(id).await?;

    Ok(redirect_to!(get_slash))
}

fn TodoForm(todo: Option<Todo>) -> Component {
    let name = Todo::names();
    let action = match todo {
        Some(ref todo) => url!(todos_update, todo.id),
        None => url!(todos_create),
    };
    let todo = todo.unwrap_or_default();

    html! {
        <form method="post" action=action>
            <input type="text" name=name.content autofocus value=todo.content/>
            <input type="submit" value="save"/>
        </form>
    }
}

fn DeleteTodoForm(id: i64) -> Component {
    html! {
        <form method="post" action=url!(todos_delete, id)>
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
                <a href=url!(todos_edit, todo.id)>edit</a>
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
            <head>
                <title>ryde db example</title>
            </head>
            <body>{elements}</body>
        </html>
    }
}

#[derive(Clone, RequestParts)]
struct Cx {
    db: Db,
}

impl Cx {
    fn render(&self, elements: Elements) -> Html {
        html! { <View>{elements}</View> }
    }
}

db!(
    let create_todos = r#"
        create table if not exists todos (
            id integer primary key,
            content text unique not null,
            created_at integer not null default(unixepoch())
        )
    "# as Todo;

    let insert_todo = r#"
        insert into todos (content)
        values (?)
        on conflict do
        update set content = excluded.content
        returning *
    "# as Todo;

    let update_todo = r#"
        update todos
        set content = ?
        where id = ?
        returning *
    "# as Todo;

    let delete_todo = r#"
        delete
        from todos
        where id = ?
    "#;

    let todo = r#"
        select todos.*
        from todos
        where id = ?
        limit 1
    "# as Todo;

    let todos = r#"
        select todos.*
        from todos
        order by created_at desc
        limit 30
    "# as Vec<Todo>;
);
