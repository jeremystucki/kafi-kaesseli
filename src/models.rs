use crate::schema::products;

#[derive(Queryable)]
pub struct Product {
    pub id: i32,
    pub identifier: String,
    pub name: String,
    pub price: i32,
}

#[derive(Insertable, Clone)]
#[table_name = "products"]
pub struct NewProduct {
    pub identifier: String,
    pub name: String,
    pub price: i32,
}
