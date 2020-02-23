use crate::db;
use crate::models::feed_item::FeedItem;
use crate::schema::feed_items;
use chrono::prelude::{DateTime, Utc};
use diesel::pg::upsert::excluded;
use diesel::result::Error;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};

#[derive(Insertable, AsChangeset)]
#[table_name = "feed_items"]
pub struct NewFeedItem<'a> {
    pub feed_id: &'a i32,
    pub title: &'a str,
    pub description: &'a str,
    pub link: &'a str,
    pub author: &'a str,
    pub guid: &'a str,
    pub categories: &'a Vec<String>,
    pub publication_date: &'a DateTime<Utc>,
}

pub fn create(
    conn: &PgConnection,
    new_feed_items: &Vec<NewFeedItem>,
) -> Result<Vec<FeedItem>, Error> {
    diesel::insert_into(feed_items::table)
        .values(new_feed_items)
        .on_conflict((feed_items::feed_id, feed_items::guid))
        .do_update()
        .set((
            feed_items::created_at.eq(excluded(feed_items::created_at)),
            feed_items::updated_at.eq(db::current_time()),
        ))
        .get_results(conn)
}

#[cfg(test)]
mod tests {
    use super::NewFeedItem;
    use crate::db;
    use crate::db::feeds;
    use diesel::connection::Connection;
    use diesel::result::Error;

    #[test]
    fn it_creates_new_feed_items() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let feed = feeds::create(&connection, "Feed Title", "Link", "Description").unwrap();

            let categories = vec!["1".to_string(), "2".to_string()];
            let publication_date = db::current_time();

            let feed_items = &vec![
                NewFeedItem {
                    feed_id: &feed.id,
                    title: "FeedItem1",
                    description: "Description1",
                    link: "Link1",
                    author: "Authot1",
                    guid: "Guid1",
                    categories: &categories,
                    publication_date: &publication_date,
                },
                NewFeedItem {
                    feed_id: &feed.id,
                    title: "FeedItem2",
                    description: "Description2",
                    link: "Link2",
                    author: "Authot2",
                    guid: "Guid2",
                    categories: &categories,
                    publication_date: &publication_date,
                },
            ];
            let result = super::create(&connection, feed_items).unwrap();

            let inserted_first_item = result
                .iter()
                .find(|item| item.guid == feed_items[0].guid)
                .unwrap();

            assert_eq!(&inserted_first_item.feed_id, feed_items[0].feed_id);
            assert_eq!(inserted_first_item.title, feed_items[0].title);
            assert_eq!(inserted_first_item.description, feed_items[0].description);
            assert_eq!(inserted_first_item.link, feed_items[0].link);
            assert_eq!(inserted_first_item.categories, categories);

            let inserted_second_item = result
                .into_iter()
                .find(|item| item.guid == feed_items[1].guid)
                .unwrap();

            assert_eq!(&inserted_second_item.feed_id, feed_items[1].feed_id);
            assert_eq!(inserted_second_item.title, feed_items[1].title);
            assert_eq!(inserted_second_item.description, feed_items[1].description);
            assert_eq!(inserted_second_item.link, feed_items[1].link);
            assert_eq!(inserted_second_item.categories, categories);

            Ok(())
        });
    }
}
