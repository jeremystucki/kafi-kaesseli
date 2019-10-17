use chrono::prelude::Utc;
use diesel::{insert_into, update, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use diesel_migrations::name;

use crate::currency_formatter::CurrencyFormatter;
use crate::message_router::MessageRouter;
use crate::models::{Balance, Product, Transaction, User};
use crate::schema::{balances, products, transactions, users};
use crate::{Command, Message, MessageAction, Person, Rappen, Response};

use balances::dsl::balances as balances_dsl;
use products::dsl::products as products_dsl;
use transactions::dsl::transactions as transactions_dsl;
use users::dsl::users as users_dsl;

pub trait MessageHandler {
    fn handle_message(&self, message: &Message) -> Vec<Response>;
}

pub struct MessageHandlerImpl<'a> {
    database_connection: &'a SqliteConnection,
    message_router: Box<dyn MessageRouter>,
    currency_formatter: Box<dyn CurrencyFormatter>,
}

impl MessageHandlerImpl<'_> {
    fn handle_message_action(
        &self,
        message_action: MessageAction,
        sender: &Person,
    ) -> Vec<Response> {
        let confirmation = match message_action {
            MessageAction::Command(command) => return self.handle_command(command, sender),
            MessageAction::Product(product) => {
                let product_name = product.name.clone();

                if self
                    .register_transaction(-product.price, Some(&product), sender)
                    .is_err()
                {
                    return vec![Response {
                        contents: "Internal error (4)".to_string(),
                    }];
                }

                Response {
                    contents: format!("Recorded {}", product_name),
                }
            }
            MessageAction::Amount(amount) => {
                if self.register_transaction(amount, None, sender).is_err() {
                    return vec![Response {
                        contents: "Internal error (5)".to_string(),
                    }];
                }

                Response {
                    contents: format!("Recorded {}", self.currency_formatter.format_amount(amount)),
                }
            }
        };

        if let Ok(formatted_balances) = self.get_formatted_balances(&sender) {
            vec![
                confirmation,
                Response {
                    contents: formatted_balances,
                },
            ]
        } else {
            vec![confirmation]
        }
    }

    fn register_transaction(
        &self,
        amount: Rappen,
        product: Option<&Product>,
        sender: &Person,
    ) -> Result<(), ()> {
        self.update_user(sender)?;

        diesel::insert_into(transactions::table)
            .values(&Transaction {
                amount,
                timestamp: Utc::now().naive_utc(),
                user: sender.id.clone(),
                product_name: product.map(|product| product.name.clone()),
            })
            .execute(self.database_connection)
            .map(|_| ())
            .map_err(|_| ())
    }

    fn update_user(&self, sender: &Person) -> Result<(), ()> {
        let Person { id, name } = sender;

        match update(users_dsl.find(id))
            .set(users::name.eq(name))
            .execute(self.database_connection)
        {
            Ok(_) => return Ok(()),
            Err(diesel::NotFound) => (),
            Err(_) => return Err(()),
        }

        diesel::insert_into(users::table)
            .values(&User {
                id: id.to_string(),
                name: name.to_string(),
            })
            .execute(self.database_connection)
            .map(|_| ())
            .map_err(|_| ())
    }

    fn handle_command(&self, command: Command, sender: &Person) -> Vec<Response> {
        let contents = match command {
            Command::ListAvailableItems => self
                .get_formatted_available_products()
                .unwrap_or_else(|_| "Internal error (2)".to_string()),
            Command::GetCurrentStats => self
                .get_formatted_balances(sender)
                .unwrap_or_else(|_| "Internal error (3)".to_string()),
        };

        vec![Response { contents }]
    }

    fn get_formatted_available_products(&self) -> Result<String, ()> {
        self.get_available_products()
            .map(|products| self.format_products(&products))
    }

    fn get_available_products(&self) -> Result<Vec<Product>, ()> {
        products_dsl
            .load::<Product>(self.database_connection)
            .map_err(|_| ())
    }

    fn format_products(&self, products: &[Product]) -> String {
        let message_header = "Available products:";
        let message_body = products
            .into_iter()
            .map(|product| {
                format!(
                    "- /{} - {} ({})",
                    product.identifier, product.name, product.price
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("{}\n{}", message_header, message_body)
    }

    fn get_formatted_balances(&self, sender: &Person) -> Result<String, ()> {
        self.get_balances()
            .map(|balances| self.format_balances(balances, sender))
    }

    fn get_balances(&self) -> Result<Vec<Balance>, ()> {
        balances_dsl
            .load::<Balance>(self.database_connection)
            .map_err(|_| ())
    }

    fn format_balances(&self, balances: Vec<Balance>, sender: &Person) -> String {
        let message_header = "Current stats:";
        let message_body = balances
            .into_iter()
            .map(|balance| {
                let text = format!(
                    "{} - {}",
                    balance.name,
                    self.currency_formatter.format_amount(balance.amount)
                );

                if balance.user_id == sender.id {
                    format!("**{}**", text)
                } else {
                    text
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("{}\n{}", message_header, message_body)
    }
}

impl MessageHandler for MessageHandlerImpl<'_> {
    fn handle_message(&self, message: &Message) -> Vec<Response> {
        match self.message_router.route_message(message) {
            Err(_) => vec![Response {
                contents: "Internal error (1)".to_string(),
            }],
            Ok(None) => vec![Response {
                contents: "Invalid input".to_string(),
            }],
            Ok(Some(message_action)) => self.handle_message_action(message_action, &message.sender),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency_formatter::CurrencyFormatterMock;
    use crate::message_router::MessageRouterMock;
    use crate::{Command, MessageAction, Person, Response};
    use diesel::Connection;

    fn message_mock() -> Message {
        Message {
            sender: Person {
                id: "some id".to_string(),
                name: "foo".to_string(),
            },
            contents: "bar".to_string(),
        }
    }

    #[test]
    fn invalid_message() {
        let database_connection = SqliteConnection::establish(":memory:").unwrap();

        let mut message_router = MessageRouterMock::new();
        message_router
            .expect_route_message(|arg| arg.any())
            .returns_once(Ok(None));

        let currency_formatter = CurrencyFormatterMock::new();

        let message_handler = MessageHandlerImpl {
            database_connection: &database_connection,
            message_router: Box::new(message_router),
            currency_formatter: Box::new(currency_formatter),
        };

        let responses = message_handler.handle_message(&message_mock());
        assert_eq!(
            vec![Response {
                contents: "Invalid input".to_string()
            }],
            responses
        );
    }
}
