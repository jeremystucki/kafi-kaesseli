#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use crate::models::{Product, User};

mod currency_handling;

pub mod message_handler;
mod message_router;

mod models;
mod schema;

pub mod data_loader;

mod services;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::Rappen;

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
