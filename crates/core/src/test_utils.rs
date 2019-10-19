use diesel::{Connection, SqliteConnection};

embed_migrations!("migrations");

pub(crate) fn setup_in_memory_database() -> SqliteConnection {
    let database_connection = SqliteConnection::establish(":memory:").unwrap();

    embedded_migrations::run(&database_connection).unwrap();

    database_connection
}
