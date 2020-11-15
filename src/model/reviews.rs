use validator::{Deserialize, Validate};

#[derive(Validate)]
struct Review {
    id: i32,
    uid: String,
    problem_name: String,
    problem_url: String,
    comment: String,
}
