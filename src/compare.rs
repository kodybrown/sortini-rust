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
        if !self.natural {
            return self.collator.compare(a, b);
        }
        // Pair-dependent fallback to whole-string collation creates cycles:
        // -2 < -1 numerically, but -1 < -1a < -2 lexically. Every label must
        // receive the same key regardless of its comparison partner.
        let (ap, an) = natural_key(a);
        let (bp, bn) = natural_key(b);
        self.collator.compare(ap, bp).then_with(|| an.cmp(&bn))
    }
}

fn natural_key(text: &str) -> (&str, Option<Decimal>) {
    if let Some(value) = number(text) {
        ("", Some(value))
    } else if let Some((prefix, value)) = suffix(text) {
        (prefix, Some(value))
    } else {
        (text, None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{prelude::*, test_runner::RngSeed};

    fn label() -> impl Strategy<Value = String> {
        prop_oneof![
            "[a-zA-Z0-9éΩ._+-]{0,12}",
            "[a-zA-ZéΩ._+-]{0,5}[0-9]{0,4}",
            (-10000i32..10000).prop_map(|n| n.to_string()),
            (-10000i32..10000, 0u32..1000).prop_map(|(n, f)| format!("{n}.{f}")),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 1024, rng_seed: RngSeed::Fixed(0x5074696e69), ..ProptestConfig::default() })]

        #[test]
        fn comparison_is_reflexive_and_antisymmetric(a in label(), b in label(), ignore_case in any::<bool>(), natural in any::<bool>()) {
            let comparison = Comparison::new(ignore_case, natural);
            prop_assert_eq!(comparison.compare(&a, &a), Ordering::Equal);
            prop_assert_eq!(comparison.compare(&a, &b), comparison.compare(&b, &a).reverse());
        }

        #[test]
        fn comparison_is_transitive(a in label(), b in label(), c in label(), ignore_case in any::<bool>(), natural in any::<bool>()) {
            let comparison = Comparison::new(ignore_case, natural);
            let ab = comparison.compare(&a, &b);
            let bc = comparison.compare(&b, &c);
            if ab != Ordering::Greater && bc != Ordering::Greater {
                prop_assert_ne!(comparison.compare(&a, &c), Ordering::Greater, "a={:?}, b={:?}, c={:?}", a, b, c);
            }
        }
    }

    #[test]
    fn numeric_and_text_labels_have_a_consistent_order() {
        let comparison = Comparison::new(true, true);
        for labels in [["-2", "-1", "-1a"], ["item2", "item10", "item1x"]] {
            let [a, b, c] = labels;
            assert!(
                !(comparison.compare(a, b).is_lt()
                    && comparison.compare(b, c).is_lt()
                    && comparison.compare(c, a).is_lt()),
                "comparison cycle: {labels:?}"
            );
        }
    }
}
