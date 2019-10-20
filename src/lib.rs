#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use crate::models::{Product, User};

pub mod currency_handling;

pub mod message_handler;
pub mod message_router;

pub mod models;
mod schema;

pub mod data_loader;

pub mod services;

#[cfg(test)]
mod test_utils;

embed_migrations!("migrations");

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::Rappen;

    use currency_formatter::CurrencyFormatter;
    use currency_handling::*;
    use currency_parser::CurrencyParser;

    fn format_and_parse(amount: Rappen) {
        let formatted_amount =
            currency_formatter::CurrencyFormatterImpl::default().format_amount(amount);

        let parser = currency_parser::CurrencyParserImpl::default();
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
