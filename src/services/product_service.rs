use diesel::SqliteConnection;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
#[cfg(test)]
use mockall::automock;

use products::dsl::products as products_dsl;

use crate::models::Product;
use crate::schema::products;

#[cfg_attr(test, automock)]
pub trait ProductService {
    fn get_available_products(&self) -> Result<Vec<Product>, ()>;
    fn get_product_with_identifier(&self, identifier: &str) -> Result<Option<Product>, ()>;
}

pub struct ProductServiceImpl<'a> {
    database_connection: &'a SqliteConnection,
}

impl<'a> ProductServiceImpl<'a> {
    pub fn new(database_connection: &'a SqliteConnection) -> Self {
        Self {
            database_connection,
        }
    }
}

impl ProductService for ProductServiceImpl<'_> {
    fn get_available_products(&self) -> Result<Vec<Product>, ()> {
        products_dsl
            .load::<Product>(self.database_connection)
            .map_err(|_| ())
    }

    fn get_product_with_identifier(&self, identifier: &str) -> Result<Option<Product>, ()> {
        products_dsl
            .find(identifier)
            .first::<Product>(self.database_connection)
            .optional()
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
    fn get_product_with_empty_database() {
        let database_connection = setup_in_memory_database();

        let product_service = ProductServiceImpl::new(&database_connection);

        let result = product_service.get_product_with_identifier("foo");
        assert_eq!(Ok(None), result);
    }

    #[test]
    fn get_product() {
        let product = Product {
            identifier: "foo".to_string(),
            name: "bar".to_string(),
            price: 120,
        };

        let database_connection = setup_in_memory_database();
        diesel::insert_into(products::table)
            .values(&product)
            .execute(&database_connection)
            .unwrap();

        let product_service = ProductServiceImpl::new(&database_connection);

        let result = product_service.get_product_with_identifier("foo");
        assert_eq!(Ok(Some(product)), result);
    }
}
