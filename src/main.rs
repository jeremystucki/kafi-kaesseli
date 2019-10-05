#![allow(dead_code)]

#[macro_use]
extern crate diesel;

mod currency_formatter;
mod currency_parser;
mod message_router;

mod models;
mod schema;

mod data_provider;

#[derive(Debug, PartialEq)]
struct Person {
    id: usize,
    name: String,
}

#[derive(Debug, PartialEq)]
struct Message {
    sender: Person,
    contents: String,
}

type Rappen = i32;

#[derive(Debug, PartialEq)]
enum Command {
    GetCurrentStats,
    ListAvailableItems,
}

#[derive(Debug, PartialEq)]
struct Product {
    identifier: String,
    name: String,
    price: Rappen,
}

#[derive(Debug, PartialEq)]
enum MessageAction<'a> {
    Amount(Rappen),
    Command(Command),
    Product(&'a Product),
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use currency_formatter::CurrencyFormatter;
    use currency_parser::CurrencyParser;

    fn format_and_parse(amount: Rappen) {
        let formatted_amount = currency_formatter::CurrencyFormatterImpl {}.format_amount(amount);

        let parser = currency_parser::CurrencyParserImpl {};
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
