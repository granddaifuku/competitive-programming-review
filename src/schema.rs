table! {
    reviews (id) {
        id -> Int4,
        uid -> Varchar,
        problem_name -> Varchar,
        problem_url -> Varchar,
        comment -> Nullable<Varchar>,
    }
}

table! {
    users (id) {
        id -> Int4,
        uid -> Varchar,
        user_name -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    reviews,
    users,
);
