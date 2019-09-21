extern crate nom;

use self::nom::bytes::complete::{tag, take_while1, take_while_m_n};
use self::nom::combinator::value;
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
        let sign = map(opt(nom_char('-')), |option| option.is_none());

        let franken_amount = alt::<_, _, (), _>((
            map(nom_char('-'), |_| 0),
            map(digit1, |string: &str| string.parse::<u16>().unwrap()),
        ));

        let separator = alt((nom_char('.'), nom_char(',')));

        let rappen_amount = map(
            opt(preceded(
                separator,
                alt((
                    map(nom_char('-'), |_| 0),
                    // TODO: is_char_digit is weird
                    map(take_while_m_n(1, 2, is_char_digit), |digits: &str| {
                        // TODO: Does not need to be u16
                        let parsed = digits.parse::<u16>().unwrap();

                        if digits.len() == 1 {
                            parsed * 10
                        } else {
                            parsed
                        }
                    }),
                )),
            )),
            |option| option.unwrap_or(0),
        );

        let parser = tuple((
            map(opt(tuple((sign, franken_amount))), |option| {
                option.unwrap_or((true, 0))
            }),
            rappen_amount,
        ));

        let result = parser(&text);

        dbg!(&result);

        if let Ok((remaining_text, ((is_positive, franken_amount), rappen_amount))) = result {
            if !remaining_text.is_empty() {
                return None;
            }

            let total_amount: Rappen = (rappen_amount + franken_amount * 100).into();

            if is_positive {
                Some(total_amount)
            } else {
                Some(-total_amount)
            }
        } else {
            None
        }
    }
}

pub fn is_char_digit(chr: char) -> bool {
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
    fn rappen_only() {
        test_parser(".5", Some(50))
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
