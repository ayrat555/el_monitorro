use crate::models::feed_item::FeedItem;
use crate::schema::feed_items;
use crate::sync::FetchedFeedItem;
use chrono::{DateTime, Utc};
use diesel::result::Error;
use diesel::{sql_query, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

#[derive(Insertable, AsChangeset)]
#[table_name = "feed_items"]
pub struct NewFeedItem {
    pub feed_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub link: String,
    pub author: Option<String>,
    pub guid: Option<String>,
    pub publication_date: DateTime<Utc>,
}

pub fn create(
    conn: &PgConnection,
    feed_id: i64,
    fetched_items: Vec<FetchedFeedItem>,
) -> Result<Vec<FeedItem>, Error> {
    let new_feed_items = fetched_items
        .into_iter()
        .map(|fetched_feed_item| NewFeedItem {
            feed_id: feed_id,
            title: fetched_feed_item.title,
            description: fetched_feed_item.description,
            link: fetched_feed_item.link,
            author: fetched_feed_item.author,
            guid: fetched_feed_item.guid,
            publication_date: fetched_feed_item.publication_date,
        })
        .collect::<Vec<NewFeedItem>>();

    diesel::insert_into(feed_items::table)
        .values(new_feed_items)
        .on_conflict((feed_items::feed_id, feed_items::title, feed_items::link))
        .do_nothing()
        .get_results(conn)
}

pub fn find(conn: &PgConnection, feed_id: i64) -> Option<Vec<FeedItem>> {
    match feed_items::table
        .filter(feed_items::feed_id.eq(feed_id))
        .get_results::<FeedItem>(conn)
    {
        Ok(record) => Some(record),
        _ => None,
    }
}

pub fn remove_old_feed_items(conn: &PgConnection, limit: i32) -> Result<usize, Error> {
    let query = format!(
         "delete  from feed_items using feed_items as items1 left join  (
           select feed_id, title, link
           from feed_items
           order by publication_date desc
           limit {}
         ) as items2 on items1.feed_id = items2.feed_id and items1.title = items2.title and items1.link = items2.link
         where items2.feed_id is null", limit);

    sql_query(query).execute(conn)
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::db::feeds;
    use crate::sync::FetchedFeedItem;
    use diesel::connection::Connection;
    use diesel::result::Error;

    #[test]
    fn create_creates_new_feed_items() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "rss".to_string()).unwrap();
            let publication_date = db::current_time();
            let feed_items = vec![
                FetchedFeedItem {
                    title: "FeedItem1".to_string(),
                    description: Some("Description1".to_string()),
                    link: "Link1".to_string(),
                    author: Some("Author1".to_string()),
                    guid: Some("Guid1".to_string()),
                    publication_date: publication_date,
                },
                FetchedFeedItem {
                    title: "FeedItem2".to_string(),
                    description: Some("Description2".to_string()),
                    link: "Link2".to_string(),
                    author: Some("Author2".to_string()),
                    guid: Some("Guid2".to_string()),
                    publication_date: publication_date,
                },
            ];

            let result = super::create(&connection, feed.id, feed_items.clone()).unwrap();
            let inserted_first_item = result
                .iter()
                .find(|item| item.guid == Some("Guid1".to_string()))
                .unwrap();

            assert_eq!(inserted_first_item.feed_id, feed.id);
            assert_eq!(inserted_first_item.title, feed_items[0].title);
            assert_eq!(inserted_first_item.description, feed_items[0].description);
            assert_eq!(inserted_first_item.link, feed_items[0].link);

            let inserted_second_item = result
                .into_iter()
                .find(|item| item.guid == Some("Guid2".to_string()))
                .unwrap();

            assert_eq!(inserted_second_item.feed_id, feed.id);
            assert_eq!(inserted_second_item.title, feed_items[1].title);
            assert_eq!(inserted_second_item.description, feed_items[1].description);
            assert_eq!(inserted_second_item.link, feed_items[1].link);

            Ok(())
        });
    }

    #[test]
    fn create_does_not_update_existing_feed_items() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "atom".to_string()).unwrap();
            let publication_date = db::current_time();
            let feed_items = vec![FetchedFeedItem {
                title: "FeedItem1".to_string(),
                description: Some("Description1".to_string()),
                link: "Link1".to_string(),
                author: Some("Author1".to_string()),
                guid: Some("Guid1".to_string()),
                publication_date: publication_date,
            }];

            let old_result = super::create(&connection, feed.id, feed_items.clone()).unwrap();
            let old_item = old_result
                .iter()
                .find(|item| item.guid == Some("Guid1".to_string()))
                .unwrap();

            assert_eq!(old_item.feed_id, feed.id);
            assert_eq!(old_item.title, feed_items[0].title);
            assert_eq!(old_item.description, feed_items[0].description);
            assert_eq!(old_item.link, feed_items[0].link);

            let updated_feed_items = vec![FetchedFeedItem {
                title: "FeedItem1".to_string(),
                description: Some("Description1".to_string()),
                link: "Link1".to_string(),
                author: Some("Author2".to_string()),
                guid: Some("Guid2".to_string()),
                publication_date: publication_date,
            }];

            let new_result =
                super::create(&connection, feed.id, updated_feed_items.clone()).unwrap();

            assert!(new_result.is_empty());

            Ok(())
        });
    }

    #[test]
    fn remove_old_feed_items() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "rss".to_string()).unwrap();
            let feed_items = vec![
                FetchedFeedItem {
                    title: "FeedItem1".to_string(),
                    description: Some("Description1".to_string()),
                    link: "Link1".to_string(),
                    author: Some("Author1".to_string()),
                    guid: Some("Guid1".to_string()),
                    publication_date: db::current_time(),
                },
                FetchedFeedItem {
                    title: "FeedItem2".to_string(),
                    description: Some("Description2".to_string()),
                    link: "Link2".to_string(),
                    author: Some("Author2".to_string()),
                    guid: Some("Guid2".to_string()),
                    publication_date: db::current_time(),
                },
            ];

            super::create(&connection, feed.id, feed_items.clone()).unwrap();

            let result = super::remove_old_feed_items(&connection, 1).unwrap();

            assert_eq!(result, 1);

            Ok(())
        });
    }
}
