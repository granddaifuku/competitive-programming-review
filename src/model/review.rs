use rocket_contrib::uuid::Uuid;
use validator::{Deserialize, Validate};

#[derive(Validate)]
struct Review {
    id: i32,
    uid: Uuid,
    prob_name: &str,
    prob_url: &str,
    comment: &str,
}

impl Review {
    fn new(uid: Uuid, prob_name: &str, prob_url: &str, comment: &str) -> Review {
        Review {
            id: 0,
            uid,
            prob_name,
            prob_url,
            comment,
        }
    }
}
