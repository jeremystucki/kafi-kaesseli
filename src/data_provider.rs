use crate::models::NewProduct;

#[cfg(test)]
use mockiato::mockable;

#[cfg_attr(test, mockable)]
pub trait DataProvider<T> {
    fn get_data(&self) -> Box<dyn Iterator<Item = Result<T, ()>>>;
}

pub struct ProductDataProvider;

impl DataProvider<NewProduct> for ProductDataProvider {
    fn get_data(&self) -> Box<dyn Iterator<Item = Result<NewProduct, ()>>> {
        unimplemented!()
    }
}
