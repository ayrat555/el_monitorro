use crate::db;
use crate::models::feed::Feed;
use crate::schema::feeds;
use chrono::offset::Utc;
use chrono::DateTime;
use diesel::result::Error;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

#[derive(Insertable, AsChangeset)]
#[table_name = "feeds"]
struct NewFeed {
    link: String,
}

pub fn create(conn: &PgConnection, link: String) -> Result<Feed, Error> {
    let new_feed = &NewFeed { link: link };

    diesel::insert_into(feeds::table)
        .values(new_feed)
        .on_conflict(feeds::link)
        .do_update()
        .set(feeds::updated_at.eq(db::current_time()))
        .get_result::<Feed>(conn)
}

pub fn set_error(conn: &PgConnection, feed: &Feed, error: &str) -> Result<Feed, Error> {
    diesel::update(feed)
        .set((
            feeds::error.eq(error),
            feeds::synced_at.eq(db::current_time()),
            feeds::updated_at.eq(db::current_time()),
        ))
        .get_result::<Feed>(conn)
}

pub fn set_synced_at(
    conn: &PgConnection,
    feed: &Feed,
    title: Option<String>,
    description: Option<String>,
) -> Result<Feed, Error> {
    diesel::update(feed)
        .set((
            feeds::synced_at.eq(db::current_time()),
            feeds::title.eq(title),
            feeds::description.eq(description),
            feeds::updated_at.eq(db::current_time()),
        ))
        .get_result::<Feed>(conn)
}

pub fn find(conn: &PgConnection, id: i64) -> Option<Feed> {
    match feeds::table.filter(feeds::id.eq(id)).first::<Feed>(conn) {
        Ok(record) => Some(record),
        _ => None,
    }
}

