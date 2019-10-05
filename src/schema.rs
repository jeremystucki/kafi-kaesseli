table! {
    products (id) {
        id -> Integer,
        identifier -> Text,
        name -> Text,
        price -> Integer,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    products,
    users,
);
