use crate::currency_parser::CurrencyParser;
use crate::{Command, Message, MessageAction, Product};

trait MessageRouter {
    fn route_message(&self, message: &Message) -> Result<MessageAction, ()>;
}

pub struct MessageRouterImpl<'a> {
    available_commands: Vec<Command>,
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
        let available_commands = vec![];
        let available_products = vec![];

        let mut currency_parser = CurrencyParserMock::new();
        currency_parser
            .expect_parse_text(|arg| arg.partial_eq("Foo"))
            .times(..=1);

        let message = Message {
            sender: Person {
                id: 0,
                name: "Test".to_string(),
            },
            contents: "Foo".to_string(),
        };

        let router = MessageRouterImpl {
            available_commands,
            available_products,
            currency_parser: Box::new(currency_parser),
        };

        router.route_message(&message).err().unwrap();
    }
}
