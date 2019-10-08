use crate::currency_formatter::CurrencyFormatter;
use crate::message_router::MessageRouter;
use crate::{Message, Response};
use diesel::SqliteConnection;

pub trait MessageHandler {
    fn handle_message(&self, message: Message) -> Vec<Response>;
}

pub struct MessageHandlerImpl<'a> {
    database_connection: &'a SqliteConnection,
    message_router: Box<dyn MessageRouter>,
    currency_formatter: Box<dyn CurrencyFormatter>,
}

impl MessageHandler for MessageHandlerImpl<'_> {
    fn handle_message(&self, message: Message) -> Vec<Response> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency_formatter::CurrencyFormatterMock;
    use crate::message_router::MessageRouterMock;
    use crate::{Person, Response};
    use diesel::Connection;

    #[test]
    fn invalid_message() {
        let message = Message {
            sender: Person {
                id: 1,
                name: "foo".to_string(),
            },
            contents: "bar".to_string(),
        };

        let database_connection = SqliteConnection::establish(":memory:").unwrap();

        let mut message_router = MessageRouterMock::new();
        message_router
            .expect_route_message(|arg| arg.partial_eq(&message))
            .returns(None);

        let currency_formatter = CurrencyFormatterMock::new();

        let message_handler = MessageHandlerImpl {
            database_connection: &database_connection,
            message_router: Box::new(message_router),
            currency_formatter: Box::new(currency_formatter),
        };

        let responses = message_handler.handle_message(message);
        assert_eq!(
            vec![Response {
                contents: "Invalid input".to_string()
            }],
            responses
        );
    }
}
