use crate::currency_formatter::CurrencyFormatter;
use crate::message_router::MessageRouter;
use crate::models::Product;
use crate::schema::products;
use crate::{Command, Message, MessageAction, Response};
use diesel::{RunQueryDsl, SqliteConnection};

pub trait MessageHandler {
    fn handle_message(&self, message: Message) -> Vec<Response>;
}

pub struct MessageHandlerImpl<'a> {
    database_connection: &'a SqliteConnection,
    message_router: Box<dyn MessageRouter>,
    currency_formatter: Box<dyn CurrencyFormatter>,
}

impl MessageHandlerImpl<'_> {
    fn handle_message_action(&self, message_action: MessageAction) -> Vec<Response> {
        match message_action {
            MessageAction::Command(command) => self.handle_command(command),
            MessageAction::Product(product) => unimplemented!(),
            MessageAction::Amount(amount) => unimplemented!(),
        }
    }

    fn handle_command(&self, command: Command) -> Vec<Response> {
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
            Command::GetCurrentStats => unimplemented!(),
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
}

impl MessageHandler for MessageHandlerImpl<'_> {
    fn handle_message(&self, message: Message) -> Vec<Response> {
        match self.message_router.route_message(&message) {
            Err(_) => vec![Response {
                contents: "Internal error (1)".to_string(),
            }],
            Ok(None) => vec![Response {
                contents: "Invalid input".to_string(),
            }],
            Ok(Some(message_action)) => self.handle_message_action(message_action),
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
                id: 1,
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

        let responses = message_handler.handle_message(message_mock());
        assert_eq!(
            vec![Response {
                contents: "Invalid input".to_string()
            }],
            responses
        );
    }
}
