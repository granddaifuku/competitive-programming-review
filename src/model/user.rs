use rocket_contrib::uuid::Uuid;
use validator::{Deserialize, Validate};

#[derive(Validate)]
struct User {
    id: i32,
    uid: Uuid,
    username: &str,
    password: &str,
}

impl User {
    fn new(uid: Uuid, username: &str, password: &str) -> User {
        User {
            id: 0,
            uid,
            username,
            password,
        }
    }
}