pub fn find_unsynced_feeds(
    conn: &PgConnection,
    last_updated_at: DateTime<Utc>,
    page: i64,
    count: i64,
) -> Result<Vec<Feed>, Error> {
    let offset = (page - 1) * count;

    feeds::table
        .filter(feeds::synced_at.lt(last_updated_at))
        .or_filter(feeds::synced_at.is_null())
        .order(feeds::id)
        .limit(count)
        .offset(offset)
        .get_results(conn)
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::models::feed::Feed;
    use crate::schema::feeds;
    use chrono::offset::Utc;
    use chrono::Duration;
    use diesel::connection::Connection;
    use diesel::result::Error;
    use diesel::{ExpressionMethods, RunQueryDsl};

    #[test]
    fn create_creates_new_feed() {
        let link = "Link";
        let connection = db::establish_connection();

        let result = connection
            .test_transaction::<Feed, Error, _>(|| super::create(&connection, link.to_string()));

        assert_eq!(result.title, None);
        assert_eq!(result.link, link);
        assert_eq!(result.description, None);
    }

    #[test]
    fn create_fails_to_create_feed_without_link() {
        let link = "".to_string();
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let result = super::create(&connection, link);

            match result.err().unwrap() {
                Error::DatabaseError(_, error_info) => assert_eq!(
                    "new row for relation \"feeds\" violates check constraint \"feed_link_size\"",
                    error_info.message()
                ),
                _ => panic!("Error doesn't match"),
            };

            Ok(())
        });
    }

    #[test]
    fn set_synced_at_sets_description_and_title_to_feed() {
        let link = "Link".to_string();

        let title = "Title".to_string();
        let description = "Description".to_string();

        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let feed = super::create(&connection, link.clone()).unwrap();

            assert_eq!(feed.title, None);
            assert_eq!(feed.link, link);
            assert_eq!(feed.description, None);

            let updated_feed = super::set_synced_at(
                &connection,
                &feed,
                Some(title.clone()),
                Some(description.clone()),
            )
            .unwrap();

            assert_eq!(updated_feed.title, Some(title));
            assert_eq!(updated_feed.link, link);
            assert_eq!(updated_feed.description, Some(description));
            assert!(updated_feed.synced_at.is_some());

            Ok(())
        });
    }

    #[test]
    fn find_finds_feed() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let link = "Link".to_string();
            let feed = super::create(&connection, link.clone()).unwrap();

            let found_feed = super::find(&connection, feed.id).unwrap();

            assert_eq!(feed.id, found_feed.id);
            assert_eq!(found_feed.title, None);
            assert_eq!(found_feed.link, link);
            assert_eq!(found_feed.description, None);

            Ok(())
        });
    }

    #[test]
    fn find_cant_find_feed() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let found_feed = super::find(&connection, 42);

            assert_eq!(found_feed, None);

            Ok(())
        });
    }

    #[test]
    fn set_error_sets_error_message_to_feed() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let link = "Link".to_string();
            let feed = super::create(&connection, link).unwrap();
            let error = "Error syncing feed";

            let updated_feed = super::set_error(&connection, &feed, error).unwrap();

            assert_eq!(updated_feed.error.unwrap(), error);
            assert!(updated_feed.synced_at.is_some());

            Ok(())
        })
    }

    #[test]
    fn set_synced_at_sets_current_time_to_synced_at() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let link = "Link".to_string();
            let description = Some("Description".to_string());
            let title = Some("Title".to_string());
            let feed = super::create(&connection, link).unwrap();

            assert!(feed.synced_at.is_none());

            let updated_feed =
                super::set_synced_at(&connection, &feed, description, title).unwrap();

            assert!(updated_feed.synced_at.is_some());
            assert!(updated_feed.title.is_some());
            assert!(updated_feed.description.is_some());

            Ok(())
        })
    }

    #[test]
    fn find_unsynced_feeds_fetches_unsynced_feeds_without_synced_at() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let link = "Link".to_string();
            let feed_without_synced_at = super::create(&connection, link).unwrap();
            assert!(feed_without_synced_at.synced_at.is_none());

            let found_unsynced_feeds =
                super::find_unsynced_feeds(&connection, Utc::now(), 1, 1).unwrap();

            assert_eq!(found_unsynced_feeds.len(), 1);
            assert_eq!(found_unsynced_feeds[0].id, feed_without_synced_at.id);

            let found_unsynced_feeds_page2 =
                super::find_unsynced_feeds(&connection, Utc::now(), 2, 1).unwrap();

            assert_eq!(found_unsynced_feeds_page2.len(), 0);

            Ok(())
        })
    }

    #[test]
    fn it_fetches_unsynced_feeds_with_expired_synced_at() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let link = "Link".to_string();
            let mut feed = super::create(&connection, link).unwrap();

            let expired_synced_at = Utc::now() - Duration::hours(40);

            feed = diesel::update(&feed)
                .set(feeds::synced_at.eq(expired_synced_at))
                .get_result::<Feed>(&connection)
                .unwrap();

            assert!(feed.synced_at.is_some());

            let found_unsynced_feeds =
                super::find_unsynced_feeds(&connection, Utc::now() - Duration::hours(24), 1, 1)
                    .unwrap();

            assert_eq!(found_unsynced_feeds.len(), 1);
            assert_eq!(found_unsynced_feeds[0].id, feed.id);

            let found_unsynced_feeds_page2 =
                super::find_unsynced_feeds(&connection, Utc::now(), 2, 1).unwrap();

            assert_eq!(found_unsynced_feeds_page2.len(), 0);

            Ok(())
        })
    }

    #[test]
    fn find_unsynced_feeds_doesnt_fetch_synced_feeds() {
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let link = "Link".to_string();
            let mut feed = super::create(&connection, link).unwrap();

            let expired_synced_at = Utc::now() - Duration::hours(10);

            feed = diesel::update(&feed)
                .set(feeds::synced_at.eq(expired_synced_at))
                .get_result::<Feed>(&connection)
                .unwrap();

            assert!(feed.synced_at.is_some());

            let found_unsynced_feeds =
                super::find_unsynced_feeds(&connection, Utc::now() - Duration::hours(24), 1, 1)
                    .unwrap();

            assert_eq!(found_unsynced_feeds.len(), 0);

            Ok(())
        })
    }
}
