use std::sync::OnceLock;

pub use db_macros::db;
use tokio_rusqlite::Connection;
extern crate self as db;

async fn connection() -> &'static Connection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in env");
    static CONNECTION: OnceLock<Connection> = OnceLock::new();
    match CONNECTION.get() {
        Some(connection) => connection,
        None => {
            let connection = Connection::open(database_url)
                .await
                .expect("Failed to connect to db");
            connection
                .call(|conn| {
                    conn.execute_batch(
                        "PRAGMA foreign_keys = ON;
                        PRAGMA journal_mode = WAL;
                        PRAGMA synchronous = NORMAL;",
                    )
                    .map_err(|err| err.into())
                })
                .await
                .expect("Failed to connect to db");
            CONNECTION.set(connection).expect("Failed to connect to db");
            CONNECTION.get().expect("Failed to connect to db")
        }
    }
}

pub trait Query
where
    Self: Sized,
{
    fn sql() -> &'static str;
    fn is_execute() -> bool;
    fn params(&self) -> Vec<tokio_rusqlite::types::Value>;
    fn new(row: &tokio_rusqlite::Row<'_>) -> rusqlite::Result<Self>;
}

pub async fn execute<T: Query + Send + Sync + 'static>(t: T) -> tokio_rusqlite::Result<usize> {
    connection()
        .await
        .call(move |conn| {
            let params_iter = t.params();
            let params = tokio_rusqlite::params_from_iter(params_iter);
            conn.execute(T::sql(), params).map_err(|err| err.into())
        })
        .await
        .into()
}

pub async fn query<T: Query + Send + Sync + 'static>(t: T) -> tokio_rusqlite::Result<Vec<T>> {
    connection()
        .await
        .call(move |conn| {
            let sql = T::sql();
            let mut stmt = conn.prepare(sql)?;
            let params_iter = t.params();
            let params = tokio_rusqlite::params_from_iter(params_iter);
            let rows = stmt
                .query_map(params, |row| T::new(row))?
                .collect::<rusqlite::Result<Vec<T>>>();
            rows.map_err(|err| err.into())
        })
        .await
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    db!(
        "create table if not exists posts (id integer primary key not null, title text not null)",
        (InsertPost, "insert into posts (title) values (?)"),
        (SelectPost, "select title from posts where id = ?"),
        (SelectPost2, "select title, id from posts where id = ?"),
    );

    #[tokio::test]
    async fn it_works() {
        let post_id = execute(InsertPost {
            title: "title".into(),
        })
        .await
        .unwrap();
        assert_eq!(1, post_id);

        let rows: Vec<SelectPost> = query(SelectPost {
            id: 1,
            ..Default::default()
        })
        .await
        .unwrap();
        assert_eq!(rows[0].id, 0);
        assert_eq!(rows[0].title, "title");

        let rows: Vec<SelectPost2> = query(SelectPost2 {
            id: 1,
            ..Default::default()
        })
        .await
        .unwrap();
        assert_eq!(rows[0].id, 1);
        assert_eq!(rows[0].title, "title");
    }
}
