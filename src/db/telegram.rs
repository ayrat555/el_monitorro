use crate::db;
use crate::models::telegram_chat::TelegramChat;
use crate::models::telegram_subscription::TelegramSubscription;
use crate::schema::{telegram_chats, telegram_subscriptions};
use diesel::pg::upsert::excluded;
use diesel::result::Error;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};

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

#[derive(Insertable, Clone)]
#[table_name = "telegram_subscriptions"]
pub struct NewTelegramSubscription {
    pub chat_id: i64,
    pub feed_id: i32,
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

pub fn create_subscription(
    conn: &PgConnection,
    subscription: NewTelegramSubscription,
) -> Result<TelegramSubscription, Error> {
    diesel::insert_into(telegram_subscriptions::table)
        .values(subscription)
        .get_result::<TelegramSubscription>(conn)
}

#[cfg(test)]
mod tests {
    use super::NewTelegramChat;
    use crate::db;
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
}
