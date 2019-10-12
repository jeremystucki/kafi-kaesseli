use crate::schema::products;

#[derive(Queryable, Insertable, Identifiable, Clone, Debug)]
#[primary_key(identifier)]
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

#[derive(Queryable, Clone, Debug)]
pub struct Balance {
    pub user_id: i32,
    pub name: String,
    pub amount: i32,
}
