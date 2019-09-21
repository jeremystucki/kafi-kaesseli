use crate::currency_parser::CurrencyParser;
use crate::MessageValidity;

pub(crate) struct MessageHandler {
    available_commands: Vec<String>,
    currency_parser: CurrencyParser,
}
