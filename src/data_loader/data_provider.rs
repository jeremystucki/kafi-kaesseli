use crate::models::Product;

#[cfg(test)]
use mockiato::mockable;

#[cfg_attr(test, mockable)]
pub trait DataProvider<T> {
    fn get_data(&self) -> Box<dyn Iterator<Item = Result<T, ()>>>;
}

pub struct ProductDataProvider;

impl ProductDataProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl DataProvider<Product> for ProductDataProvider {
    fn get_data(&self) -> Box<dyn Iterator<Item = Result<Product, ()>>> {
        unimplemented!()
    }
}
