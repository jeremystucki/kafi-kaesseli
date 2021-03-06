use chrono::{NaiveDate, NaiveDateTime};

use crate::schema::*;

pub type Rappen = i32;

#[derive(Debug, PartialEq)]
pub struct Message {
    pub sender: User,
    pub contents: String,
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub contents: String,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    GetCurrentBalances,
    ListAvailableItems,
}

#[derive(Debug, PartialEq)]
pub enum MessageAction {
    Amount(Rappen),
    Command(Command),
    Product(Product),
}

#[derive(Queryable, Insertable, Identifiable, Clone, Debug)]
#[primary_key(identifier)]
pub struct Product {
    pub identifier: String,
    pub name: String,
    pub price: Rappen,
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

#[derive(Queryable, Insertable, Identifiable, Clone, Debug)]
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
pub(crate) struct Transaction {
    pub(crate) amount: i32,
    pub(crate) timestamp: NaiveDateTime,
    pub(crate) user: String,
    pub(crate) product_name: Option<String>,
}
