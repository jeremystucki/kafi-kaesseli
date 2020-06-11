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
    fn register_product_transaction(&self, product: &Product, sender: &User) -> Result<(), ()>;

    fn register_amount_transaction(&self, amount: Rappen, sender: &User) -> Result<(), ()>;
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

    fn insert_transaction(&self, transaction: Transaction) -> Result<(), ()> {
        diesel::insert_into(transactions::table)
            .values(transaction)
            .execute(self.database_connection)
            .map(|_| ())
            .map_err(|_| ())
    }
}

impl TransactionService for TransactionServiceImpl<'_> {
    fn register_product_transaction(&self, product: &Product, sender: &User) -> Result<(), ()> {
        self.insert_transaction(Transaction {
            amount: -product.price,
            timestamp: Utc::now().naive_utc(),
            user: sender.id.clone(),
            product_name: Some(product.name.clone()),
        })
    }

    fn register_amount_transaction(&self, amount: Rappen, sender: &User) -> Result<(), ()> {
        self.insert_transaction(Transaction {
            amount,
            timestamp: Utc::now().naive_utc(),
            user: sender.id.clone(),
            product_name: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use diesel::Connection;

    use transactions::dsl::transactions as transactions_dsl;

    use crate::schema::transactions;
    use crate::test_utils::*;

    use super::*;
    use chrono::NaiveDateTime;

    #[derive(Queryable, Debug)]
    struct Transaction {
        id: i32,
        amount: i32,
        timestamp: NaiveDateTime,
        user: String,
        product_name: Option<String>,
    }

    #[test]
    fn register_product_transaction() {
        let database_connection = setup_in_memory_database();

        let transaction_service = TransactionServiceImpl::new(&database_connection);

        let sender = User {
            id: "foo".to_string(),
            name: "bar".to_string(),
        };

        let product = Product {
            identifier: "lorem".to_string(),
            name: "ipsum".to_string(),
            price: 200,
        };

        transaction_service
            .register_product_transaction(&product, &sender)
            .unwrap();

        let transactions = transactions_dsl
            .load::<Transaction>(&database_connection)
            .unwrap();

        assert_eq!(transactions.len(), 1);

        let transaction = &transactions[0];
        assert_eq!(transaction.amount, -200);
        assert_eq!(transaction.user, "foo");
        assert_eq!(transaction.product_name, Some("ipsum".to_string()));
    }

    #[test]
    fn register_amount_transaction() {
        let database_connection = setup_in_memory_database();

        let transaction_service = TransactionServiceImpl::new(&database_connection);

        let sender = User {
            id: "foo".to_string(),
            name: "bar".to_string(),
        };

        let amount = 120;

        transaction_service
            .register_amount_transaction(amount, &sender)
            .unwrap();

        let transactions = transactions_dsl
            .load::<Transaction>(&database_connection)
            .unwrap();

        assert_eq!(transactions.len(), 1);

        let transaction = &transactions[0];
        assert_eq!(transaction.amount, 120);
        assert_eq!(transaction.user, "foo");
        assert_eq!(transaction.product_name, None);
    }
}
