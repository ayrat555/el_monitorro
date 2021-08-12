// use super::Command;
// use super::Message;
// use diesel::r2d2::ConnectionManager;
// use diesel::r2d2::Pool;
// use diesel::Connection;
// use diesel::PgConnection;

// static COMMAND: &str = "/unsubscribe";

// impl Command for Unsubscribe {
//     fn response(
//         &self,
//         db_pool: Pool<ConnectionManager<PgConnection>>,
//         message: &Message,
//     ) -> String {
//         match self.fetch_db_connection(db_pool) {
//             Ok(connection) => {
//                 let text = message.text().unwrap();
//                 let argument = self.parse_argument(&text);
//                 self.subscribe(&connection, message, argument)
//             }
//             Err(error_message) => error_message,
//         }
//     }

//     fn command(&self) -> &str {
//         COMMAND
//     }
// }
