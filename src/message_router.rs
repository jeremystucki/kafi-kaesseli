use crate::currency_parser::CurrencyParser;
use crate::{Command, Message, MessageAction, Product};

trait MessageRouter {
    fn route_message(&self, message: &Message) -> Result<MessageAction, ()>;
}

pub struct MessageRouterImpl<'a> {
    available_products: Vec<&'a Product>,
    currency_parser: Box<dyn CurrencyParser>,
}

impl MessageRouterImpl<'_> {
    fn get_command(&self, message: &Message) -> Option<Command> {
        match message.contents.as_ref() {
            "/list" => Some(Command::ListAvailableItems),
            "/stats" => Some(Command::GetCurrentStats),
            _ => None,
        }
    }

    fn get_product(&self, message: &Message) -> Option<&Product> {
        let product_identifier = message.contents.trim_start_matches("/");

        self.available_products
            .iter()
            .cloned()
            .find(|&product| product.identifier == product_identifier)
    }
}

impl MessageRouter for MessageRouterImpl<'_> {
    fn route_message(&self, message: &Message) -> Result<MessageAction<'_>, ()> {
        None.or_else(|| self.get_command(message).map(MessageAction::Command))
            .or_else(|| self.get_product(message).map(MessageAction::Product))
            .map_or_else(
                || {
                    self.currency_parser
                        .parse_text(&message.contents)
                        .map(MessageAction::Amount)
                },
                Result::Ok,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency_parser::CurrencyParserMock;
    use crate::Person;

    #[test]
    fn route_unknown_message() {
        let available_products = vec![];

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
            available_products,
            currency_parser: Box::new(currency_parser),
        };

        let action = router.route_message(&message);
        assert_eq!(Err(()), action);
    }

    #[test]
    fn route_stats_command() {
        let available_products = vec![];

        let currency_parser = CurrencyParserMock::new();

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "/stats".to_string(),
        };

        let router = MessageRouterImpl {
            available_products,
            currency_parser: Box::new(currency_parser),
        };

        let action = router.route_message(&message).unwrap();
        assert_eq!(MessageAction::Command(Command::GetCurrentStats), action);
    }

    #[test]
    fn route_list_available_items_command() {
        let available_products = vec![];

        let currency_parser = CurrencyParserMock::new();

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "/list".to_string(),
        };

        let router = MessageRouterImpl {
            available_products,
            currency_parser: Box::new(currency_parser),
        };

        let action = router.route_message(&message).unwrap();
        assert_eq!(MessageAction::Command(Command::ListAvailableItems), action);
    }

    #[test]
    fn route_known_product() {
        let foo = Product {
            identifier: "foo".to_string(),
            name: "test product".to_string(),
            price: 60,
        };

        let bar = Product {
            identifier: "bar".to_string(),
            name: "test product".to_string(),
            price: 120,
        };

        let available_products = vec![&foo, &bar];

        let currency_parser = CurrencyParserMock::new();

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "/foo".to_string(),
        };

        let router = MessageRouterImpl {
            available_products,
            currency_parser: Box::new(currency_parser),
        };

        let action = router.route_message(&message).unwrap();
        assert_eq!(MessageAction::Product(&foo), action);
    }

    #[test]
    fn route_known_product_without_slash() {
        let foo = Product {
            identifier: "foo".to_string(),
            name: "test product".to_string(),
            price: 60,
        };

        let available_products = vec![&foo];

        let currency_parser = CurrencyParserMock::new();

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "foo".to_string(),
        };

        let router = MessageRouterImpl {
            available_products,
            currency_parser: Box::new(currency_parser),
        };

        let action = router.route_message(&message).unwrap();
        assert_eq!(MessageAction::Product(&foo), action);
    }
}
