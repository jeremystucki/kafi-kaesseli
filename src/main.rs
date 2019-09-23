#![feature(exclusive_range_pattern)]

mod currency_formatter;
mod currency_parser;
mod message_handler;

type Rappen = i32;

enum MessageValidity {
    Valid(MessageType),
    Invalid,
}

enum MessageType {
    Command(String),
    Amount(Rappen),
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_and_parse(amount: Rappen) {
        let formatted_amount = currency_formatter::format_amount(amount);

        let parser = currency_parser::CurrencyParser {};
        let parsed_amount = parser.parse_text(formatted_amount).unwrap();

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
