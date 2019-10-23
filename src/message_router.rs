#[cfg(test)]
use mockall::automock;

use crate::currency_handling::currency_parser::CurrencyParser;
use crate::models::{Command, Message, MessageAction, Product};
use crate::services::product_service::ProductService;

#[cfg_attr(test, automock)]
pub trait MessageRouter: Send {
    fn route_message(&self, message: &Message) -> Result<Option<MessageAction>, ()>;
}

pub struct MessageRouterImpl<'a> {
    product_service: Box<dyn ProductService + 'a>,
    currency_parser: Box<dyn CurrencyParser + 'a>,
}

impl<'a> MessageRouterImpl<'a> {
    pub fn new(
        product_service: Box<dyn ProductService + 'a>,
        currency_parser: Box<dyn CurrencyParser + 'a>,
    ) -> Self {
        Self {
            product_service,
            currency_parser,
        }
    }

    fn get_command(&self, message: &Message) -> Option<Command> {
        match message.contents.as_ref() {
            "/list" => Some(Command::ListAvailableItems),
            "/stats" => Some(Command::GetCurrentStats),
            _ => None,
        }
    }

    fn get_product(&self, message: &Message) -> Result<Option<Product>, ()> {
        let product_identifier = message.contents.trim_start_matches('/');

        self.product_service
            .get_product_with_identifier(product_identifier)
    }
}

impl<'a> MessageRouter for MessageRouterImpl<'a> {
    fn route_message(&self, message: &Message) -> Result<Option<MessageAction>, ()> {
        if let Some(command) = self.get_command(message) {
            return Ok(Some(MessageAction::Command(command)));
        }

        if let Some(product) = self.get_product(message)? {
            return Ok(Some(MessageAction::Product(product)));
        }

        if let Ok(amount) = self.currency_parser.parse_text(&message.contents) {
            return Ok(Some(MessageAction::Amount(amount)));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::currency_handling::currency_parser::MockCurrencyParser;
    use crate::services::product_service::MockProductService;
    use crate::User;

    use super::*;
    use mockall::predicate::eq;

    #[test]
    fn unknown_message() {
        let mut product_service = MockProductService::new();
        product_service
            .expect_get_product_with_identifier()
            .with(eq("Foo"))
            .times(1)
            .returning(|_| Ok(None));

        let mut currency_parser = MockCurrencyParser::new();
        currency_parser
            .expect_parse_text()
            .with(eq("Foo"))
            .times(1)
            .returning(|_| Err(()));

        let message = Message {
            sender: User {
                id: "some id".to_string(),
                name: "Test".to_string(),
            },
            contents: "Foo".to_string(),
        };

        let router = MessageRouterImpl::new(Box::new(product_service), Box::new(currency_parser));

        let action = router.route_message(&message).unwrap();
        assert_eq!(None, action);
    }

    #[test]
    fn stats_command() {
        let product_service = MockProductService::new();

        let currency_parser = MockCurrencyParser::new();

        let message = Message {
            sender: User {
                id: "some id".to_string(),
                name: "Test".to_string(),
            },
            contents: "/stats".to_string(),
        };

        let router = MessageRouterImpl::new(Box::new(product_service), Box::new(currency_parser));

        let action = router.route_message(&message).unwrap();
        assert_eq!(
            Some(MessageAction::Command(Command::GetCurrentStats)),
            action
        );
    }

    #[test]
    fn list_available_items_command() {
        let product_service = MockProductService::new();

        let currency_parser = MockCurrencyParser::new();

        let message = Message {
            sender: User {
                id: "some id".to_string(),
                name: "Test".to_string(),
            },
            contents: "/list".to_string(),
        };

        let router = MessageRouterImpl::new(Box::new(product_service), Box::new(currency_parser));

        let action = router.route_message(&message).unwrap();
        assert_eq!(
            Some(MessageAction::Command(Command::ListAvailableItems)),
            action
        );
    }

    #[test]
    fn known_product() {
        let product = Product {
            identifier: "foo".to_string(),
            name: "test product".to_string(),
            price: 60,
        };

        let product_clone = product.clone();

        let mut product_service = MockProductService::new();
        product_service
            .expect_get_product_with_identifier()
            .with(eq("foo"))
            .times(1)
            .return_once(move |_| Ok(Some(product_clone)));

        let currency_parser = MockCurrencyParser::new();

        let message = Message {
            sender: User {
                id: "some id".to_string(),
                name: "Test".to_string(),
            },
            contents: "/foo".to_string(),
        };

        let router = MessageRouterImpl::new(Box::new(product_service), Box::new(currency_parser));

        let action = router.route_message(&message).unwrap();
        assert_eq!(Some(MessageAction::Product(product)), action);
    }

    #[test]
    fn known_product_without_slash() {
        let product = Product {
            identifier: "foo".to_string(),
            name: "test product".to_string(),
            price: 60,
        };

        let product_clone = product.clone();

        let mut product_service = MockProductService::new();
        product_service
            .expect_get_product_with_identifier()
            .with(eq("foo"))
            .times(1)
            .return_once(|_| Ok(Some(product_clone)));

        let currency_parser = MockCurrencyParser::new();

        let message = Message {
            sender: User {
                id: "some id".to_string(),
                name: "Test".to_string(),
            },
            contents: "foo".to_string(),
        };

        let router = MessageRouterImpl::new(Box::new(product_service), Box::new(currency_parser));

        let action = router.route_message(&message).unwrap();
        assert_eq!(Some(MessageAction::Product(product)), action);
    }

    #[test]
    fn amount() {
        let mut product_service = MockProductService::new();
        product_service
            .expect_get_product_with_identifier()
            .with(eq("1.20"))
            .times(1)
            .returning(|_| Ok(None));

        let mut currency_parser = MockCurrencyParser::new();
        currency_parser
            .expect_parse_text()
            .with(eq("1.20"))
            .times(1)
            .returning(|_| Ok(120));

        let message = Message {
            sender: User {
                id: "some id".to_string(),
                name: "Test".to_string(),
            },
            contents: "1.20".to_string(),
        };

        let router = MessageRouterImpl::new(Box::new(product_service), Box::new(currency_parser));

        let action = router.route_message(&message).unwrap();
        assert_eq!(Some(MessageAction::Amount(120)), action);
    }

    #[test]
    fn error_in_product_service() {
        let mut product_service = MockProductService::new();
        product_service
            .expect_get_product_with_identifier()
            .with(eq("1.20"))
            .times(1)
            .returning(|_| Err(()));

        let currency_parser = MockCurrencyParser::new();

        let message = Message {
            sender: User {
                id: "some id".to_string(),
                name: "Test".to_string(),
            },
            contents: "1.20".to_string(),
        };

        let router = MessageRouterImpl::new(Box::new(product_service), Box::new(currency_parser));

        router.route_message(&message).unwrap_err();
    }
}
