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
