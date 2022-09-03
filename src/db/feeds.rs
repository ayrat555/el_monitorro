use crate::db;
use crate::models::feed::Feed;
use crate::schema::{feeds, telegram_subscriptions};
use chrono::{DateTime, Utc};
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::BigInt;
use diesel::sql_types::Bool;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

const MAX_RETRIES: i32 = 5;

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = feeds)]
struct NewFeed {
    link: String,
    feed_type: String,
}

pub fn create(conn: &mut PgConnection, link: String, feed_type: String) -> Result<Feed, Error> {
    if feed_type != *"atom" && feed_type != *"rss" && feed_type != *"json" {
        unimplemented!()
    }

    let new_feed = NewFeed {
        link: link.trim().to_string(),
        feed_type,
    };

    let feed = diesel::insert_into(feeds::table)
        .values(new_feed)
        .on_conflict(feeds::link)
        .do_update()
        .set(feeds::updated_at.eq(db::current_time()))
        .get_result::<Feed>(conn)?;
    Ok(feed)
}

pub fn set_error(conn: &mut PgConnection, feed: &Feed, error: &str) -> Result<Feed, Error> {
    let next_retry_number = if feed.sync_retries == MAX_RETRIES {
        MAX_RETRIES
    } else {
        feed.sync_retries + 1
    };

    diesel::update(feed)
        .set((
            feeds::error.eq(error),
            feeds::updated_at.eq(db::current_time()),
            feeds::sync_retries.eq(next_retry_number),
        ))
        .get_result::<Feed>(conn)
}

pub fn set_synced_at(
    conn: &mut PgConnection,
    feed: &Feed,
    title: Option<String>,
    description: Option<String>,
) -> Result<Feed, Error> {
    let error: Option<String> = None;

    diesel::update(feed)
        .set((
            feeds::synced_at.eq(db::current_time()),
            feeds::title.eq(title),
            feeds::description.eq(description),
            feeds::updated_at.eq(db::current_time()),
            feeds::error.eq(error),
            feeds::sync_retries.eq(0),
            feeds::sync_skips.eq(0),
        ))
        .get_result::<Feed>(conn)
}

pub fn find(conn: &mut PgConnection, id: i64) -> Option<Feed> {
    match feeds::table.filter(feeds::id.eq(id)).first::<Feed>(conn) {
        Ok(record) => Some(record),
        _ => None,
    }
}

pub fn find_by_link(conn: &mut PgConnection, link: String) -> Option<Feed> {
    match feeds::table
        .filter(feeds::link.eq(link))
        .first::<Feed>(conn)
    {
        Ok(record) => Some(record),
        _ => None,
    }
}

pub fn remove_feed(conn: &mut PgConnection, feed_id: i64) -> Result<usize, Error> {
    let record_query = feeds::table.filter(feeds::id.eq(feed_id));

    diesel::delete(record_query).execute(conn)
}

pub fn find_unsynced_feeds(
    conn: &mut PgConnection,
    last_updated_at: DateTime<Utc>,
    page: i64,
    count: i64,
) -> Result<Vec<i64>, Error> {
    let offset = (page - 1) * count;

    feeds::table
        .inner_join(telegram_subscriptions::table)
        .filter(
            feeds::synced_at
                .lt(last_updated_at)
                .or(feeds::synced_at.is_null()),
        )
        .filter(feeds::sync_retries.eq(0).or(sql::<Bool>(
            "\"feeds\".\"sync_skips\" = pow(2, \"feeds\".\"sync_retries\" - 1)",
        )))
        .select(feeds::id)
        .order(feeds::id)
        .distinct()
        .limit(count)
        .offset(offset)
        .load::<i64>(conn)
}

