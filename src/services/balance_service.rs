use diesel::SqliteConnection;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
#[cfg(test)]
use mockall::automock;

use balances::dsl::balances as balances_dsl;

use crate::models::Balance;
use crate::schema::balances;

#[cfg_attr(test, automock)]
pub trait BalanceService {
    fn get_balances(&self) -> Result<Vec<Balance>, ()>;
}

pub struct BalanceServiceImpl<'a> {
    database_connection: &'a SqliteConnection,
}

impl<'a> BalanceServiceImpl<'a> {
    pub fn new(database_connection: &'a SqliteConnection) -> Self {
        Self {
            database_connection,
        }
    }
}

impl BalanceService for BalanceServiceImpl<'_> {
    fn get_balances(&self) -> Result<Vec<Balance>, ()> {
        balances_dsl
            .load::<Balance>(self.database_connection)
            .map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
