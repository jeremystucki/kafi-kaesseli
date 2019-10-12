use crate::currency_formatter::CurrencyFormatter;
use crate::message_router::MessageRouter;
use crate::models::{Balance, Product, Transaction, User};
use crate::schema::{balances, products, transactions, users};
use crate::{Command, Message, MessageAction, Person, Rappen, Response};
use chrono::prelude::Utc;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use diesel_migrations::name;

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

                if let Err(_) = self.register_transaction(-product.price, Some(product), sender) {
                    return vec![Response {
                        contents: "Internal error (4)".to_string(),
                    }];
                } else {
                    Response {
                        contents: format!("Recorded {}", product_name),
                    }
                }
            }
            MessageAction::Amount(amount) => {
                if let Err(_) = self.register_transaction(amount, None, sender) {
                    return vec![Response {
                        contents: "Internal error (5)".to_string(),
                    }];
                } else {
                    Response {
                        contents: format!(
                            "Recorded {}",
                            self.currency_formatter.format_amount(amount)
                        ),
                    }
                }
            }
        };

        if let Ok(products) = self.get_available_products() {
            vec![
                confirmation,
                Response {
                    contents: self.format_available_products(products),
                },
            ]
        } else {
            vec![confirmation]
        }
    }

    fn register_transaction(
        &self,
        amount: Rappen,
        product: Option<Product>,
        sender: &Person,
    ) -> Result<(), ()> {
        use transactions::dsl;

        let sender_id = sender.id.clone();

        self.update_user(sender)?;

        diesel::insert_into(transactions::table)
            .values(&Transaction {
                amount,
                timestamp: Utc::now().naive_utc(),
                user: sender_id,
                product_name: product.map(|product| product.name),
            })
            .execute(self.database_connection)
            .map(|_| Ok(()))
            .unwrap_or_else(|_| Err(()))
    }

    fn update_user(&self, sender: &Person) -> Result<(), ()> {
        use users::dsl;

        let Person { id, name } = sender;

        let result = diesel::update(users::table)
            .filter(dsl::id.eq(id))
            .set(dsl::name.eq(name))
            .execute(self.database_connection);

        match result {
            Ok(_) => return Ok(()),
            Err(diesel::NotFound) => (),
            Err(_) => return Err(()),
        };

        diesel::insert_into(users::table)
            .values(&User {
                id: id.to_string(),
                name: name.to_string(),
            })
            .execute(self.database_connection)
            .map(|_| Ok(()))
            .unwrap_or_else(|_| Err(()))
    }

    fn handle_command(&self, command: Command, sender: &Person) -> Vec<Response> {
        match command {
            Command::ListAvailableItems => self
                .get_available_products()
                .map(|products| {
                    vec![Response {
                        contents: self.format_available_products(products),
                    }]
                })
                .unwrap_or_else(|_| {
                    vec![Response {
                        contents: "Internal error (2)".to_string(),
                    }]
                }),
            Command::GetCurrentStats => self
                .get_balances()
                .map(|balances| {
                    vec![Response {
                        contents: self.format_balances(balances, sender),
                    }]
                })
                .unwrap_or_else(|_| {
                    vec![Response {
                        contents: "Internal error (3)".to_string(),
                    }]
                }),
        }
    }

    fn get_available_products(&self) -> Result<Vec<Product>, ()> {
        products::dsl::products
            .load::<Product>(self.database_connection)
            .map_err(|_| ())
    }

    fn format_available_products(&self, available_products: Vec<Product>) -> String {
        let message_header = "Available products:";
        let message_body = available_products
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

    fn get_balances(&self) -> Result<Vec<Balance>, ()> {
        balances::dsl::balances
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
