use crate::Rappen;
use std::ops::Add;

pub(crate) struct CurrencyFormatter {}

impl CurrencyFormatter {
    pub(crate) fn format_amount(&self, amount: Rappen) -> String {
        let prefix = if amount >= 0 { "" } else { "- " };
        let formatted_franken = format_whole_franken_amount(amount);
        let formatted_rappen = format_rappen_amount(amount);

        format!("{}{}.{}", prefix, formatted_franken, formatted_rappen)
    }
}

fn format_whole_franken_amount(amount: Rappen) -> String {
    let franken_amount = amount.abs() / 100;

    // Only return "-" if the rappen amount is not zero
    if amount != 0 && franken_amount == 0 {
        String::from("-")
    } else {
        franken_amount.to_string()
    }
}

fn format_rappen_amount(amount: Rappen) -> String {
    let rappen_amount = amount.abs() % 100;

    match rappen_amount {
        0 => String::from("-"),
        1..9 => format!("0{}", rappen_amount),
        _ => rappen_amount.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_zero_correctly() {
        let expected = String::from("0.-");

        let formatter = CurrencyFormatter {};
        let actual = formatter.format_amount(0);

        assert_eq!(expected, actual);
    }

    #[test]
    fn formats_negative_one_hundred_correctly() {
        let expected = String::from("- 1.-");

        let formatter = CurrencyFormatter {};
        let actual = formatter.format_amount(-100);

        assert_eq!(expected, actual);
    }

    #[test]
    fn formats_negative_one_hundred_and_one_correctly() {
        let expected = String::from("- 1.01");

        let formatter = CurrencyFormatter {};
        let actual = formatter.format_amount(-101);

        assert_eq!(expected, actual);
    }

    #[test]
    fn formats_negative_ninety_nine_correctly() {
        let expected = String::from("- -.99");

        let formatter = CurrencyFormatter {};
        let actual = formatter.format_amount(-99);

        assert_eq!(expected, actual);
    }

    #[test]
    fn formats_one_hundred_correctly() {
        let expected = String::from("1.-");

        let formatter = CurrencyFormatter {};
        let actual = formatter.format_amount(100);

        assert_eq!(expected, actual);
    }

    #[test]
    fn formats_one_hundred_and_one_correctly() {
        let expected = String::from("1.01");

        let formatter = CurrencyFormatter {};
        let actual = formatter.format_amount(101);

        assert_eq!(expected, actual);
    }

    #[test]
    fn formats_ninety_nine_correctly() {
        let expected = String::from("-.99");

        let formatter = CurrencyFormatter {};
        let actual = formatter.format_amount(99);

        assert_eq!(expected, actual);
    }
}
