use diesel::table;





table! {
    users {
        id -> Integer,
        name -> Text,
    }
}
