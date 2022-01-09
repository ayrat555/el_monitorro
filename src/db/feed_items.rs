use crate::models::feed_item::FeedItem;
use crate::schema::feed_items;
use crate::sync::FetchedFeedItem;
use chrono::{DateTime, Utc};
use diesel::result::Error;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use html2text::from_read;
use sha2::{Digest, Sha256};

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
    pub content_hash: String,
}

pub fn create(
    conn: &PgConnection,
    feed_id: i64,
    fetched_items: Vec<FetchedFeedItem>,
) -> Result<Vec<FeedItem>, Error> {
    let new_feed_items = fetched_items
        .into_iter()
        .map(|fetched_feed_item| {
            let hash = calculate_content_hash(&fetched_feed_item.link, &fetched_feed_item.title);

            NewFeedItem {
                feed_id,
                title: from_read(fetched_feed_item.title.as_bytes(), 500)
                    .trim()
                    .to_string(),
                description: fetched_feed_item.description,
                link: fetched_feed_item.link,
                author: fetched_feed_item.author,
                guid: fetched_feed_item.guid,
                publication_date: fetched_feed_item.publication_date,
                content_hash: hash,
            }
        })
        .collect::<Vec<NewFeedItem>>();

    diesel::insert_into(feed_items::table)
        .values(new_feed_items)
        .on_conflict((feed_items::feed_id, feed_items::content_hash))
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

pub fn delete_old_feed_items(
    conn: &PgConnection,
    feed_id: i64,
    offset: i64,
) -> Result<usize, Error> {
    let creation_date_result = feed_items::table
        .filter(feed_items::feed_id.eq(feed_id))
        .order((
            feed_items::created_at.desc(),
            feed_items::publication_date.desc(),
        ))
        .offset(offset)
        .limit(1)
        .select(feed_items::created_at)
        .load::<DateTime<Utc>>(conn);

    match creation_date_result {
        Ok(creation_dates) => {
            if !creation_dates.is_empty() {
                let creation_date = creation_dates[0];

                let delete_query = feed_items::table
                    .filter(feed_items::feed_id.eq(feed_id))
                    .filter(feed_items::created_at.le(creation_date));

                diesel::delete(delete_query).execute(conn)
            } else {
                Ok(0)
            }
        }
        Err(error) => Err(error),
    }
}

fn calculate_content_hash(link: &str, title: &str) -> String {
    let mut content_hash: String = "".to_string();
    content_hash.push_str(link);
    content_hash.push_str(title);

    let mut hasher = Sha256::new();
    hasher.update(content_hash.as_bytes());

    let result = hasher.finalize();
    hex::encode(result)
}

pub fn get_latest_item(conn: &PgConnection, feed_id: i64) -> Option<FeedItem> {
    match feed_items::table
        .filter(feed_items::feed_id.eq(feed_id))
        .order(feed_items::created_at.desc())
        .limit(1)
        .get_result::<FeedItem>(conn)
    {
        Ok(record) => Some(record),
        _ => None,
    }
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
        let connection = db::establish_test_connection();

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
                    publication_date,
                },
                FetchedFeedItem {
                    title: "FeedItem2".to_string(),
                    description: Some("Description2".to_string()),
                    link: "Link2".to_string(),
                    author: Some("Author2".to_string()),
                    guid: Some("Guid2".to_string()),
                    publication_date,
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
        let connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "atom".to_string()).unwrap();
            let publication_date = db::current_time();
            let feed_items = vec![FetchedFeedItem {
                title: "FeedItem1".to_string(),
                description: Some("Description1".to_string()),
                link: "Link1".to_string(),
                author: Some("Author1".to_string()),
                guid: Some("Guid1".to_string()),
                publication_date,
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
                publication_date,
            }];

            let new_result = super::create(&connection, feed.id, updated_feed_items).unwrap();

            assert!(new_result.is_empty());

            Ok(())
        });
    }

    #[test]
    fn delete_old_feed_items() {
        let connection = db::establish_test_connection();

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

            super::create(&connection, feed.id, feed_items).unwrap();

            let result = super::delete_old_feed_items(&connection, feed.id, 1).unwrap();
            assert_eq!(result, 2);

            let found_feed_items = super::find(&connection, feed.id).unwrap();
            assert_eq!(found_feed_items.len(), 0);

            Ok(())
        });
    }

    #[test]
    fn delete_old_feed_items_does_not_delete_if_not_enough_items() {
        let connection = db::establish_test_connection();

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

            super::create(&connection, feed.id, feed_items).unwrap();

            let result = super::delete_old_feed_items(&connection, feed.id, 10).unwrap();
            assert_eq!(result, 0);

            let found_feed_items = super::find(&connection, feed.id).unwrap();
            assert_eq!(found_feed_items.len(), 2);

            Ok(())
        });
    }

    #[test]
    fn get_latest_item_returns_none_if_no_items() {
        let connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "rss".to_string()).unwrap();

            assert!(super::get_latest_item(&connection, feed.id).is_none());

            Ok(())
        });
    }
}
