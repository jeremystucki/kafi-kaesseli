#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use crate::models::{Product, User};

mod currency_handling;

mod message_handler;
mod message_router;

mod models;
mod schema;

mod data_loader;

mod services;

#[cfg(test)]
mod test_utils;

#[derive(Debug, PartialEq)]
pub struct Message {
    sender: User,
    contents: String,
}

#[derive(Debug, PartialEq)]
pub struct Response {
    contents: String,
}

pub type Rappen = i32;

#[derive(Debug, PartialEq)]
pub enum Command {
    GetCurrentStats,
    ListAvailableItems,
}

#[derive(Debug, PartialEq)]
pub enum MessageAction {
    Amount(Rappen),
    Command(Command),
    Product(Product),
}

#[cfg(test)]
mod tests {
    use super::*;
    use currency_formatter::CurrencyFormatter;
    use currency_handling::*;
    use currency_parser::CurrencyParser;

    fn format_and_parse(amount: Rappen) {
        let formatted_amount =
            currency_formatter::CurrencyFormatterImpl::new().format_amount(amount);

        let parser = currency_parser::CurrencyParserImpl::new();
        let parsed_amount = parser.parse_text(&formatted_amount).unwrap();

        assert_eq!(amount, parsed_amount);
    }

    #[test]
    fn parses_and_formats_zero() {
        format_and_parse(0)
    }

    #[test]
    fn parses_and_formats_negative_rappen_amount() {
        format_and_parse(-50)
    }
}
