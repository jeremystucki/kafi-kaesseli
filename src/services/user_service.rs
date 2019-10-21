use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;
use diesel::SqliteConnection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use mockiato::mockable;

use users::dsl::users as users_dsl;

use crate::models::User;
use crate::schema::users;

#[cfg_attr(test, mockable)]
pub trait UserService {
    fn update_user(&self, user: &User) -> Result<(), ()>;
}

pub struct UserServiceImpl<'a> {
    database_connection: &'a SqliteConnection,
}

impl<'a> UserServiceImpl<'a> {
    pub fn new(database_connection: &'a SqliteConnection) -> Self {
        Self {
            database_connection,
        }
    }
}

impl UserService for UserServiceImpl<'_> {
    fn update_user(&self, user: &User) -> Result<(), ()> {
        let User { id, name } = user;

        match diesel::update(users_dsl.find(id))
            .set(users::name.eq(name))
            .execute(self.database_connection)
        {
            Ok(0) => (),
            Ok(_) => return Ok(()),
            Err(_) => return Err(()),
        }

        diesel::insert_into(users::table)
            .values(user)
            .execute(self.database_connection)
            .map(|_| ())
            .map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use diesel::Connection;

    use products::dsl::products as products_dsl;

    use crate::models::Product;
    use crate::schema::products;
    use crate::test_utils::*;

    use super::*;

    #[test]
    fn insert_user() {
        let database_connection = setup_in_memory_database();

        let user_service = UserServiceImpl::new(&database_connection);

        let user = User {
            id: "foo".to_string(),
            name: "bar".to_string(),
        };

        user_service.update_user(&user).unwrap();

        let users = users_dsl.load::<User>(&database_connection).unwrap();
        assert_eq!(vec![user], users)
    }

    #[test]
    fn update_user() {
        let database_connection = setup_in_memory_database();

        let mut user = User {
            id: "foo".to_string(),
            name: "bar".to_string(),
        };

        diesel::insert_into(users::table)
            .values(&user)
            .execute(&database_connection)
            .unwrap();

        user.name = "baz".to_string();

        let user_service = UserServiceImpl::new(&database_connection);
        user_service.update_user(&user).unwrap();

        let users = users_dsl.load::<User>(&database_connection).unwrap();
        assert_eq!(vec![user], users)
    }
}
