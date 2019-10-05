use crate::models::Product;

#[cfg(test)]
use mockiato::mockable;

#[cfg_attr(test, mockable)]
pub trait ProductDataSource {
    fn get_product_with_identifier(&self, identifier: &str) -> Result<Option<Product>, ()>;
}
