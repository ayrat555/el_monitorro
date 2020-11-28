use crate::db;
use crate::models::feed::Feed;
use crate::models::feed_item::FeedItem;
use crate::models::telegram_chat::TelegramChat;
use crate::models::telegram_subscription::TelegramSubscription;
use crate::schema::feed_items;
use crate::schema::{feeds, telegram_chats, telegram_subscriptions};

use chrono::{DateTime, Duration, Utc};
use diesel::dsl::*;
use diesel::pg::upsert::excluded;
use diesel::result::Error;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

#[derive(Insertable, Clone, Debug)]
#[table_name = "telegram_chats"]
pub struct NewTelegramChat {
    pub id: i64,
    pub kind: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Insertable, Clone, Copy)]
#[table_name = "telegram_subscriptions"]
pub struct NewTelegramSubscription {
    pub chat_id: i64,
    pub feed_id: i64,
}

pub fn create_chat(conn: &PgConnection, new_chat: NewTelegramChat) -> Result<TelegramChat, Error> {
    diesel::insert_into(telegram_chats::table)
        .values(new_chat)
        .on_conflict(telegram_chats::id)
        .do_update()
        .set((
            telegram_chats::updated_at.eq(db::current_time()),
            telegram_chats::kind.eq(excluded(telegram_chats::kind)),
            telegram_chats::username.eq(excluded(telegram_chats::username)),
            telegram_chats::first_name.eq(excluded(telegram_chats::first_name)),
            telegram_chats::last_name.eq(excluded(telegram_chats::last_name)),
            telegram_chats::title.eq(excluded(telegram_chats::title)),
        ))
        .get_result::<TelegramChat>(conn)
}

pub fn find_chat(conn: &PgConnection, chat_id: i64) -> Option<TelegramChat> {
    match telegram_chats::table
        .filter(telegram_chats::id.eq(chat_id))
        .first::<TelegramChat>(conn)
    {
        Ok(record) => Some(record),
        _ => None,
    }
}

pub fn set_utc_offset_minutes(
    conn: &PgConnection,
    chat: &TelegramChat,
    offset: i32,
) -> Result<TelegramChat, Error> {
    diesel::update(chat)
        .set(telegram_chats::utc_offset_minutes.eq(offset))
        .get_result::<TelegramChat>(conn)
}

pub fn set_global_template(
    conn: &PgConnection,
    chat: &TelegramChat,
    template: String,
) -> Result<TelegramChat, Error> {
    diesel::update(chat)
        .set(telegram_chats::template.eq(template))
        .get_result::<TelegramChat>(conn)
}

pub fn set_template(
    conn: &PgConnection,
    chat: &TelegramSubscription,
    template: String,
) -> Result<TelegramSubscription, Error> {
    diesel::update(chat)
        .set(telegram_subscriptions::template.eq(template))
        .get_result::<TelegramSubscription>(conn)
}

pub fn create_subscription(
    conn: &PgConnection,
    subscription: NewTelegramSubscription,
) -> Result<TelegramSubscription, Error> {
    diesel::insert_into(telegram_subscriptions::table)
        .values(subscription)
        .get_result::<TelegramSubscription>(conn)
}

pub fn find_subscription(
    conn: &PgConnection,
    subscription: NewTelegramSubscription,
) -> Option<TelegramSubscription> {
    match telegram_subscriptions::table
        .filter(telegram_subscriptions::chat_id.eq(subscription.chat_id))
        .filter(telegram_subscriptions::feed_id.eq(subscription.feed_id))
        .first::<TelegramSubscription>(conn)
    {
        Ok(record) => Some(record),
        _ => None,
    }
}

pub fn remove_subscription(
    conn: &PgConnection,
    subscription: NewTelegramSubscription,
) -> Result<usize, Error> {
    let record_query = telegram_subscriptions::table
        .filter(telegram_subscriptions::chat_id.eq(subscription.chat_id))
        .filter(telegram_subscriptions::feed_id.eq(subscription.feed_id));

    diesel::delete(record_query).execute(conn)
}

