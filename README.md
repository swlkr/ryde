# ryde

ryde is a single person, single file web development library for rust

# Install

```sh
cargo new your-project
cd your-project
cargo add ryde
```

# A simple example

Open up your-project/src/main.rs in your favorite editor

```rust
// src/main.rs
use ryde::*;

#[router]
enum Route {
    #[get("/")]
    Index
    #[allow(unused)]
    #[embed]
    StaticFiles
}

#[main]
async fn main() {
    serve!("localhost:3000", Route);
}

async fn index() -> Response {
    document()
        .head((title("ryde with rust"), render!(StaticFiles)))
        .body(
            h1("ryde with rust")
                .css(css!(
                    "font-size: var(--font-size-2)",
                    "line-height: var(--line-height-2)"
                ))
        )
        .render()
}
```

# A more complete example

```rust
// src/main.rs

db!(
    "create table if not exists todos (
        id integer primary key,
        content text not null,
        created_at integer not null default(unixepoch())
    )",
    (insert_todo, "insert into todos (content) values (?)"),
    (todos, "select * from todos order by created_at desc limit 30")
)

#[router]
enum Route {
    #[get("/")]
    TodosIndex,
    #[post("/todos")]
    TodosCreate,
    #[allow(unused)]
    #[embed]
    StaticFiles
}

#[main]
async fn main() {
    serve!("localhost:3000", Route)
}

async fn todos_index() -> Result<Response> {
    let todos = todos().await?;
    Ok(render_todos_index(todos))
}

async fn todos_create(Form(todo): Form<InsertTodo>) -> Result<Response> {
    let _todo = insert_todo(todo.content).await?
    Ok(redirect_to(Route::TodosIndex))
}

fn render_todos_index(todos: Vec<Todos>) -> Response {
    render(div((
        h1("todos").css(css!(
            "font-size: var(--font-size-2)",
            "line-height: var(--line-height-2)",
            "color: var(--gray-9)",
            "dark:color: var(--amber-5)",
        )),
        ul(todos.iter().map(|todo| li((todo.content))).collect::<Vec<_>>()),
        todo_form()
    )))
}

fn todo_form() -> Element {
    form((
        input().type_("text").name("content").value(content),
        input().type_("submit").name("save")
    ))
    .method("POST")
    .action(Route::TodosCreate)
}

fn render(element: Element) -> Response {
    document()
        .head(())
        .body(
            div(element)
                .css(css!(
                    "background: var(--gray-1)",
                    "dark:background: var(--gray-9)"
                ))
        )
        .render()
}
```

# More examples

Clone the repo and check out the examples!

```sh
cargo run --example simple
```
