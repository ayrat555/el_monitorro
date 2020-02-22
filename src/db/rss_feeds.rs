use crate::schema::feeds;

#[derive(Insertable)]
#[table_name = "feeds"]
struct NewRssFeed<'a> {
    title: &'a str,
    link: &'a str,
    description: &'a str,
}
