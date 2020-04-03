use crate::db;
use crate::models::telegram_chat::TelegramChat;
use crate::models::telegram_subscription::TelegramSubscription;
use crate::schema::{telegram_chats, telegram_subscriptions};
use diesel::pg::upsert::excluded;
use diesel::result::Error;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

#[derive(Insertable, Clone)]
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
            telegram_chats::title.eq(excluded(telegram_chats::title)),
            telegram_chats::username.eq(excluded(telegram_chats::username)),
            telegram_chats::first_name.eq(excluded(telegram_chats::first_name)),
            telegram_chats::last_name.eq(excluded(telegram_chats::last_name)),
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

pub fn count_subscriptions_for_chat(conn: &PgConnection, chat_id: i64) -> i64 {
    telegram_subscriptions::table
        .filter(telegram_subscriptions::chat_id.eq(chat_id))
        .count()
        .get_result::<i64>(conn)
        .unwrap()
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
    fn it_creates_new_telegram_chat() {
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };
        let connection = db::establish_connection();

        let result = connection.test_transaction::<TelegramChat, Error, _>(|| {
            super::create_chat(&connection, new_chat.clone())
        });

        assert_eq!(result.id, new_chat.id);
        assert_eq!(result.kind, new_chat.kind);
        assert_eq!(result.title, new_chat.title);
        assert_eq!(result.username, new_chat.username);
        assert_eq!(result.first_name, new_chat.first_name);
        assert_eq!(result.last_name, new_chat.last_name);
    }

    #[test]
    fn it_updates_telegram_chat() {
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };
        let updated_chat = NewTelegramChat {
            id: 42,
            kind: "public1".to_string(),
            title: Some("title1".to_string()),
            username: Some("Username1".to_string()),
            first_name: Some("First1".to_string()),
            last_name: Some("Last1".to_string()),
        };
        let connection = db::establish_connection();

        let new_result = connection.test_transaction::<TelegramChat, Error, _>(|| {
            let result = super::create_chat(&connection, new_chat.clone()).unwrap();

            assert_eq!(result.id, new_chat.id);
            assert_eq!(result.kind, new_chat.kind);
            assert_eq!(result.title, new_chat.title);
            assert_eq!(result.username, new_chat.username);
            assert_eq!(result.first_name, new_chat.first_name);
            assert_eq!(result.last_name, new_chat.last_name);

            super::create_chat(&connection, updated_chat.clone())
        });

        assert_eq!(new_result.id, updated_chat.id);
        assert_eq!(new_result.kind, updated_chat.kind);
        assert_eq!(new_result.title, updated_chat.title);
        assert_eq!(new_result.username, updated_chat.username);
        assert_eq!(new_result.first_name, updated_chat.first_name);
        assert_eq!(new_result.last_name, updated_chat.last_name);
    }

    #[test]
    fn it_creates_new_subscription() {
        let connection = db::establish_connection();

        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string()).unwrap();
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
    fn it_fails_to_create_new_subscription_if_it_already_exists() {
        let connection = db::establish_connection();

        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string()).unwrap();
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
    fn it_fails_to_create_new_subscription_if_it_chat_does_not_exist() {
        let connection = db::establish_connection();

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string()).unwrap();

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
    fn it_finds_subscription() {
        let connection = db::establish_connection();

        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string()).unwrap();
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
    fn it_fails_to_find_a_subscription() {
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
    fn it_counts_the_number_of_subscriptions() {
        let connection = db::establish_connection();

        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        connection.test_transaction::<(), Error, _>(|| {
            let feed = feeds::create(&connection, "Link".to_string()).unwrap();
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
}