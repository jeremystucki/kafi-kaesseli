use diesel::SqliteConnection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::models::Product;
use crate::schema::products;

use products::dsl::products as products_dsl;

#[cfg(test)]
use mockiato::mockable;

#[cfg_attr(test, mockable)]
pub trait ProductDataSource {
    fn get_product_with_identifier(&self, identifier: &str) -> Result<Option<Product>, ()>;
}

pub struct ProductDataSourceImpl<'a> {
    database_connection: &'a SqliteConnection,
}

impl<'a> ProductDataSourceImpl<'a> {
    fn new(database_connection: &'a SqliteConnection) -> Self {
        Self {
            database_connection,
        }
    }
}

impl ProductDataSource for ProductDataSourceImpl<'_> {
    fn get_product_with_identifier(&self, identifier: &str) -> Result<Option<Product>, ()> {
        products_dsl
            .filter(products::identifier.eq(identifier))
            .load::<Product>(self.database_connection)
            .map(|products| products.into_iter().next())
            .map_err(|_| ())
    }
}
