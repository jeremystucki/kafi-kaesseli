use chrono::Utc;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;
use diesel::SqliteConnection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use mockall::automock;

use transactions::dsl::transactions as transactions_dsl;

use crate::models::{Product, Rappen, Transaction, User};
use crate::schema::transactions;

pub enum TransactionKind {
    Amount(Rappen),
    Product(Product),
}

#[cfg_attr(test, automock)]
pub trait TransactionService: Send {
    fn register_transaction(
        &self,
        transaction_kind: TransactionKind,
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
}

impl TransactionService for TransactionServiceImpl<'_> {
    fn register_transaction(
        &self,
        transaction_kind: TransactionKind,
        sender: &User,
    ) -> Result<(), ()> {
        let (amount, product_name) = match transaction_kind {
            TransactionKind::Amount(amount) => (amount, None),
            TransactionKind::Product(Product { price, name, .. }) => (-price, Some(name)),
        };

        diesel::insert_into(transactions::table)
            .values(&Transaction {
                amount,
                product_name,
                timestamp: Utc::now().naive_utc(),
                user: sender.id.clone(),
            })
            .execute(self.database_connection)
            .map(|_| ())
            .map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
