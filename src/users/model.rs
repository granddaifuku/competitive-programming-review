use crate::utils::RE_ALP_NUM_SYM;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewUser {
    #[validate(length(min = 1, max = 100), regex(path = "RE_ALP_NUM_SYM"))]
    pub user_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 100), regex(path = "RE_ALP_NUM_SYM"))]
    pub password: String,
}
