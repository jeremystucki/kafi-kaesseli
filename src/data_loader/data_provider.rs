#[cfg(test)]
use mockiato::mockable;

use crate::models::Product;

#[cfg_attr(test, mockable)]
pub trait DataProvider<T> {
    fn get_data(&self) -> Box<dyn Iterator<Item = Result<T, ()>>>;
}
