use chrono::prelude::Utc;
use diesel::{insert_into, update, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use diesel_migrations::name;

use balances::dsl::balances as balances_dsl;
use products::dsl::products as products_dsl;
use transactions::dsl::transactions as transactions_dsl;
use users::dsl::users as users_dsl;

use crate::currency_handling::currency_formatter::CurrencyFormatter;
use crate::message_router::MessageRouter;
use crate::models::{
    Balance, Command, Message, MessageAction, Product, Rappen, Response, Transaction, User,
};
use crate::schema::{balances, products, transactions, users};
use crate::services::balance_service;
use crate::services::balance_service::BalanceService;
use crate::services::product_service;
use crate::services::product_service::ProductService;
use crate::services::transaction_service;
use crate::services::transaction_service::{TransactionKind, TransactionService};
use crate::services::user_service;
use crate::services::user_service::UserService;

error_chain! {
    links {
        BalanceService(balance_service::Error, balance_service::ErrorKind);
        ProductService(product_service::Error, product_service::ErrorKind);
        TransactionService(transaction_service::Error, transaction_service::ErrorKind);
        UserService(user_service::Error, user_service::ErrorKind);
    }
}

pub trait MessageHandler {
    fn handle_message(&self, message: &Message) -> Vec<Response>;
}

pub struct MessageHandlerImpl<'a> {
    message_router: Box<dyn MessageRouter + 'a>,
    user_service: Box<dyn UserService + 'a>,
    product_service: Box<dyn ProductService + 'a>,
    transaction_service: Box<dyn TransactionService + 'a>,
    balance_service: Box<dyn BalanceService + 'a>,
    currency_formatter: Box<dyn CurrencyFormatter + 'a>,
}

impl<'a> MessageHandlerImpl<'a> {
    pub fn new(
        message_router: Box<dyn MessageRouter + 'a>,
        user_service: Box<dyn UserService + 'a>,
        product_service: Box<dyn ProductService + 'a>,
        transaction_service: Box<dyn TransactionService + 'a>,
        balance_service: Box<dyn BalanceService + 'a>,
        currency_formatter: Box<dyn CurrencyFormatter + 'a>,
    ) -> Self {
        Self {
            message_router,
            user_service,
            product_service,
            transaction_service,
            balance_service,
            currency_formatter,
        }
    }

    fn handle_message_action(
        &self,
        message_action: MessageAction,
        sender: &User,
    ) -> Result<Vec<Response>> {
        let transaction_kind = match message_action {
            MessageAction::Command(command) => return self.handle_command(command, sender),
            MessageAction::Product(product) => TransactionKind::Product(product),
            MessageAction::Amount(amount) => TransactionKind::Amount(amount),
        };

        let success_message = match &transaction_kind {
            TransactionKind::Amount(amount) => format!(
                "Recorded {}",
                self.currency_formatter.format_amount(*amount)
            ),
            TransactionKind::Product(Product { name, .. }) => format!("Recorded {}", name),
        };

        self.user_service
            .update_user(sender)
            .map_err(|e| ErrorKind::UserService(e.0))?; // TODO
        self.transaction_service
            .register_transaction(transaction_kind, sender)
            .map_err(|e| ErrorKind::TransactionService(e.0))?; // TODO

        let mut responses = vec![Response {
            contents: success_message,
        }];

        if let Ok(formatted_balances) = self.get_formatted_balances(&sender) {
            responses.push(Response {
                contents: formatted_balances,
            });
        }

        Ok(responses)
    }

    fn handle_command(&self, command: Command, sender: &User) -> Result<Vec<Response>> {
        let contents = match command {
            Command::ListAvailableItems => self.get_formatted_available_products()?,
            Command::GetCurrentStats => self.get_formatted_balances(sender)?,
        };

        Ok(vec![Response { contents }])
    }

    fn get_formatted_available_products(&self) -> Result<String> {
        self.product_service
            .get_available_products()
            .map(|products| self.format_products(&products))
            .map_err(Into::into)
    }

