use crate::models::feed::Feed;
use crate::schema::feeds;
use diesel::result::Error;
use diesel::{PgConnection, RunQueryDsl};

#[derive(Insertable)]
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
    fn it_failes_to_create_feed_without_title() {
        let title = "Title";
        let link = "";
        let description = "Description";
        let connection = db::establish_connection();

        connection.test_transaction::<Feed, Error, _>(|| {
            super::create(&connection, &title, &link, &description)
        });
    }
}