pub fn increment_and_reset_skips(conn: &mut PgConnection) -> Result<usize, Error> {
    // diesel doesn't support updates with joins
    // https://github.com/diesel-rs/diesel/issues/1478
    let query = "UPDATE \"feeds\" SET \"sync_skips\" = -1\
                 FROM \"telegram_subscriptions\"
                 WHERE \"telegram_subscriptions\".\"feed_id\" = \"feeds\".\"id\" AND\
                 \"feeds\".\"sync_retries\" != 0 AND \"feeds\".\"sync_skips\" = pow(2, \"feeds\".\"sync_retries\" - 1)";

    diesel::sql_query(query).execute(conn)?;

    // diesel doesn't support updates with joins
    // https://github.com/diesel-rs/diesel/issues/1478
    let query = "UPDATE \"feeds\" SET \"sync_skips\" = \"sync_skips\" + 1\
                 FROM \"telegram_subscriptions\"
                 WHERE \"telegram_subscriptions\".\"feed_id\" = \"feeds\".\"id\" AND\
                 \"feeds\".\"sync_retries\" != 0 AND \"feeds\".\"sync_skips\" != pow(2, \"feeds\".\"sync_retries\" - 1)";

    diesel::sql_query(query).execute(conn)
}

pub fn set_content_fields(
    conn: &mut PgConnection,
    feed: &Feed,
    content_fields: Vec<String>,
) -> Result<Feed, Error> {
    diesel::update(feed)
        .set(feeds::content_fields.eq(content_fields))
        .get_result::<Feed>(conn)
}

pub fn load_feed_ids(conn: &mut PgConnection, page: i64, count: i64) -> Result<Vec<i64>, Error> {
    let offset = (page - 1) * count;

    feeds::table
        .select(feeds::id)
        .order(feeds::id)
        .limit(count)
        .offset(offset)
        .load::<i64>(conn)
}

pub fn delete_feeds_without_subscriptions(conn: &mut PgConnection) -> Result<usize, Error> {
    let feeds_without_subscriptions = feeds::table
        .left_join(telegram_subscriptions::table)
        .filter(telegram_subscriptions::feed_id.is_null())
        .limit(2000)
        .select(feeds::id)
        .load::<i64>(conn)?;

    let delete_query = feeds::table.filter(feeds::id.eq_any(feeds_without_subscriptions));

    diesel::delete(delete_query).execute(conn)
}