pub fn remove_chat(conn: &PgConnection, chat_id: i64) -> Result<usize, Error> {
    let record_query = telegram_chats::table.filter(telegram_chats::id.eq(chat_id));

    diesel::delete(record_query).execute(conn)
}

pub fn count_subscriptions_for_chat(conn: &PgConnection, chat_id: i64) -> i64 {
    telegram_subscriptions::table
        .filter(telegram_subscriptions::chat_id.eq(chat_id))
        .count()
        .get_result::<i64>(conn)
        .unwrap()
}

pub fn find_subscriptions_for_chat(
    conn: &PgConnection,
    chat_id: i64,
) -> Result<Vec<TelegramSubscription>, Error> {
    telegram_subscriptions::table
        .filter(telegram_subscriptions::chat_id.eq(chat_id))
        .get_results::<TelegramSubscription>(conn)
}

pub fn find_feeds_by_chat_id(conn: &PgConnection, chat_id: i64) -> Result<Vec<Feed>, Error> {
    let feed_ids = telegram_subscriptions::table
        .filter(telegram_subscriptions::chat_id.eq(chat_id))
        .select(telegram_subscriptions::feed_id);

    feeds::table
        .filter(feeds::id.eq(any(feed_ids)))
        .get_results::<Feed>(conn)
}

pub fn find_chats_by_feed_id(
    conn: &PgConnection,
    feed_id: i64,
) -> Result<Vec<TelegramChat>, Error> {
    let chat_ids = telegram_subscriptions::table
        .filter(telegram_subscriptions::feed_id.eq(feed_id))
        .select(telegram_subscriptions::chat_id);

    telegram_chats::table
        .filter(telegram_chats::id.eq(any(chat_ids)))
        .get_results::<TelegramChat>(conn)
}

pub fn fetch_subscriptions(
    conn: &PgConnection,
    page: i64,
    count: i64,
) -> Result<Vec<TelegramSubscription>, Error> {
    let offset = (page - 1) * count;

    telegram_subscriptions::table
        .order(telegram_subscriptions::chat_id)
        .limit(count)
        .offset(offset)
        .get_results(conn)
}

pub fn fetch_chats_with_subscriptions(
    conn: &PgConnection,
    page: i64,
    count: i64,
) -> Result<Vec<i64>, Error> {
    let offset = (page - 1) * count;

    telegram_chats::table
        .inner_join(telegram_subscriptions::table)
        .order(telegram_chats::id)
        .select(telegram_chats::id)
        .distinct()
        .limit(count)
        .offset(offset)
        .get_results(conn)
}

pub fn find_undelivered_feed_items(
    conn: &PgConnection,
    subscription: &TelegramSubscription,
) -> Result<Vec<FeedItem>, Error> {
    let last_delivered_at = match subscription.last_delivered_at {
        Some(value) => value,
        None => db::current_time() - Duration::days(365),
    };

    feed_items::table
        .filter(feed_items::publication_date.gt(last_delivered_at))
        .filter(feed_items::feed_id.eq(subscription.feed_id))
        .order(feed_items::publication_date.desc())
        .limit(10)
        .get_results(conn)
}

pub fn count_undelivered_feed_items(
    conn: &PgConnection,
    subscription: &TelegramSubscription,
) -> i64 {
    let last_delivered_at = match subscription.last_delivered_at {
        Some(value) => value,
        None => db::current_time() - Duration::days(1),
    };

    feed_items::table
        .filter(feed_items::publication_date.gt(last_delivered_at))
        .filter(feed_items::feed_id.eq(subscription.feed_id))
        .count()
        .get_result::<i64>(conn)
        .unwrap()
}

pub fn set_subscription_last_delivered_at(
    conn: &PgConnection,
    subscription: &TelegramSubscription,
    last_delivered_at: DateTime<Utc>,
) -> Result<TelegramSubscription, Error> {
    diesel::update(subscription)
        .set(telegram_subscriptions::last_delivered_at.eq(last_delivered_at))
        .get_result::<TelegramSubscription>(conn)
}

