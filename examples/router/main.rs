use ryde::*;

#[main]
async fn main() {
    serve!("localhost:3000", Routes)
}

async fn index() -> String {
    Routes::Index.to_string()
}

async fn create_todo() -> String {
    Routes::CreateTodo.to_string()
}

async fn edit_todo(Path(id): Path<i64>) -> String {
    Routes::EditTodo(id).to_string()
}

async fn update_todo(Path(id): Path<i64>) -> String {
    Routes::UpdateTodo(id).to_string()
}

async fn update_org_todo(Path((org_id, id)): Path<(i64, i64)>) -> String {
    Routes::UpdateOrgTodo(org_id, id).to_string()
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

async fn search(Query(SearchParams { q }): Query<SearchParams>) -> String {
    Routes::Search {
        q: q.unwrap_or_default(),
    }
    .to_string()
}

#[router]
enum Routes {
    #[get("/")]
    Index,
    #[post("/todos")]
    CreateTodo,
    #[get("/todos/:id/edit")]
    EditTodo(i64),
    #[put("/todos/:id")]
    UpdateTodo(i64),
    #[put("/org/:org_id/todos/:id")]
    UpdateOrgTodo(i64, i64),
    #[get("/search")]
    Search { q: String },
}