pub fn count_feeds_with_subscriptions(conn: &mut PgConnection) -> Result<i64, Error> {
    let result = feeds::table
        .inner_join(telegram_subscriptions::table)
        .select(sql::<BigInt>("COUNT (DISTINCT \"feeds\".\"id\")"))
        .first::<i64>(conn);

    if let Err(Error::NotFound) = result {
        return Ok(0);
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::db::telegram;
    use crate::db::telegram::{NewTelegramChat, NewTelegramSubscription};
    use crate::models::feed::Feed;
    use crate::models::telegram_subscription::TelegramSubscription;
    use crate::schema::feeds;
    use chrono::{Duration, Utc};
    use diesel::connection::Connection;
    use diesel::result::Error;
    use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};

    #[test]
    fn create_creates_new_feed() {
        let link = "Link";
        let mut connection = db::establish_test_connection();

        let result = connection.test_transaction::<Feed, Error, _>(|connection| {
            super::create(connection, link.to_string(), "atom".to_string())
        });

        assert_eq!(result.title, None);
        assert_eq!(result.link, link);
        assert_eq!(result.description, None);
    }

    #[test]
    fn create_fails_to_create_feed_without_link() {
        let link = "".to_string();
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let result = super::create(connection, link, "rss".to_string());

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

        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let feed = super::create(connection, link.clone(), "rss".to_string()).unwrap();

            assert_eq!(feed.title, None);
            assert_eq!(feed.link, link);
            assert_eq!(feed.description, None);

            let updated_feed = super::set_synced_at(
                connection,
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
    fn set_synced_deletes_error_from_feed() {
        let link = "Link".to_string();

        let title = "Title".to_string();
        let description = "Description".to_string();

        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let feed = super::create(connection, link.clone(), "rss".to_string()).unwrap();
            let feed_with_error = super::set_error(connection, &feed, "error").unwrap();

            assert_eq!(feed_with_error.error.unwrap(), "error".to_string());
            assert_eq!(feed_with_error.title, None);
            assert_eq!(feed_with_error.link, link);
            assert_eq!(feed_with_error.description, None);

            let updated_feed = super::set_synced_at(
                connection,
                &feed,
                Some(title.clone()),
                Some(description.clone()),
            )
            .unwrap();

            assert!(updated_feed.error.is_none());
            assert_eq!(updated_feed.title, Some(title));
            assert_eq!(updated_feed.link, link);
            assert_eq!(updated_feed.description, Some(description));
            assert!(updated_feed.synced_at.is_some());

            Ok(())
        });
    }

    #[test]
    fn find_finds_feed() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed = super::create(connection, link.clone(), "atom".to_string()).unwrap();

            let found_feed = super::find(connection, feed.id).unwrap();

            assert_eq!(feed.id, found_feed.id);
            assert_eq!(found_feed.title, None);
            assert_eq!(found_feed.link, link);
            assert_eq!(found_feed.description, None);

            Ok(())
        });
    }

    #[test]
    fn find_by_link_finds_feed() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed = super::create(connection, link.clone(), "rss".to_string()).unwrap();

            let found_feed = super::find_by_link(connection, link.clone()).unwrap();

            assert_eq!(feed.id, found_feed.id);
            assert_eq!(found_feed.title, None);
            assert_eq!(found_feed.link, link);
            assert_eq!(found_feed.description, None);

            Ok(())
        });
    }

    #[test]
    fn find_cant_find_feed() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let found_feed = super::find(connection, 42);

            assert_eq!(found_feed, None);

            Ok(())
        });
    }

    #[test]
    fn set_error_sets_error_message_to_feed() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed = super::create(connection, link, "atom".to_string()).unwrap();
            let error = "Error syncing feed";

            let updated_feed = super::set_error(connection, &feed, error).unwrap();

            assert_eq!(updated_feed.error.unwrap(), error);

            Ok(())
        })
    }

    #[test]
    fn set_error_increments_retries() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed = super::create(connection, link, "atom".to_string()).unwrap();
            let error = "Error syncing feed";

            assert_eq!(0, feed.sync_retries);

            let mut updated_feed = super::set_error(connection, &feed, error).unwrap();

            assert_eq!(updated_feed.error.clone().unwrap(), error);
            assert_eq!(1, updated_feed.sync_retries);

            updated_feed = super::set_error(connection, &updated_feed, error).unwrap();

            assert_eq!(updated_feed.error.clone().unwrap(), error);
            assert_eq!(2, updated_feed.sync_retries);

            updated_feed = super::set_error(connection, &updated_feed, error).unwrap();
            assert_eq!(updated_feed.error.clone().unwrap(), error);
            assert_eq!(3, updated_feed.sync_retries);

            Ok(())
        })
    }

    #[test]
    fn set_synced_at_sets_current_time_to_synced_at() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let description = Some("Description".to_string());
            let title = Some("Title".to_string());
            let feed = super::create(connection, link, "rss".to_string()).unwrap();

            assert!(feed.synced_at.is_none());

            let updated_feed = super::set_synced_at(connection, &feed, description, title).unwrap();

            assert!(updated_feed.synced_at.is_some());
            assert!(updated_feed.title.is_some());
            assert!(updated_feed.description.is_some());

            Ok(())
        })
    }

    #[test]
    fn set_synced_at_removes_retries_and_skips() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let description = Some("Description".to_string());
            let title = Some("Title".to_string());
            let feed = super::create(connection, link, "rss".to_string()).unwrap();

            let updated_feed = super::set_error(connection, &feed, "Error").unwrap();
            assert_eq!(updated_feed.sync_retries, 1);

            let updated_feed = super::set_synced_at(connection, &feed, description, title).unwrap();

            assert_eq!(updated_feed.sync_retries, 0);

            assert!(updated_feed.synced_at.is_some());
            assert!(updated_feed.title.is_some());
            assert!(updated_feed.description.is_some());

            Ok(())
        })
    }

    #[test]
    fn find_unsynced_feeds_does_not_fetch_feeds_without_telegram_subscriptions() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed_without_synced_at =
                super::create(connection, link, "rss".to_string()).unwrap();
            assert!(feed_without_synced_at.synced_at.is_none());

            let found_unsynced_feeds =
                super::find_unsynced_feeds(connection, Utc::now(), 1, 1).unwrap();

            assert_eq!(found_unsynced_feeds.len(), 0);

            Ok(())
        })
    }

    #[test]
    fn increment_and_reset_skips_doesnt_update_feeds_without_subscriptions() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed = super::create(connection, link, "rss".to_string()).unwrap();

            super::set_error(connection, &feed, "error").unwrap();

            let result = super::increment_and_reset_skips(connection).unwrap();
            assert_eq!(0, result);

            Ok(())
        })
    }

    #[test]
    fn set_content_fields() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed = super::create(connection, link, "rss".to_string()).unwrap();

            assert_eq!(None, feed.content_fields);

            let updated_feed = super::set_content_fields(
                connection,
                &feed,
                vec!["guid".to_string(), "description".to_string()],
            )
            .unwrap();

            assert_eq!(
                Some(vec!["guid".to_string(), "description".to_string()]),
                updated_feed.content_fields
            );

            Ok(())
        })
    }

    #[test]
    fn increment_skips_updates_feeds_with_subscriptions() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed = super::create(connection, link, "rss".to_string()).unwrap();

            super::set_error(connection, &feed, "error").unwrap();

            create_telegram_subscription(connection, &feed);

            let result1 = super::increment_and_reset_skips(connection).unwrap();
            assert_eq!(1, result1);

            let result_feed1 = super::find(connection, feed.id).unwrap();

            assert_eq!(1, result_feed1.sync_skips);

            let result2 = super::increment_and_reset_skips(connection).unwrap();
            assert_eq!(1, result2);

            let result_feed2 = super::find(connection, feed.id).unwrap();

            assert_eq!(0, result_feed2.sync_skips);

            let result3 = super::increment_and_reset_skips(connection).unwrap();
            assert_eq!(1, result3);

            let result_feed3 = super::find(connection, feed.id).unwrap();

            assert_eq!(1, result_feed3.sync_skips);

            let result4 = super::increment_and_reset_skips(connection).unwrap();
            assert_eq!(1, result4);

            let result_feed4 = super::find(connection, feed.id).unwrap();

            assert_eq!(0, result_feed4.sync_skips);

            Ok(())
        })
    }

    #[test]
    fn increment_skips_resets_max_skips() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let mut feed = super::create(connection, link, "rss".to_string()).unwrap();
            create_telegram_subscription(connection, &feed);

            for _i in 1..5 {
                feed = super::set_error(connection, &feed, "error").unwrap();
            }

            assert_eq!(4, feed.sync_retries);
            assert_eq!(0, feed.sync_skips);

            for i in 1..9 {
                let result = super::increment_and_reset_skips(connection).unwrap();
                assert_eq!(1, result);

                let result_feed = super::find(connection, feed.id).unwrap();

                assert_eq!(i, result_feed.sync_skips);
            }

            feed = super::set_error(connection, &feed, "error").unwrap();
            assert_eq!(5, feed.sync_retries);

            for i in 9..17 {
                let result = super::increment_and_reset_skips(connection).unwrap();
                assert_eq!(1, result);

                let result_feed = super::find(connection, feed.id).unwrap();

                assert_eq!(i, result_feed.sync_skips);
            }

            let result_feed = super::find(connection, feed.id).unwrap();

            assert_eq!(16, result_feed.sync_skips);

            let result = super::increment_and_reset_skips(connection).unwrap();
            assert_eq!(1, result);

            let result_feed = super::find(connection, feed.id).unwrap();
            assert_eq!(0, result_feed.sync_skips);

            Ok(())
        })
    }

    #[test]
    fn find_unsynced_feeds_fetches_unsynced_feeds_without_synced_at() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed_without_synced_at =
                super::create(connection, link, "atom".to_string()).unwrap();
            assert!(feed_without_synced_at.synced_at.is_none());

            create_telegram_subscription(connection, &feed_without_synced_at);

            let found_unsynced_feeds =
                super::find_unsynced_feeds(connection, Utc::now(), 1, 1).unwrap();

            assert_eq!(found_unsynced_feeds.len(), 1);
            assert_eq!(found_unsynced_feeds[0], feed_without_synced_at.id);

            let found_unsynced_feeds_page2 =
                super::find_unsynced_feeds(connection, Utc::now(), 2, 1).unwrap();

            assert_eq!(found_unsynced_feeds_page2.len(), 0);

            Ok(())
        })
    }

    #[test]
    fn find_unsynced_feeds_skips_based_on_retries() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let mut feed = super::create(connection, link, "atom".to_string()).unwrap();

            create_telegram_subscription(connection, &feed);

            let found_unsynced_feeds =
                super::find_unsynced_feeds(connection, Utc::now(), 1, 1).unwrap();

            assert_eq!(found_unsynced_feeds.len(), 1);

            super::set_error(connection, &feed, "error").unwrap();

            for i in 0..17 {
                feed = super::find(connection, feed.id).unwrap();
                let retry = feed.sync_retries;

                let found_unsynced_feeds =
                    super::find_unsynced_feeds(connection, Utc::now(), 1, 1).unwrap();

                if i == 2_i32.pow((retry - 1) as u32) {
                    assert_eq!(found_unsynced_feeds.len(), 1);
                    super::set_error(connection, &feed, "error").unwrap();
                } else {
                    assert_eq!(found_unsynced_feeds.len(), 0);
                }

                super::increment_and_reset_skips(connection).unwrap();
            }

            feed = super::find(connection, feed.id).unwrap();
            assert_eq!(5, feed.sync_retries);
            assert_eq!(0, feed.sync_skips);

            Ok(())
        })
    }

    #[test]
    fn find_unsynced_feeds_fetches_unsynced_feeds_with_expired_synced_at() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let mut feed = super::create(connection, link, "atom".to_string()).unwrap();

            create_telegram_subscription(connection, &feed);

            let expired_synced_at = Utc::now() - Duration::hours(40);

            feed = diesel::update(&feed)
                .set(feeds::synced_at.eq(expired_synced_at))
                .get_result::<Feed>(connection)
                .unwrap();

            assert!(feed.synced_at.is_some());

            let found_unsynced_feeds =
                super::find_unsynced_feeds(connection, Utc::now() - Duration::hours(24), 1, 1)
                    .unwrap();

            assert_eq!(found_unsynced_feeds.len(), 1);
            assert_eq!(found_unsynced_feeds[0], feed.id);

            let found_unsynced_feeds_page2 =
                super::find_unsynced_feeds(connection, Utc::now(), 2, 1).unwrap();

            assert_eq!(found_unsynced_feeds_page2.len(), 0);

            Ok(())
        })
    }

    #[test]
    fn find_unsynced_feeds_doesnt_fetch_synced_feeds() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let mut feed = super::create(connection, link, "rss".to_string()).unwrap();

            create_telegram_subscription(connection, &feed);

            let expired_synced_at = Utc::now() - Duration::hours(10);

            feed = diesel::update(&feed)
                .set(feeds::synced_at.eq(expired_synced_at))
                .get_result::<Feed>(connection)
                .unwrap();

            assert!(feed.synced_at.is_some());

            let found_unsynced_feeds =
                super::find_unsynced_feeds(connection, Utc::now() - Duration::hours(24), 1, 1)
                    .unwrap();

            assert_eq!(found_unsynced_feeds.len(), 0);

            Ok(())
        })
    }

    #[test]
    fn delete_feeds_without_subscriptions() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed = super::create(connection, link, "rss".to_string()).unwrap();

            let deleted_feeds_count =
                super::delete_feeds_without_subscriptions(connection).unwrap();

            assert!(super::find(connection, feed.id).is_none());
            assert_eq!(deleted_feeds_count, 1);

            Ok(())
        })
    }

    #[test]
    fn delete_feeds_without_subscriptions_doesnt_remove_feeds_with_subscriptions() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<_, Error, _>(|connection| {
            let link = "Link".to_string();
            let feed = super::create(connection, link, "rss".to_string()).unwrap();

            create_telegram_subscription(connection, &feed);

            let deleted_feeds_count =
                super::delete_feeds_without_subscriptions(connection).unwrap();

            assert!(super::find(connection, feed.id).is_some());
            assert_eq!(deleted_feeds_count, 0);

            Ok(())
        })
    }

    fn create_telegram_subscription(
        connection: &mut PgConnection,
        feed: &Feed,
    ) -> TelegramSubscription {
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
            title: None,
        };
        let chat = telegram::create_chat(connection, new_chat).unwrap();

        let new_subscription = NewTelegramSubscription {
            feed_id: feed.id,
            chat_id: chat.id,
        };

        telegram::create_subscription(connection, new_subscription).unwrap()
    }
}
