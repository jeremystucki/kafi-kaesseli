use chrono::Utc;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;
use diesel::SqliteConnection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use mockiato::mockable;

use transactions::dsl::transactions as transactions_dsl;

use crate::models::{Product, Rappen, Transaction, User};
use crate::schema::transactions;

#[cfg_attr(test, mockable)]
pub trait TransactionService {
    fn register_product_transaction(
        &self,
        product: &Product,
        sender: &User,
    ) -> Result<(), ()>;

    fn register_amount_transaction(
        &self,
        amount: Rappen,
        sender: &User,
    ) -> Result<(), ()>;
}

pub struct TransactionServiceImpl<'a> {
    database_connection: &'a SqliteConnection,
}

impl<'a> TransactionServiceImpl<'a> {
    pub fn new(database_connection: &'a SqliteConnection) -> Self {
        Self {
            database_connection,
        }
    }

    fn insert_transaction(
        &self,
        transaction: Transaction
    ) -> Result<(), ()> {
        diesel::insert_into(transactions::table)
            .values(transaction)
            .execute(self.database_connection)
            .map(|_| ())
            .map_err(|_| ())
    }
}

impl TransactionService for TransactionServiceImpl<'_> {
    fn register_product_transaction(
        &self,
        product: &Product,
        sender: &User,
    ) -> Result<(), ()> {
        self.insert_transaction(Transaction {
            amount: -product.price,
            timestamp: Utc::now().naive_utc(),
            user: sender.id.clone(),
            product_name: Some(product.name.clone())
        })
    }

    fn register_amount_transaction(
        &self,
        amount: Rappen,
        sender: &User,
    ) -> Result<(), ()> {
        self.insert_transaction(Transaction {
            amount,
            timestamp: Utc::now().naive_utc(),
            user: sender.id.clone(),
            product_name: None
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
