use crate::currency_parser::CurrencyParser;
use crate::product_data_source::ProductDataSource;
use crate::{Command, Message, MessageAction, Product};

#[cfg(test)]
use mockiato::mockable;

#[cfg_attr(test, mockable)]
pub trait MessageRouter {
    fn route_message(&self, message: &Message) -> Result<Option<MessageAction>, ()>;
}

pub struct MessageRouterImpl {
    product_data_source: Box<dyn ProductDataSource>,
    currency_parser: Box<dyn CurrencyParser>,
}

impl MessageRouterImpl {
    fn get_command(&self, message: &Message) -> Option<Command> {
        match message.contents.as_ref() {
            "/list" => Some(Command::ListAvailableItems),
            "/stats" => Some(Command::GetCurrentStats),
            _ => None,
        }
    }

    fn get_product(&self, message: &Message) -> Result<Option<Product>, ()> {
        let product_identifier = message.contents.trim_start_matches('/');

        self.product_data_source
            .get_product_with_identifier(product_identifier)
    }
}

impl MessageRouter for MessageRouterImpl {
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
    use super::*;
    use crate::currency_parser::CurrencyParserMock;
    use crate::product_data_source::ProductDataSourceMock;
    use crate::Person;

    #[test]
    fn unknown_message() {
        let mut product_data_source = ProductDataSourceMock::new();
        product_data_source
            .expect_get_product_with_identifier(|arg| arg.partial_eq("Foo"))
            .times(1)
            .returns(Ok(None));

        let mut currency_parser = CurrencyParserMock::new();
        currency_parser
            .expect_parse_text(|arg| arg.partial_eq("Foo"))
            .times(1)
            .returns(Err(()));

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "Foo".to_string(),
        };

        let router = MessageRouterImpl {
            product_data_source: Box::new(product_data_source),
            currency_parser: Box::new(currency_parser),
        };

        let action = router.route_message(&message).unwrap();
        assert_eq!(None, action);
    }

    #[test]
    fn stats_command() {
        let product_data_source = ProductDataSourceMock::new();

        let currency_parser = CurrencyParserMock::new();

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "/stats".to_string(),
        };

        let router = MessageRouterImpl {
            product_data_source: Box::new(product_data_source),
            currency_parser: Box::new(currency_parser),
        };

        let action = router.route_message(&message).unwrap();
        assert_eq!(
            Some(MessageAction::Command(Command::GetCurrentStats)),
            action
        );
    }

    #[test]
    fn list_available_items_command() {
        let product_data_source = ProductDataSourceMock::new();

        let currency_parser = CurrencyParserMock::new();

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "/list".to_string(),
        };

        let router = MessageRouterImpl {
            product_data_source: Box::new(product_data_source),
            currency_parser: Box::new(currency_parser),
        };

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

        let mut product_data_source = ProductDataSourceMock::new();
        product_data_source
            .expect_get_product_with_identifier(|arg| arg.partial_eq("foo"))
            .times(1)
            .returns(Ok(Some(product.clone())));

        let currency_parser = CurrencyParserMock::new();

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "/foo".to_string(),
        };

        let router = MessageRouterImpl {
            product_data_source: Box::new(product_data_source),
            currency_parser: Box::new(currency_parser),
        };

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

        let mut product_data_source = ProductDataSourceMock::new();
        product_data_source
            .expect_get_product_with_identifier(|arg| arg.partial_eq("foo"))
            .times(1)
            .returns(Ok(Some(product.clone())));

        let currency_parser = CurrencyParserMock::new();

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "foo".to_string(),
        };

        let router = MessageRouterImpl {
            product_data_source: Box::new(product_data_source),
            currency_parser: Box::new(currency_parser),
        };

        let action = router.route_message(&message).unwrap();
        assert_eq!(Some(MessageAction::Product(product)), action);
    }

    #[test]
    fn amount() {
        let mut product_data_source = ProductDataSourceMock::new();
        product_data_source
            .expect_get_product_with_identifier(|arg| arg.partial_eq("1.20"))
            .times(1)
            .returns(Ok(None));

        let mut currency_parser = CurrencyParserMock::new();
        currency_parser
            .expect_parse_text(|arg| arg.partial_eq("1.20"))
            .times(1)
            .returns(Ok(120));

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "1.20".to_string(),
        };

        let router = MessageRouterImpl {
            product_data_source: Box::new(product_data_source),
            currency_parser: Box::new(currency_parser),
        };

        let action = router.route_message(&message).unwrap();
        assert_eq!(Some(MessageAction::Amount(120)), action);
    }

    #[test]
    fn error_in_product_data_source() {
        let mut product_data_source = ProductDataSourceMock::new();
        product_data_source
            .expect_get_product_with_identifier(|arg| arg.partial_eq("1.20"))
            .times(1)
            .returns(Err(()));

        let currency_parser = CurrencyParserMock::new();

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "1.20".to_string(),
        };

        let router = MessageRouterImpl {
            product_data_source: Box::new(product_data_source),
            currency_parser: Box::new(currency_parser),
        };

        router.route_message(&message).unwrap_err();
    }
}
