use std::sync::OnceLock;

pub use rusqlite;
pub use ryde_db_macros::db;
pub use serde::{Deserialize, Serialize};
pub use tokio_rusqlite::{self, Connection};
extern crate self as ryde_db;

async fn connection() -> &'static Connection {
    let database_url = std::env::var("DATABASE_URL").unwrap_or("db.sqlite3".into());
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

pub async fn query_one<T: Clone + Query + Send + Sync + 'static>(
    t: T,
) -> tokio_rusqlite::Result<Option<T>> {
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
            match rows {
                Ok(rows) => Ok(rows.last().cloned()),
                Err(err) => Err(err.into()),
            }
        })
        .await
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    db!(
        "create table if not exists posts (id integer primary key not null, title text not null)",
        (
            insert_post,
            "insert into posts (title) values (?) returning *"
        ),
        (select_posts, "select * from posts where id = ?"),
    );

    #[test]
    async fn it_works() {
        let posts = insert_post("title".into()).await.unwrap();
        assert_eq!(posts[0].title, "title");

        let found_posts = select_posts(1).await.unwrap();
        // assert_eq!(found_posts);

        // let rows = query(Post{
        //     id: 1,
        //     ..Default::default()
        // })
        // .await
        // .unwrap();
        // assert_eq!(rows[0].id, 1);
        // assert_eq!(rows[0].title, "title");
    }
}
