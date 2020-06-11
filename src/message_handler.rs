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
use crate::services::balance_service::BalanceService;
use crate::services::product_service::ProductService;
use crate::services::transaction_service::TransactionService;
use crate::services::user_service::UserService;

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
    ) -> Result<Vec<Response>, ()> {
        self.user_service.update_user(sender)?;

        let response = match &message_action {
            MessageAction::Command(command) => self.handle_command(command, sender),
            MessageAction::Product(product) => self.handle_product(product, sender),
            MessageAction::Amount(amount) => self.handle_amount(*amount, sender),
        }?;

        if matches!(message_action, MessageAction::Product(_) | MessageAction::Amount(_)) {
            let balances = self.balance_service.get_balances()?;
            let formatted_balances = self.format_balances(&balances, sender);

            Ok(vec![
                response,
                Response {
                    contents: formatted_balances,
                },
            ])
        } else {
            Ok(vec![response])
        }
    }

    fn handle_command(&self, command: &Command, sender: &User) -> Result<Response, ()> {
        let contents = match command {
            Command::ListAvailableItems => {
                let products = self.product_service.get_available_products()?;
                self.format_products(&products)
            }
            Command::GetCurrentBalances => {
                let balances = self.balance_service.get_balances()?;
                self.format_balances(&balances, sender)
            }
        };

        Ok(Response { contents })
    }

    fn handle_product(&self, product: &Product, sender: &User) -> Result<Response, ()> {
        self.transaction_service
            .register_product_transaction(product, sender)?;

        Ok(Response {
            contents: format!(
                "Recorded {} ({})",
                product.name,
                self.currency_formatter.format_amount(product.price)
            ),
        })
    }

    fn handle_amount(&self, amount: Rappen, sender: &User) -> Result<Response, ()> {
        self.transaction_service
            .register_amount_transaction(amount, sender)?;

        Ok(Response {
            contents: format!("Recorded {}", amount),
        })
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

    fn format_balances(&self, balances: &[Balance], sender: &User) -> String {
        let message_header = "Current stats:";
        let message_body = balances
            .iter()
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
                .unwrap_or_else(|_| {
                    vec![Response {
                        contents: "Internal error (4)".to_string(),
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

        let user = User {
            id: "some id".to_string(),
            name: "foo".to_string(),
        };

        let mut user_service = UserServiceMock::new();
        user_service
            .expect_update_user(|arg| arg.partial_eq_owned(user.clone()))
            .returns_once(Ok(()));

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
            Box::new(user_service),
            Box::new(product_service),
            Box::new(TransactionServiceMock::new()),
            Box::new(BalanceServiceMock::new()),
            Box::new(currency_formatter),
        );

        let responses = message_handler.handle_message(&Message {
            sender: user,
            contents: "bar".to_string(),
        });

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
