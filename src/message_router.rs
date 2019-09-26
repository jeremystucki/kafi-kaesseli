use crate::currency_parser::CurrencyParser;
use crate::{Command, Message, MessageAction, Product};

trait MessageRouter {
    fn route_message(&self, message: &Message) -> Result<MessageAction, ()>;
}

pub struct MessageRouterImpl<'a> {
    available_products: Vec<&'a Product>,
    currency_parser: Box<dyn CurrencyParser>,
}

impl MessageRouter for MessageRouterImpl<'_> {
    fn route_message(&self, message: &Message) -> Result<MessageAction<'_>, ()> {
        unimplemented!()
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
            .times(1);

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

        router.route_message(&message).err().unwrap();
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
        assert_eq!(MessageAction::Command(Command::GetCurrentStats));
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
        assert_eq!(MessageAction::Command(Command::ListAvailableItems));
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
        assert_eq!(MessageAction::Product(&foo));
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
        assert_eq!(MessageAction::Product(&foo));
    }
}
