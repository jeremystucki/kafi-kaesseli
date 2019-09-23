use crate::currency_parser::CurrencyParser;

pub(crate) struct MessageHandler {
    available_commands: Vec<String>,
    currency_parser: Box<dyn CurrencyParser>,
}
