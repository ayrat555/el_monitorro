use once_cell::sync::OnceCell;
use rocket::local::{Client, LocalResponse};
use serde_json::Value;

pub fn test_client() -> &'static Client {
    static INSTANCE: OnceCell<Client> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let rocket = el_monitorro::rocket();
        Client::new(rocket).expect("valid rocket instance")
    })
}

pub fn response_json_value(response: &mut LocalResponse) -> Value {
    let body = response.body().expect("no body");
    serde_json::from_reader(body.into_inner()).expect("can't parse value")
}
