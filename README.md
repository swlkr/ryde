# ryde

ryde is a single person, single file web development library for rust

# Install

```sh
cargo new your-project
cd your-project
cargo add ryde
mkdir static
```

# Quickstart

Open up your-project/src/main.rs in your favorite editor

```rust
use ryde::*;

routes!(
    ("/", get(get_slash)),
    ("/*files", get(get_files)) // serves the static files from the root ::1:3000/test.css, ::1:3000/app.js
);

embed_static_files!("static");

#[main]
async fn main() {
    serve("::1:9001", routes()).await
}

async fn index() -> Html {
    html! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <title>ryde with rust</title>
                {render_static_files!("static")}
            </head>
            <body>
                <h1 class="text-2xl">ryde with rust</h1>
            </body>
        </html>
    }
}
```

# A longer example

```rust
use ryde::*;

db!(
    create_todos = "
        create table if not exists todos (
            id integer primary key,
            content text not null,
            created_at integer not null default(unixepoch())
        );",
    insert_todo = "insert into todos (content) values (?)",
    todos = "select * from todos order by created_at desc limit 30"
);

routes!(
    ("/", get(get_slash)),
    ("/todos", get(get_todos).post(post_todos)),
    ("/*files", get(files_handler))
    with Arc<AppState>
);

#[derive(Clone)]
struct AppState {
    some_state: String
};

embed_static_files!("static");

#[main]
async fn main() {
    create_todos().await;
    let routes = routes().with(Arc::new(AppState { some_state: "".into() }));

    serve("::1:9002", routes).await
}

async fn get_slash() -> Html {
    html! { 
        <View>
            <h1>ryde with rust</h1>
            <a href={url!(get_todos)}>check your todos</a>
        </View>
    }
}

async fn get_todos() -> Result<Html> {
    let todos = todos().await?;

    Ok(html! { 
        <View>
            <TodoList />
        </View>
    })
}

async fn post_todos(Form(todo): Form<InsertTodo>) -> Result<Response> {
    let _todo = insert_todo(todo.content).await?;

    Ok(redirect_to!(get_todos))
}

fn Form(action: &str, elements: Elements) -> Component {
    html! {
        <form method="post" action={action}>
            {elements}
        </form>
    }
}

fn TodoForm() -> Component {
    let InsertTodoNames { content } = InsertTodo::names();

    html! {
        <Form action={url!(todos_create)}>
            <input type="text" name={content} />
            <input type="submit" name="save" />
        </Form>
    }
}

fn TodoList(todos: Vec<Todos>) -> Component {
    html! {
        <div class="text-black dark:text-white">
            <h1 class="text-2xl">todos</h1>
            <ul>{todos.iter().map(|todo| html! { <TodoListItem todo=todo /> })}</ul>
            <TodoForm />
        </div>
    }
}

fn TodoListItem(todo: Todos) -> Component {
    html! { <li>{&todo.content}</li> }
}

fn View(elements: Elements) -> Component {
    html! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>ryde with rust</title>
                {render_static_files!()}
            </head>
            <body>
                <div class="text-white bg-orange-500 dark:bg-orange-700">{elements}</div>
            </body>
        </html>
    }
}
```

# More examples

Clone the repo and check out the rest of examples!
