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
// src/main.rs
use ryde::*;

route!(
    (get, "/", index),
    (get, "/*files", files_handler) // serves the static files from the root ::1:3000/test.css, ::1:3000/app.js
);

fn main() {
    serve!("::1:3000");
}

async fn index() -> Response {
    document()
        .head((title("ryde with rust"), render_static_files!()))
        .body(
            h1("ryde with rust")
                .css(css!(
                    "font-size: 1.5rem",
                    "line-height: 2rem"
                ))
        )
        .render()
}

// embeds all files in the static/ folder
// and serves them
serve_static_files!("static", files_handler);
```

# A longer example

```rust
// src/main.rs
use ryde::*;

db!(
    "create table if not exists todos (
        id integer primary key,
        content text not null,
        created_at integer not null default(unixepoch())
    )",
    (insert_todo, "insert into todos (content) values (?)"),
    (todos, "select * from todos order by created_at desc limit 30")
);

route!(
    (get, "/", todos_index),
    (post, "/todos", todos_create),
    (get, "/*files", files_handler)
);

serve_static_files!("static", files_handler);

fn main() {
    serve!("::1:3000")
}

async fn todos_index() -> Result<Response> {
    let todos = todos().await?;
    Ok(render_todos_index(todos))
}

async fn todos_create(Form(todo): Form<InsertTodo>) -> Result<Response> {
    let _todo = insert_todo(todo.content).await?;
    Ok(redirect_to(Route::TodosIndex))
}

fn render_todos_index(todos: Vec<Todos>) -> Response {
    render(div((
        h1("todos").css(css!(
            "font-size: 1.5rem",
            "line-height: 2rem",
        )),
        ul(todos.iter().map(|todo| li(todo.content.clone())).collect::<Vec<_>>()),
        todo_form()
    )).css(css!(
        "color: rgb(3 7 18)",
        "dark:color: rgb(245 158 11)")))
}

fn todo_form() -> Element {
    form((
        input().type_("text").name("content"),
        input().type_("submit").name("save")
    ))
    .method("POST")
    .action(Route::TodosCreate)
}

fn render(element: Element) -> Response {
    document()
        .head(render_static_files!())
        .body(
            div(element)
                .css(css!(
                    "background: rgb(243 244 246)",
                    "dark:background: rgb(3 7 18)"
                ))
        )
        .render()
}
```

# More examples

Clone the repo and check out the rest of examples!
