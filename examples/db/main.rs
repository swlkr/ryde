#![allow(non_snake_case)]

use ryde::*;

db!(
    create_todos = "create table if not exists todos (
        id integer primary key,
        content text not null,
        created_at integer not null default(unixepoch())
    )",
    create_todos_content_ix =
        "create unique index if not exists todos_content_ix on todos(content)",
    insert_todo = "insert into todos (content) values (?) returning content",
    update_todo = "update todos set content = ? where id = ? returning id, content",
    delete_todo = "delete from todos where id = ?",
    todo = "select todos.* from todos where id = ? limit 1",
    todos = "select todos.* from todos order by created_at desc limit 30",
);

routes!(
    ("/", get(get_slash)),
    ("/todos", post(post_todos)),
    ("/todos/:id/edit", get(get_todos_edit).post(post_todos_edit)),
    ("/todos/:id/delete", post(post_todos_delete))
);

#[main]
async fn main() {
    create_todos().await.unwrap();
    create_todos_content_ix().await.unwrap();

    serve("::1:9001", routes()).await
}

async fn get_slash() -> Result<Html> {
    let todos = todos().await?;

    Ok(html! {
        <View>
            <GetSlash msg="" todos=todos/>
        </View>
    })
}

async fn post_todos(Form(todo): Form<InsertTodo>) -> Result<Response> {
    let result = insert_todo(todo.content).await;

    let res = match is_unique!(result)? {
        true => redirect_to!(get_slash),
        false => {
            let todos = todos().await?;
            html! {
                <View>
                    <GetSlash msg="todos already exists" todos=todos/>
                </View>
            }.into_response()
        }
    };

    Ok(res)
}

async fn get_todos_edit(Path(id): Path<i64>) -> Result<Html> {
    let todo = todo(id).await?.ok_or(Error::NotFound)?;

    Ok(html! {
        <View>
            <h1>Edit todo</h1>
            <EditTodoForm todo=&todo/>
        </View>
    })
}

async fn post_todos_edit(Path(id): Path<i64>, Form(todo): Form<UpdateTodo>) -> Result<Response> {
    let result = update_todo(todo.content, id).await;

    let res = match is_unique!(result)? {
        true => redirect_to!(get_slash),
        false => {
            let todos = todos().await?;
            html! {
                <View>
                    <GetSlash msg="todo already exists" todos=todos/>
                </View>
            }.into_response()
        }
    };

    Ok(res)
}

async fn post_todos_delete(Path(id): Path<i64>) -> Result<Response> {
    let _ = delete_todo(id).await?;

    Ok(redirect_to!(get_slash))
}

fn NewTodoForm() -> Component {
    let name = InsertTodo::names();

    html! {
        <form method="post" action=url!(post_todos)>
            <input type="text" name=name.content autofocus/>
            <input type="submit" value="add"/>
        </form>
    }
}

fn EditTodoForm(todo: &Todo) -> Component {
    let name = UpdateTodo::names();

    html! {
        <form method="post" action=url!(post_todos_edit, todo.id)>
            <input type="text" name=name.content autofocus value=&todo.content/>
            <input type="hidden" name=name.id value=todo.id/>
            <input type="submit" value="add"/>
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
    html! { <ul>{todos.iter().map(TodoListItem).collect::<Vec<_>>()}</ul> }
}

fn GetSlash(msg: &'static str, todos: Vec<Todos>) -> Component {
    html! {
        <div>
            <h1>todos</h1>
            <div>
                <NewTodoForm/>
                <TodoList todos=todos/>
                <div>{msg}</div>
            </div>
        </div>
    }
}

fn TodoListItem(todo: &Todos) -> Component {
    html! {
        <li>
            <div>{todo.id} {&todo.content} {todo.created_at}</div>
            <a href=url!(get_todos_edit, todo.id)>edit</a>
            <DeleteTodoForm id=todo.id/>
        </li>
    }
}

fn View(elements: Elements) -> Component {
    html! {
        <!DOCTYPE html> 
        <html>
            <head></head>
            <body>{elements}</body>
        </html>
    }
}