#[cfg(test)]
mod tests {
    use super::NewTelegramChat;
    use super::NewTelegramSubscription;
    use crate::db;
    use crate::db::feeds;
    use crate::models::telegram_chat::TelegramChat;
    use diesel::connection::Connection;
    use diesel::result::Error;

    #[test]
    fn create_chat_creates_new_telegram_chat() {
        let new_chat = build_new_chat();
        let connection = db::establish_connection();

        let result = connection.test_transaction::<TelegramChat, Error, _>(|| {
            super::create_chat(&connection, new_chat.clone())
        });

        assert_eq!(result.id, new_chat.id);
        assert_eq!(result.kind, new_chat.kind);
        assert_eq!(result.username, new_chat.username);
        assert_eq!(result.first_name, new_chat.first_name);
        assert_eq!(result.last_name, new_chat.last_name);
    }

    #[test]
    fn it_updates_telegram_chat() {
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
            title: None,
        };
        let updated_chat = NewTelegramChat {
            id: 42,
            kind: "public1".to_string(),
            username: Some("Username1".to_string()),
            first_name: Some("First1".to_string()),
            last_name: Some("Last1".to_string()),
            title: None,
        };
        let connection = db::establish_connection();

        let new_result = connection.test_transaction::<TelegramChat, Error, _>(|| {
            let result = super::create_chat(&connection, new_chat.clone()).unwrap();

            assert_eq!(result.id, new_chat.id);
            assert_eq!(result.kind, new_chat.kind);
            assert_eq!(result.username, new_chat.username);
            assert_eq!(result.first_name, new_chat.first_name);
            assert_eq!(result.last_name, new_chat.last_name);

            super::create_chat(&connection, updated_chat.clone())
        });

