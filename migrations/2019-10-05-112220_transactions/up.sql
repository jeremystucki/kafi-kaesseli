CREATE TABLE transactions (
    id INTEGER PRIMARY KEY NOT NULL,
    amount INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    user TEXT NOT NULL,
    product_name TEXT,

    FOREIGN KEY(user) REFERENCES users(id)
)
