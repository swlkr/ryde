use ryde::*;

routes!(
    ("/", get(index)),
    ("/todos", post(post_todos)),
    ("/todos/:id/edit", get(todos_edit)),
    ("/todos/:id", put(todos_update)),
    ("/orgs/:org_id/todos/:id", get(org_todos_update)),
    ("/search", get(search))
);

#[main]
async fn main() {
    serve("::1:9001", routes()).await
}

async fn index() -> String {
    url!(index)
}

async fn post_todos() -> String {
    url!(post_todos)
}

async fn todos_edit(Path(id): Path<i64>) -> String {
    url!(todos_edit, id)
}

async fn todos_update(Path(id): Path<i64>) -> String {
    url!(todos_update, id)
}

async fn org_todos_update(Path((org_id, id)): Path<(i64, i64)>) -> String {
    url!(org_todos_update, org_id, id)
}

#[derive(Serialize, Deserialize)]
struct Search {
    q: Option<String>,
}

async fn search(Query(Search { q }): Query<Search>) -> String {
    format!("{}?q={}", url!(search), q.unwrap_or_default())
}
