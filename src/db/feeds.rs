use crate::db;
use crate::models::feed::Feed;
use crate::schema::feeds;
use diesel::result::Error;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};

#[derive(Insertable, AsChangeset)]
#[table_name = "feeds"]
struct NewFeed<'a> {
    title: &'a str,
    link: &'a str,
    description: &'a str,
}

pub fn create(
    conn: &PgConnection,
    title: &str,
    link: &str,
    description: &str,
) -> Result<Feed, Error> {
    let new_feed = &NewFeed {
        title,
        link,
        description,
    };

    diesel::insert_into(feeds::table)
        .values(new_feed)
        .on_conflict(feeds::link)
        .do_update()
        .set((
            feeds::title.eq(new_feed.title),
            feeds::description.eq(new_feed.description),
            feeds::updated_at.eq(db::current_time()),
        ))
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::models::feed::Feed;
    use diesel::connection::Connection;
    use diesel::result::Error;

    #[test]
    fn it_creates_new_feed() {
        let title = "Title";
        let link = "Link";
        let description = "Description";
        let connection = db::establish_connection();

        let result = connection.test_transaction::<Feed, Error, _>(|| {
            super::create(&connection, &title, &link, &description)
        });

        assert_eq!(result.title, title);
        assert_eq!(result.link, link);
        assert_eq!(result.description, description);
    }

    #[test]
    fn it_fails_to_create_feed_without_link() {
        let title = "Title";
        let link = "";
        let description = "Description";
        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let result = super::create(&connection, &title, &link, &description);

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
    fn it_updates_feed_if_it_already_exists() {
        let title = "Title";
        let updated_title = "NewTitle";

        let link = "Link";

        let description = "Description";
        let updated_description = "NewDescripton";

        let connection = db::establish_connection();

        connection.test_transaction::<_, Error, _>(|| {
            let feed = super::create(&connection, &title, &link, &description).unwrap();

            assert_eq!(feed.title, title);
            assert_eq!(feed.link, link);
            assert_eq!(feed.description, description);

            let updated_feed =
                super::create(&connection, &updated_title, &link, &updated_description).unwrap();

            assert_eq!(updated_feed.title, updated_title);
            assert_eq!(updated_feed.link, link);
            assert_eq!(updated_feed.description, updated_description);
            assert_eq!(updated_feed.created_at, feed.created_at);

            let updated_at_diff = updated_feed
                .updated_at
                .signed_duration_since(feed.updated_at)
                .num_microseconds()
                .unwrap();

            assert!(updated_at_diff > 0);

            Ok(())
        });
    }
}
