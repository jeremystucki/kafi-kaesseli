extern crate nom;

use self::nom::bytes::complete::take_while_m_n;
use self::nom::combinator::{not, peek};
use self::nom::sequence::terminated;
use crate::Rappen;
use nom::branch::alt;
use nom::character::complete::{char as nom_char, digit1};
use nom::character::is_digit;
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::tuple;

#[cfg(test)]
use mockiato::mockable;

#[cfg_attr(test, mockable)]
pub trait CurrencyParser {
    fn parse_text(&self, text: &str) -> Result<Rappen, ()>;
}

pub struct CurrencyParserImpl;

impl CurrencyParserImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl CurrencyParser for CurrencyParserImpl {
    fn parse_text(&self, text: &str) -> Result<Rappen, ()> {
        let separator = alt((nom_char('.'), nom_char(',')));

        let sign = map(
            opt(terminated(
                nom_char('-'),
                tuple((not(peek(&separator)), many0(nom_char(' ')))),
            )),
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

        let (remaining_text, parsed_data) = parser(&text).map_err(|_| ())?;

        if !remaining_text.is_empty() {
            return Err(());
        }

        let (is_positive, (franken_amount, rappen_amount)) = parsed_data;

        let value: Rappen = (franken_amount * 100 + rappen_amount).into();
        if is_positive {
            Ok(value)
        } else {
            Ok(-value)
        }
    }
}

fn is_char_digit(chr: char) -> bool {
    chr.is_ascii() && is_digit(chr as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_parser(input: &str, expected: Result<Rappen, ()>) {
        let parser = CurrencyParserImpl::new();
        let actual = parser.parse_text(input);

        assert_eq!(expected, actual);
    }

    #[test]
    fn franken_only() {
        test_parser("2", Ok(200))
    }

    #[test]
    fn dash_for_rappen() {
        test_parser("2.-", Ok(200))
    }

    #[test]
    fn franken_with_zero_rappen() {
        test_parser("2.0", Ok(200))
    }

    #[test]
    fn dash_for_franken() {
        test_parser("-.05", Ok(5))
    }

    #[test]
    fn with_comma() {
        test_parser("1,20", Ok(120))
    }

    #[test]
    fn negative() {
        test_parser("-2.5", Ok(-250))
    }

    #[test]
    fn invalid_rappen_amount() {
        test_parser("2.005", Err(()))
    }

    #[test]
    fn space_after_minus_sign() {
        test_parser("- 1.-", Ok(-100))
    }
}
