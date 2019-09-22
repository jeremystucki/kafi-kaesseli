extern crate nom;

use self::nom::bytes::complete::{tag, take_while1, take_while_m_n};
use self::nom::combinator::{not, peek, value};
use self::nom::sequence::terminated;
use crate::Rappen;
use nom::branch::alt;
use nom::character::complete::{char as nom_char, digit1};
use nom::character::is_digit;
use nom::combinator::{map, opt};
use nom::multi::{many0, many1};
use nom::number::complete::be_u16;
use nom::sequence::{pair, preceded, tuple};

pub(crate) struct CurrencyParser;

impl CurrencyParser {
    pub(crate) fn parse_text(&self, text: String) -> Option<Rappen> {
        let separator = alt((nom_char('.'), nom_char(',')));

        let sign = map(
            opt(terminated(nom_char('-'), not(peek(&separator)))),
            |option| option.is_none(),
        );

        let franken_amount = map(digit1, |digits: &str| digits.parse::<u16>().unwrap());
        let rappen_amount = alt((
            map(take_while_m_n(2, 2, is_char_digit), |digits: &str| {
                digits.parse::<u16>().unwrap()
            }),
            map(take_while_m_n(1, 1, is_char_digit), |digits: &str| {
                digits.parse::<u16>().unwrap() * 10
            }),
        ));

        let dash = nom_char('-');

        let parser = tuple::<_, _, (), _>((
            sign,
            alt((
                map(
                    tuple((&franken_amount, &separator, &rappen_amount)),
                    |(franken_amount, _, rappen_amount)| (franken_amount, rappen_amount),
                ),
                map(
                    tuple((&franken_amount, &separator, &dash)),
                    |(franken_amount, _, _)| (franken_amount, 0),
                ),
                map(
                    tuple((&dash, &separator, &rappen_amount)),
                    |(_, _, rappen_amount)| (0, rappen_amount),
                ),
                map(&franken_amount, |franken_amount| (franken_amount, 0)),
            )),
        ));

        parser(&text)
            .ok()
            .and_then(|(remaining_text, parsed_data)| {
                if remaining_text.is_empty() {
                    Some(parsed_data)
                } else {
                    None
                }
            })
            .map(|(is_positive, (franken_amount, rappen_amount))| {
                let value: Rappen = (franken_amount * 100 + rappen_amount).into();

                if is_positive {
                    value
                } else {
                    -value
                }
            })
    }
}

fn is_char_digit(chr: char) -> bool {
    return chr.is_ascii() && is_digit(chr as u8);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_parser(input: &str, expected: Option<Rappen>) {
        let parser = CurrencyParser {};
        let actual = parser.parse_text(input.to_string());

        assert_eq!(expected, actual);
    }

    #[test]
    fn franken_only() {
        test_parser("2", Some(200))
    }

    #[test]
    fn dash_for_rappen() {
        test_parser("2.-", Some(200))
    }

    #[test]
    fn franken_with_zero_rappen() {
        test_parser("2.0", Some(200))
    }

    #[test]
    fn dash_for_franken() {
        test_parser("-.05", Some(5))
    }

    #[test]
    fn with_comma() {
        test_parser("1,20", Some(120))
    }

    #[test]
    fn negative() {
        test_parser("-2.5", Some(-250))
    }

    #[test]
    fn invalid_rappen_amount() {
        test_parser("2.005", None)
    }
}
