use crate::schema::*;
use chrono::{NaiveDate, NaiveDateTime};

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
    pub user_id: String,
    pub name: String,
    pub amount: i32,
}

#[derive(Queryable, Insertable, Identifiable, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Insertable, Debug)]
pub struct Transaction {
    pub amount: i32,
    pub timestamp: NaiveDateTime,
    pub user: String,
    pub product_name: Option<String>,
}
