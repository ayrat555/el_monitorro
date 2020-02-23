use crate::models::feed::Feed;
use crate::schema::feeds;
use diesel::result::Error;
use diesel::{PgConnection, RunQueryDsl};

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
        .set(new_feed)
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
    #[should_panic]
    fn it_fails_to_create_feed_without_link() {
        let title = "Title";
        let link = "";
        let description = "Description";
        let connection = db::establish_connection();

        connection.test_transaction::<Feed, Error, _>(|| {
            super::create(&connection, &title, &link, &description)
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

        let feed = connection.test_transaction::<Feed, Error, _>(|| {
            super::create(&connection, &title, &link, &description)
        });

        assert_eq!(feed.title, title);
        assert_eq!(feed.link, link);
        assert_eq!(feed.description, description);

        let updated_feed = connection.test_transaction::<Feed, Error, _>(|| {
            super::create(&connection, &updated_title, &link, &updated_description)
        });

        assert_eq!(updated_feed.title, updated_title);
        assert_eq!(updated_feed.link, link);
        assert_eq!(updated_feed.description, updated_description);
    }
}
