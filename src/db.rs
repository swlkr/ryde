pub use rusqlite;
pub use ryde_macros::db;
pub use tokio_rusqlite::{self, Connection};
extern crate self as ryde_db;

#[cfg(test)]
mod tests {
    use super::*;
    pub use serde::{self, Deserialize, Serialize};
    use tokio::test;

    db! {
        let create_posts = r#"
            create table if not exists posts (
                id integer primary key not null,
                title text not null,
                test integer
            )"# as Post;

        let create_likes = r#"
            create table if not exists likes (
                id integer primary key not null,
                post_id integer not null references posts(id)
            )"# as Like;

        let create_items = r#"
            create table if not exists items (
                value integer not null
            )"# as Item;

        let insert_post = r#"
            insert into posts (title, test)
            values (?, ?)
            returning *
        "# as Post;

        let select_posts = r#"
            select posts.*
            from posts
        "# as Vec<Post>;

        let select_post = r#"
            select posts.*
            from posts
            where id = ?
            limit 1
        "# as Post;

        let like_post = r#"
            insert into likes (post_id)
            values (?)
            returning *
        "# as Like;

        let select_likes = r#"
            select
                likes.id,
                likes.post_id,
                posts.title
            from
                likes
            join
                posts on posts.id = likes.post_id
            where
                likes.id = ?
        "#;

        let update_post = r#"
            update posts
            set title = ?, test = ?
            where id = ?
            returning *
        "# as Post;

        let delete_like = r#"
            delete from likes
            where id = ?
            returning *
        "# as Like;

        let delete_post = r#"
            delete
            from posts
            where id = ?
            returning *
        "# as Post;

        let post_count = r#"
            select count(*)
            from posts
        "#;

        let insert_select = r#"
            with all_items as (
              select 1 as value
              union all
              select value + 1 from all_items where value < 10
            )
            insert into items select value from all_items"#;

        let select_first_item = r#"
            select items.*
            from items
            order by items.value
            limit 1
        "# as Item;

        let select_items = r#"
            select items.*
            from items
            order by items.value
        "# as Vec<Item>;

        let create_post =
            r#"
            insert into posts (id, title)
            values (?, ?)
            on conflict (id)
            do nothing
            returning *
        "# as Post;
    }

    #[test]
    async fn it_works() -> ryde::Result<()> {
        let db = db(":memory:").await?;
        let _ = db.create_posts().await?;
        let _ = db.create_likes().await?;
        let _ = db.create_items().await?;
        let new_post = db.insert_post("title".into(), Some(1)).await?;
        assert_eq!(new_post.title, "title");
        assert_eq!(new_post.test, Some(1));

        let post = db.select_post(1).await?.unwrap();
        assert_eq!(post, new_post);

        let likes = db.like_post(1).await?;
        assert_eq!(likes.post_id, 1);
        let likes = db.select_likes(likes.id).await?;
        assert_eq!(likes[0].post_id, 1);
        assert_eq!(likes[0].title, "title");

        let post = db.update_post("new title".into(), Some(2), 1).await?;
        assert_eq!(post.id, 1);
        assert_eq!(post.title, "new title");
        assert_eq!(post.test, Some(2));

        let _like = db.delete_like(1).await?;
        let _post = db.delete_post(1).await?;

        let posts = db.select_posts().await?;
        assert_eq!(posts.len(), 0);

        let post_count = db.post_count().await?;
        assert_eq!(post_count, 0);

        let _ = db.insert_select().await?;
        let first_item = db.select_first_item().await?;
        assert_eq!(first_item.unwrap().value, 1);

        let items = db.select_items().await?;
        assert_eq!(1, items.first().unwrap().value);
        assert_eq!(10, items.last().unwrap().value);

        let post = db.create_post(1, String::default()).await?;
        assert_eq!(true, post.is_some());

        let post = db.create_post(1, String::default()).await?;
        assert_eq!(None, post);

        Ok(())
    }
}
