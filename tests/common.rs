use once_cell::sync::OnceCell;
use rocket::local::Client;

pub fn test_client() -> &'static Client {
    static INSTANCE: OnceCell<Client> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let rocket = realworld::rocket();
        Client::new(rocket).expect("valid rocket instance")
    })
}
