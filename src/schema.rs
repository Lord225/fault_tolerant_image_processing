// @generated automatically by Diesel CLI.

diesel::table! {
    new_table (id) {
        id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    new_table,
    users,
);
