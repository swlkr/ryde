# enum_router

enum_router is a rust enum router for axum that doesn't support nesting.

```sh
cargo add enum_router # still not on crates.io (yet)
```

# Declare your routes

```rust
use enum_router::router;

#[router]
pub enum Route {
  #[get("/")]
  Root,
  #[get("/todos/:id/edit")]
  EditTodo(i32)
  #[put("/todos/:id")]
  UpdateTodo(i32)
}
```

It will complain about missing functions which you still have to write:

```rust
async fn root() -> String {
  Route::Root.to_string() // "/"
}

async fn edit_todo(Path(id): Path<i32>) -> String {
  Route::EditTodo(id).to_string() // "/todos/:id/edit"
}

async fn update_todo(Path(id): Path<i32>) -> String {
  Route::UpdateTodo(id).to_string() // "/todos/:id"
}
```

# Use it like this

```rust
#[tokio::main]
async fn main() {
  let ip = "127.0.0.1:9001";
  let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
  let router = Route::router();
  axum::serve(listener, router).await.unwrap();
}
```

# Got state?

```rust
use std::sync::Arc;
use axum::extract::State;

struct AppState {
  count: u64
}

#[router(Arc<AppState>)]
enum Routes {
  #[get("/")]
  Index
}

async fn index(State(_st): State<Arc<AppState>>) -> String {
  Routes::Index.to_string()
}

#[tokio::main]
async fn main() {
  let ip = "127.0.0.1:9001";
  let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
  let router = Routes::router().with_state(Arc::new(AppState { count: 0 }));
  axum::serve(listener, router).await.unwrap();
}
```
