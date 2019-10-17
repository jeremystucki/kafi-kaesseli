use diesel::SqliteConnection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::models::Product;
use crate::schema::products;

use products::dsl::products as products_dsl;

#[cfg(test)]
use mockiato::mockable;

#[cfg_attr(test, mockable)]
pub trait ProductService {
    fn get_product_with_identifier(&self, identifier: &str) -> Result<Option<Product>, ()>;
}

pub struct ProductServiceImpl<'a> {
    database_connection: &'a SqliteConnection,
}

impl<'a> ProductServiceImpl<'a> {
    fn new(database_connection: &'a SqliteConnection) -> Self {
        Self {
            database_connection,
        }
    }
}

impl ProductService for ProductServiceImpl<'_> {
    fn get_product_with_identifier(&self, identifier: &str) -> Result<Option<Product>, ()> {
        match products_dsl
            .find(identifier)
            .first::<Product>(self.database_connection)
        {
            Ok(product) => Ok(Some(product)),
            Err(diesel::NotFound) => Ok(None),
            Err(_) => Err(()),
        }
    }
}
