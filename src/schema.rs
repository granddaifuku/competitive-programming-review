table! {
    reviews (id) {
        id -> Int4,
        uid -> Nullable<Varchar>,
        problem_name -> Nullable<Varchar>,
        problem_url -> Nullable<Varchar>,
        comment -> Nullable<Varchar>,
    }
}

table! {
    users (id) {
        id -> Int4,
        uid -> Nullable<Varchar>,
        user_name -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    reviews,
    users,
);
