table! {
    products (identifier) {
        identifier -> Text,
        name -> Text,
        price -> Integer,
    }
}

table! {
    transactions {
        id -> Integer,
        amount -> Integer,
        timestamp -> Timestamp,
        user -> Text,
        product_name -> Nullable<Text>,
    }
}

table! {
    users {
        id -> Text,
        name -> Text,
    }
}

table! {
    balances (user_id) {
        user_id -> Text,
        name -> Text,
        amount -> Integer,
    }
}

joinable!(transactions -> users (user));

allow_tables_to_appear_in_same_query!(products, transactions, users,);