    fn format_products(&self, products: &[Product]) -> String {
        let message_header = "Available products:";
        let message_body = products
            .iter()
            .map(|product| {
                format!(
                    "/{} - {} ({})",
                    product.identifier,
                    product.name,
                    self.currency_formatter.format_amount(product.price)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("{}\n{}", message_header, message_body)
    }

    fn get_formatted_balances(&self, sender: &User) -> Result<String> {
        self.balance_service
            .get_balances()
            .map(|balances| self.format_balances(balances, sender))
            .map_err(Into::into)
    }

    fn format_balances(&self, balances: Vec<Balance>, sender: &User) -> String {
        let message_header = "Current stats:";
        let message_body = balances
            .into_iter()
            .map(|balance| {
                let text = format!(
                    "- {} ({})",
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
            Ok(Some(message_action)) => self
                .handle_message_action(message_action, &message.sender)
                .unwrap_or_else(|error| {
                    vec![Response {
                        contents: format!("{:?}", error),
                    }]
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::currency_handling::currency_formatter::CurrencyFormatterMock;
    use crate::message_router::MessageRouterMock;
    use crate::services::balance_service::BalanceServiceMock;
    use crate::services::product_service::ProductServiceMock;
    use crate::services::transaction_service::TransactionServiceMock;
    use crate::services::user_service::UserServiceMock;

    use super::*;

    fn message_mock() -> Message {
        Message {
            sender: User {
                id: "some id".to_string(),
                name: "foo".to_string(),
            },
            contents: "bar".to_string(),
        }
    }

    #[test]
    fn invalid_input() {
        let mut message_router = MessageRouterMock::new();
        message_router
            .expect_route_message(|arg| arg.any())
            .returns_once(Ok(None));

        let message_handler = MessageHandlerImpl::new(
            Box::new(message_router),
            Box::new(UserServiceMock::new()),
            Box::new(ProductServiceMock::new()),
            Box::new(TransactionServiceMock::new()),
            Box::new(BalanceServiceMock::new()),
            Box::new(CurrencyFormatterMock::new()),
        );

        let responses = message_handler.handle_message(&Message {
            sender: User {
                id: "some id".to_string(),
                name: "foo".to_string(),
            },
            contents: "bar".to_string(),
        });
        assert_eq!(
            vec![Response {
                contents: "Invalid input".to_string()
            }],
            responses
        );
    }

    #[test]
    fn list_command() {
        let mut message_router = MessageRouterMock::new();
        message_router
            .expect_route_message(|arg| arg.any())
            .returns_once(Ok(Some(MessageAction::Command(
                Command::ListAvailableItems,
            ))));

        let mut currency_formatter = CurrencyFormatterMock::new();
        currency_formatter
            .expect_format_amount(|arg| arg.partial_eq(420))
            .returns_once("4.20".to_string());
        currency_formatter
            .expect_format_amount(|arg| arg.partial_eq(50))
            .returns_once("0.50".to_string());

        let mut product_service = ProductServiceMock::new();
        product_service
            .expect_get_available_products()
            .returns_once(Ok(vec![
                Product {
                    identifier: "coke".to_string(),
                    name: "a coke".to_string(),
                    price: 420,
                },
                Product {
                    identifier: "energy".to_string(),
                    name: "energy drink".to_string(),
                    price: 50,
                },
            ]));

        let message_handler = MessageHandlerImpl::new(
            Box::new(message_router),
            Box::new(UserServiceMock::new()),
            Box::new(product_service),
            Box::new(TransactionServiceMock::new()),
            Box::new(BalanceServiceMock::new()),
            Box::new(currency_formatter),
        );

        let responses = message_handler.handle_message(&message_mock());
        assert_eq!(
            vec![Response {
                contents:
                    "Available products:\n/coke - a coke (4.20)\n/energy - energy drink (0.50)"
                        .to_string()
            }],
            responses
        );
    }
}
