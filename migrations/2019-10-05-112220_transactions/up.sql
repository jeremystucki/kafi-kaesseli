CREATE TABLE transactions (
    id INTEGER PRIMARY KEY NOT NULL,
    amount INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    user INTEGER NOT NULL,
    message TEXT NOT NULL,

    FOREIGN KEY(user) REFERENCES users(id)
)
