use std::cmp::Ordering;
use std::str::FromStr;

use icu_collator::{
    Collator, CollatorBorrowed,
    options::{CollatorOptions, Strength},
};
use rust_decimal::Decimal;

/// Root Unicode collation corresponds to C#'s invariant-culture comparison.
/// `ordinal` in the original CLI only turns off natural numbers; it does not
/// change the underlying invariant-culture string comparison.
pub(crate) struct Comparison {
    collator: CollatorBorrowed<'static>,
    natural: bool,
}

impl Comparison {
    pub fn new(ignore_case: bool, natural: bool) -> Self {
        let mut options = CollatorOptions::default();
        options.strength = Some(if ignore_case {
            Strength::Secondary
        } else {
            Strength::Tertiary
        });
        Self {
            collator: Collator::try_new(Default::default(), options)
                .expect("the compiled ICU root collation data must be available"),
            natural,
        }
    }

    pub fn compare(&self, a: &str, b: &str) -> Ordering {
        let lexical = self.collator.compare(a, b);
        if !self.natural || lexical == Ordering::Equal {
            return lexical;
        }
        if let (Some(a), Some(b)) = (number(a), number(b)) {
            return a.cmp(&b);
        }
        if let (Some((ap, an)), Some((bp, bn))) = (suffix(a), suffix(b))
            && self.collator.compare(ap, bp) == Ordering::Equal
        {
            return an.cmp(&bn);
        }
        lexical
    }
}

fn number(text: &str) -> Option<Decimal> {
    // Decimal.TryParse uses NumberStyles.Number: grouping and a trailing sign
    // are allowed, but exponents are not. Use exact decimals rather than floats.
    let text = text.trim();
    if text.contains(['e', 'E', '_']) {
        return None;
    }
    let mut cleaned = text.replace(',', "");
    if cleaned.ends_with(['-', '+']) {
        let sign = cleaned.pop()?;
        cleaned.insert(0, sign);
    }
    Decimal::from_str(&cleaned).ok()
}

fn suffix(text: &str) -> Option<(&str, Decimal)> {
    let mut start = text.len();
    let mut dot = false;
    for (i, c) in text.char_indices().rev() {
        if c.is_ascii_digit() || (start < text.len() && c == '.' && !dot) {
            dot |= c == '.';
            start = i;
        } else {
            break;
        }
    }
    if start == text.len() {
        return None;
    }
    let prefix = &text[..start];
    let mut value = number(&text[start..])?;
    // Preserve the C# suffix rule: '-' remains part of the text prefix.
    if prefix.ends_with('-') && value > Decimal::ZERO {
        value = -value;
    }
    Some((prefix, value))
}
