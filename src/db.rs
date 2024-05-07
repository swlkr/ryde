use std::sync::OnceLock;

pub use rusqlite;
pub use ryde_macros::db;
pub use tokio_rusqlite::{self, Connection};
extern crate self as ryde_db;

static CONNECTION: OnceLock<Connection> = OnceLock::new();

pub async fn connection() -> &'static Connection {
    let database_url = std::env::var("DATABASE_URL").unwrap_or("db.sqlite3".into());
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

#[cfg(test)]
mod tests {
    use super::*;
    pub use serde::{self, Deserialize, Serialize};
    use tokio::test;

    db!(
        initial_schema = "
            create table if not exists posts (
                id integer primary key not null,
                title text not null,
                test integer
            );
            create table if not exists likes (
                id integer primary key not null,
                post_id integer not null references posts(id)
            );
            create table if not exists items (value integer not null);",
        insert_post = "
            insert into posts (title, test)
            values (?, ?)
            returning title, test
        ",
        select_posts = "select id, title, test from posts",
        select_post = "select id, title, test from posts where id = ? limit 1",
        like_post = "insert into likes (post_id) values (?) returning id, post_id",
        select_likes = "
            select
                likes.id,
                likes.post_id,
                posts.title
            from
                likes
            join
                posts on posts.id = likes.post_id
            where
                likes.id = ?",
        update_post = "
            update posts
            set title = ?, test = ?
            where id = ?
            returning id, title, test",
        delete_like = "delete from likes where id = ?",
        delete_post = "delete from posts where id = ?",
        post_count = "select count(*) from posts",
        insert_select = "
            with all_items as (
              select 1 as value
              union all
              select value + 1 from all_items where value < 10
            )
            insert into items select value from all_items",
        select_first_item = "select items.* from items order by items.value limit 1",
        select_items = "select items.* from items order by items.value"
    );

    #[test]
    async fn it_works() {
        std::env::set_var("DATABASE_URL", ":memory:");
        initial_schema().await.unwrap();
        let post: Option<InsertPost> = insert_post("title".into(), Some(1)).await.unwrap();
        let post = post.unwrap();
        assert_eq!(post.title, "title");
        assert_eq!(post.test, Some(1));

        let post: SelectPost = select_post(1).await.unwrap().unwrap();
        assert_eq!(post.id, 1);
        assert_eq!(post.title, "title");
        assert_eq!(post.test, Some(1));

        let likes = like_post(1).await.unwrap().unwrap();
        assert_eq!(likes.post_id, 1);
        let likes = select_likes(likes.id).await.unwrap();
        assert_eq!(likes[0].post_id, 1);
        assert_eq!(likes[0].title, "title");

        let post = update_post("new title".into(), Some(2), 1)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(post.id, 1);
        assert_eq!(post.title, "new title");
        assert_eq!(post.test, Some(2));

        let _like = delete_like(1).await.unwrap();
        let _post = delete_post(1).await.unwrap();

        let posts = select_posts().await.unwrap();
        assert_eq!(posts.len(), 0);

        let post_count = post_count().await.unwrap();
        assert_eq!(post_count, 0);

        let _ = insert_select().await.unwrap();
        let first_item = select_first_item().await.unwrap();
        assert_eq!(first_item.unwrap().value, 1);

        let items = select_items().await.unwrap();
        assert_eq!(1, items.first().unwrap().value);
        assert_eq!(10, items.last().unwrap().value);
    }
}
