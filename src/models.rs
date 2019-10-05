use crate::schema::products;

#[derive(Queryable, Clone, Debug)]
pub struct Product {
    pub identifier: String,
    pub name: String,
    pub price: i32,
}

impl PartialEq for Product {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

#[derive(Insertable, Clone)]
#[table_name = "products"]
pub struct NewProduct {
    pub identifier: String,
    pub name: String,
    pub price: i32,
}
