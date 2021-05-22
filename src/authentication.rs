use rocket::form::{Form, FromForm};
use rocket::State;
use sqlx::postgres::PgPool;
use std::sync::Arc;

#[derive(FromForm)]
struct NewUser {
    user_name: String,
    emai: String,
    password: String,
}

#[post("/sign-up", data = "<form>")]
pub async fn sign_up(pool: State<'_, Arc<PgPool>>, form: Form<NewUser>) {
    let pool = Arc::clone(&pool);
}
