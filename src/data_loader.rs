use diesel::{RunQueryDsl, SqliteConnection};

use crate::models::Product;
use crate::schema::products;

use data_provider::*;

mod data_provider;

pub trait DataLoader {
    fn load_product_data(&self) -> Result<(), ()>;
}

pub struct DataLoaderImpl<'a> {
    database_connection: &'a SqliteConnection,
    product_data_provider: Box<dyn DataProvider<Product>>,
}

impl<'a> DataLoaderImpl<'a> {
    pub fn new(
        database_connection: &'a SqliteConnection,
        product_data_provider: Box<dyn DataProvider<Product>>,
    ) -> DataLoaderImpl<'a> {
        Self {
            database_connection,
            product_data_provider,
        }
    }
}

impl DataLoader for DataLoaderImpl<'_> {
    fn load_product_data(&self) -> Result<(), ()> {
        diesel::delete(products::table)
            .execute(self.database_connection)
            .map_err(|_| ())?;

        self.product_data_provider
            .get_data()
            .map(|result| {
                result.map(|product| {
                    diesel::insert_into(products::table)
                        .values(product)
                        .execute(self.database_connection)
                })
            })
            .find(|result| match result {
                Ok(Ok(_)) => false,
                _ => true,
            })
            .map_or_else(|| Ok(()), |_| Err(()))
    }
}

#[cfg(test)]
mod tests {
    use diesel::Connection;

    use super::*;

    use crate::models::Product;
    use crate::test_utils::*;

    use data_provider::DataProviderMock;

    #[test]
    fn empties_product_table_before_insert() {
        let database_connection = setup_in_memory_database();

        let mut product_data_provider = DataProviderMock::<Product>::new();
        product_data_provider.expect_get_data_calls_in_order();

        product_data_provider
            .expect_get_data()
            .times(1)
            .returns_once(Box::new(
                vec![Product {
                    identifier: "foo".to_string(),
                    name: "foo bar".to_string(),
                    price: 120,
                }]
                .into_iter()
                .map(Ok),
            ));

        product_data_provider
            .expect_get_data()
            .times(1)
            .returns_once(Box::new(
                vec![Product {
                    identifier: "bar".to_string(),
                    name: "bar baz".to_string(),
                    price: 250,
                }]
                .into_iter()
                .map(Ok),
            ));

        let data_loader = DataLoaderImpl {
            database_connection: &database_connection,
            product_data_provider: Box::new(product_data_provider),
        };

        data_loader.load_product_data().unwrap();
        data_loader.load_product_data().unwrap();

        let products = products::dsl::products
            .load::<Product>(&database_connection)
            .unwrap();
        assert_eq!(1, products.len());

        let product = products.get(0).unwrap();
        assert_eq!("bar", product.identifier);
        assert_eq!("bar baz", product.name);
        assert_eq!(250, product.price);
    }
}
