use ryde::*;

route!(
    (get, "/", index),
    (post, "/todos", post_todos),
    (get, "/todos/:id/edit", todos_edit, i64),
    (put, "/todos/:id", todos_update, i64),
    (get, "/orgs/:org_id/todos/:id", org_todos_update, i64, i64),
    (get, "/search", search, q: Option<String>)
);

fn main() {
    serve!("::1:3000")
}

async fn index() -> String {
    Route::Index.to_string()
}

async fn post_todos() -> String {
    Route::PostTodos.to_string()
}

async fn todos_edit(Path(id): Path<i64>) -> String {
    Route::TodosEdit(id).to_string()
}

async fn todos_update(Path(id): Path<i64>) -> String {
    Route::TodosUpdate(id).to_string()
}

async fn org_todos_update(Path((org_id, id)): Path<(i64, i64)>) -> String {
    Route::OrgTodosUpdate(org_id, id).to_string()
}

#[derive(Deserialize)]
struct Search {
    q: Option<String>,
}

async fn search(Query(Search { q }): Query<Search>) -> String {
    Route::Search { q }.to_string()
}
