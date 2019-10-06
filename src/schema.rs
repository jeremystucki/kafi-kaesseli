table! {
    products (identifier) {
        identifier -> Text,
        name -> Text,
        price -> Integer,
    }
}

table! {
    transactions (id) {
        id -> Integer,
        amount -> Integer,
        timestamp -> Text,
        user -> Integer,
        message -> Text,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    balances (id) {
        name -> Text,
        balance -> Integer,
    }
}

joinable!(transactions -> users (user));

allow_tables_to_appear_in_same_query!(products, transactions, users,);