        assert_eq!(new_result.id, updated_chat.id);
        assert_eq!(new_result.kind, updated_chat.kind);
        assert_eq!(new_result.username, updated_chat.username);
        assert_eq!(new_result.first_name, updated_chat.first_name);
        assert_eq!(new_result.last_name, updated_chat.last_name);
    }

    #[test]
    fn create_subscription_creates_new_subscription() {
        let connection = db::establish_connection();

        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
            title: None,
        };

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "rss".to_string()).unwrap();
            let chat = super::create_chat(&connection, new_chat).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat.id,
            };

            let new_subscription =
                super::create_subscription(&connection, new_subscription).unwrap();

            assert_eq!(new_subscription.feed_id, feed.id);
            assert_eq!(new_subscription.chat_id, chat.id);

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_new_subscription_if_it_already_exists() {
        let connection = db::establish_connection();

        let new_chat = build_new_chat();

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "atom".to_string()).unwrap();
            let chat = super::create_chat(&connection, new_chat).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat.id,
            };

            let new_subscription =
                super::create_subscription(&connection, new_subscription).unwrap();

            assert_eq!(new_subscription.feed_id, feed.id);
            assert_eq!(new_subscription.chat_id, chat.id);

            let result = super::create_subscription(
                &connection,
                NewTelegramSubscription {
                    feed_id: feed.id,
                    chat_id: chat.id,
                },
            );

            match result.err().unwrap() {
                Error::DatabaseError(_, error_info) => assert_eq!(
                    "duplicate key value violates unique constraint \"telegram_subscriptions_pkey\"",
                    error_info.message()
                ),
                _ => panic!("Error doesn't match"),
            };

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_new_subscription_if_it_chat_does_not_exist() {
        let connection = db::establish_connection();

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "atom".to_string()).unwrap();

            let result = super::create_subscription(
                &connection,
                NewTelegramSubscription {
                    feed_id: feed.id,
                    chat_id: 42,
                },
            );

            match result.err().unwrap() {
                Error::DatabaseError(_, error_info) => assert_eq!(
                    "insert or update on table \"telegram_subscriptions\" violates foreign key constraint \"telegram_subscriptions_chat_id_fkey\"",
                    error_info.message()
                ),
                _ => panic!("Error doesn't match"),
            };

            Ok(())
        });
    }

    #[test]
    fn find_subscription_finds_subscription() {
        let connection = db::establish_connection();

        let new_chat = build_new_chat_with_id(999);

        connection.test_transaction::<(), Error, _>(|| {
            let feed =
                feeds::create(&connection, "Link11111".to_string(), "rss".to_string()).unwrap();
            let chat = super::create_chat(&connection, new_chat).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat.id,
            };

            let new_subscription =
                super::create_subscription(&connection, new_subscription).unwrap();

            assert_eq!(new_subscription.feed_id, feed.id);
            assert_eq!(new_subscription.chat_id, chat.id);

            let result = super::find_subscription(
                &connection,
                NewTelegramSubscription {
                    feed_id: feed.id,
                    chat_id: chat.id,
                },
            )
            .unwrap();

            assert_eq!(result.feed_id, feed.id);
            assert_eq!(result.chat_id, chat.id);

            Ok(())
        });
    }

    #[test]
    fn find_subscription_fails_to_find_a_subscription() {
        let connection = db::establish_connection();

        connection.test_transaction::<(), Error, _>(|| {
            let result = super::find_subscription(
                &connection,
                NewTelegramSubscription {
                    feed_id: 42,
                    chat_id: 42,
                },
            );

            assert!(result.is_none());

            Ok(())
        });
    }

    #[test]
    fn fetch_chats_with_subscriptions_fetches_chat_with_subscription() {
        let connection = db::establish_connection();

        connection.test_transaction::<(), Error, _>(|| {
            let new_chat = build_new_chat();
            let chat = super::create_chat(&connection, new_chat).unwrap();
            let feed = feeds::create(&connection, "Link99".to_string(), "rss".to_string()).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat.id,
            };

            super::create_subscription(&connection, new_subscription.clone()).unwrap();

            let result = super::fetch_chats_with_subscriptions(&connection, 1, 1).unwrap();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0], chat.id);

            Ok(())
        });
    }

    #[test]
    fn fetch_chats_with_subscriptions_does_not_fetch_chat_without_subscription() {
        let connection = db::establish_connection();

        connection.test_transaction::<(), Error, _>(|| {
            let new_chat = build_new_chat();
            super::create_chat(&connection, new_chat).unwrap();

            let result = super::fetch_chats_with_subscriptions(&connection, 1, 1).unwrap();

            assert_eq!(result.len(), 0);

            Ok(())
        });
    }

    #[test]
    fn fetch_chats_with_subscriptions_paginates_result() {
        let connection = db::establish_connection();

        connection.test_transaction::<(), Error, _>(|| {
            let feed =
                feeds::create(&connection, "Link98".to_string(), "atom".to_string()).unwrap();
            let chat1 = super::create_chat(&connection, build_new_chat_with_id(10)).unwrap();

            let new_subscription1 = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat1.id,
            };

            super::create_subscription(&connection, new_subscription1).unwrap();

            let chat2 = super::create_chat(&connection, build_new_chat_with_id(20)).unwrap();

            let new_subscription2 = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat2.id,
            };

            super::create_subscription(&connection, new_subscription2).unwrap();

            let result1 = super::fetch_chats_with_subscriptions(&connection, 1, 1).unwrap();

            assert_eq!(result1[0], chat1.id);

            let result2 = super::fetch_chats_with_subscriptions(&connection, 2, 1).unwrap();

            assert_eq!(result2[0], chat2.id);

            Ok(())
        });
    }

    #[test]
    fn fetch_chats_with_subscriptions_does_no_return_duplicates() {
        let connection = db::establish_connection();

        connection.test_transaction::<(), Error, _>(|| {
            let feed1 =
                feeds::create(&connection, "Link97".to_string(), "atom".to_string()).unwrap();
            let feed2 =
                feeds::create(&connection, "Link96".to_string(), "atom".to_string()).unwrap();
            let chat = super::create_chat(&connection, build_new_chat()).unwrap();

            let new_subscription1 = NewTelegramSubscription {
                feed_id: feed1.id,
                chat_id: chat.id,
            };

            super::create_subscription(&connection, new_subscription1).unwrap();

            let new_subscription2 = NewTelegramSubscription {
                feed_id: feed2.id,
                chat_id: chat.id,
            };

            super::create_subscription(&connection, new_subscription2).unwrap();

            let result1 = super::fetch_chats_with_subscriptions(&connection, 1, 1).unwrap();

            assert_eq!(result1[0], chat.id);

            let result2 = super::fetch_chats_with_subscriptions(&connection, 2, 1).unwrap();

            assert_eq!(result2.len(), 0);

            Ok(())
        });
    }

    #[test]
    fn count_subscriptions_for_chat_counts_the_number_of_subscriptions() {
        let connection = db::establish_connection();

        let new_chat = build_new_chat();

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "atom".to_string()).unwrap();
            let chat = super::create_chat(&connection, new_chat).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat.id,
            };

            super::create_subscription(&connection, new_subscription).unwrap();

            let result = super::count_subscriptions_for_chat(&connection, chat.id);

            assert_eq!(result, 1);
            assert_eq!(super::count_subscriptions_for_chat(&connection, 99), 0);

            Ok(())
        });
    }

    #[test]
    fn find_subscriptions_for_chat_finds_subscriptions_for_chat() {
        let connection = db::establish_connection();

        let new_chat = build_new_chat();

        connection.test_transaction::<(), Error, _>(|| {
            let feed1 =
                feeds::create(&connection, "Link80".to_string(), "atom".to_string()).unwrap();
            let feed2 =
                feeds::create(&connection, "Link79".to_string(), "atom".to_string()).unwrap();
            let chat = super::create_chat(&connection, new_chat).unwrap();

            let new_subscription1 = NewTelegramSubscription {
                feed_id: feed1.id,
                chat_id: chat.id,
            };

            super::create_subscription(&connection, new_subscription1).unwrap();

            let new_subscription2 = NewTelegramSubscription {
                feed_id: feed2.id,
                chat_id: chat.id,
            };

            super::create_subscription(&connection, new_subscription2).unwrap();

            let result = super::find_subscriptions_for_chat(&connection, chat.id).unwrap();

            assert_eq!(result.len(), 2);
            assert_eq!(result[0].feed_id, feed1.id);
            assert_eq!(result[1].feed_id, feed2.id);

            Ok(())
        });
    }

    #[test]
    fn find_subscriptions_for_chat_does_not_return_wrong_chats() {
        let connection = db::establish_connection();

        connection.test_transaction::<(), Error, _>(|| {
            let feed =
                feeds::create(&connection, "Link80".to_string(), "atom".to_string()).unwrap();
            let chat1 = super::create_chat(&connection, build_new_chat_with_id(99)).unwrap();

            let chat2 = super::create_chat(&connection, build_new_chat_with_id(89)).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat1.id,
            };

            super::create_subscription(&connection, new_subscription).unwrap();

            let result = super::find_subscriptions_for_chat(&connection, chat2.id).unwrap();

            assert_eq!(result.len(), 0);

            Ok(())
        });
    }

    #[test]
    fn set_subscription_last_delivered_at_updates_last_delivered_at() {
        let connection = db::establish_connection();

        let new_chat = build_new_chat_with_id(900);

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "rss".to_string()).unwrap();
            let chat = super::create_chat(&connection, new_chat).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat.id,
            };

            let subscription = super::create_subscription(&connection, new_subscription).unwrap();

            assert!(subscription.last_delivered_at.is_none());

            let updated_subscription = super::set_subscription_last_delivered_at(
                &connection,
                &subscription,
                db::current_time(),
            )
            .unwrap();

            assert!(updated_subscription.last_delivered_at.is_some());

            Ok(())
        });
    }

    #[test]
    fn remove_subscription_removes_subscription() {
        let connection = db::establish_connection();

        let new_chat = build_new_chat_with_id(9001);

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "rss".to_string()).unwrap();
            let chat = super::create_chat(&connection, new_chat).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat.id,
            };

            super::create_subscription(&connection, new_subscription.clone()).unwrap();

            let result = super::remove_subscription(&connection, new_subscription).unwrap();

            assert_eq!(result, 1);

            Ok(())
        });
    }

    #[test]
    fn remove_chat_removes_chat() {
        let connection = db::establish_connection();

        let new_chat = build_new_chat();

        connection.test_transaction::<(), Error, _>(|| {
            let chat = super::create_chat(&connection, new_chat).unwrap();

            assert!(super::find_chat(&connection, chat.id).is_some());

            let result = super::remove_chat(&connection, chat.id).unwrap();

            assert_eq!(result, 1);

            assert!(super::find_chat(&connection, chat.id).is_none());

            Ok(())
        });
    }

    #[test]
    fn set_utc_offset_minutes_sets_offset() {
        let connection = db::establish_connection();

        let new_chat = build_new_chat();

        connection.test_transaction::<(), Error, _>(|| {
            let chat = super::create_chat(&connection, new_chat).unwrap();

            assert!(super::find_chat(&connection, chat.id).is_some());

            let result = super::set_utc_offset_minutes(&connection, &chat, 180).unwrap();

            assert_eq!(result.utc_offset_minutes.unwrap(), 180);

            Ok(())
        });
    }

    #[test]
    fn set_global_template_sets_template() {
        let connection = db::establish_connection();

        let new_chat = build_new_chat_with_id(200);

        connection.test_transaction::<(), Error, _>(|| {
            let chat = super::create_chat(&connection, new_chat).unwrap();

            let result =
                super::set_global_template(&connection, &chat, "template".to_string()).unwrap();

            assert_eq!(result.template.unwrap(), "template".to_string());

            Ok(())
        });
    }

    #[test]
    fn set_template_sets_template() {
        let connection = db::establish_connection();

        connection.test_transaction::<(), Error, _>(|| {
            let new_chat = build_new_chat();
            let chat = super::create_chat(&connection, new_chat).unwrap();
            let feed = feeds::create(&connection, "Link".to_string(), "rss".to_string()).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat.id,
            };

            let subscription =
                super::create_subscription(&connection, new_subscription.clone()).unwrap();

            assert_eq!(subscription.template, None);

            let updated_subscription =
                super::set_template(&connection, &subscription, "my_template".to_string()).unwrap();

            assert_eq!(
                updated_subscription.template,
                Some("my_template".to_string())
            );

            Ok(())
        });
    }

    #[test]
    fn find_chats_by_feed_id_find_chats() {
        let connection = db::establish_connection();

        let new_chat1 = build_new_chat_with_id(10);
        let new_chat2 = build_new_chat_with_id(20);

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string(), "rss".to_string()).unwrap();
            let chat1 = super::create_chat(&connection, new_chat1).unwrap();
            let chat2 = super::create_chat(&connection, new_chat2).unwrap();

            let new_subscription1 = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat1.id,
            };

            super::create_subscription(&connection, new_subscription1.clone()).unwrap();

            let new_subscription2 = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat2.id,
            };

            super::create_subscription(&connection, new_subscription2.clone()).unwrap();

            let result = super::find_chats_by_feed_id(&connection, feed.id).unwrap();

            assert_eq!(result.len(), 2);

            Ok(())
        });
    }

    fn build_new_chat() -> NewTelegramChat {
        NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
            title: None,
        }
    }

    fn build_new_chat_with_id(id: i64) -> NewTelegramChat {
        NewTelegramChat {
            id: id,
            kind: "private".to_string(),
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
            title: None,
        }
    }
}
