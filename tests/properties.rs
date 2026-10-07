use proptest::{
    prelude::*,
    test_runner::{FileFailurePersistence, RngSeed},
};
use sortini::{
    Options,
    encoding::{EncodedText, Encoding},
    sort_bytes, sort_text,
};
use std::collections::BTreeMap;

fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..160).prop_map(|chars| chars.into_iter().collect())
}

fn encoding() -> impl Strategy<Value = Encoding> {
    prop_oneof![
        any::<bool>().prop_map(|bom| Encoding::Utf8 { bom }),
        (any::<bool>(), any::<bool>())
            .prop_map(|(big_endian, bom)| Encoding::Utf16 { big_endian, bom }),
        (any::<bool>(), any::<bool>())
            .prop_map(|(big_endian, bom)| Encoding::Utf32 { big_endian, bom }),
    ]
}

fn line_counts(text: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for line in text.replace("\r\n", "\n").replace('\r', "\n").lines() {
        *counts.entry(line.to_owned()).or_default() += 1;
    }
    counts
}

proptest! {
    // A fixed seed makes CI reproducible; proptest still shrinks a failing case.
    #![proptest_config(ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0x5074696e69),
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/properties.proptest-regressions")))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn unicode_encoding_round_trips(payload in text(), encoding in encoding()) {
        // Initial ASCII INI syntax makes BOM-less byte order unambiguous.
        let source = format!("[s]\r\n{payload}");
        let bytes = encoding.encode(&source).unwrap();
        let decoded = EncodedText::decode(&bytes).unwrap();
        prop_assert_eq!(&decoded.text, &source);
        prop_assert_eq!(decoded.encoding.encode(&decoded.text).unwrap(), bytes);
    }

    #[test]
    fn disabled_sort_is_byte_identity(payload in text(), encoding in encoding()) {
        let source = format!("[s]\n{payload}");
        let bytes = encoding.encode(&source).unwrap();
        let options = Options { sort_sections: false, sort_entries: false, ..Options::default() };
        prop_assert_eq!(sort_bytes(&bytes, &options).unwrap(), bytes);
    }

    #[test]
    fn successful_arbitrary_decoding_is_lossless(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        if let Ok(source) = EncodedText::decode(&bytes) {
            prop_assert_eq!(source.encoding.encode(&source.text).unwrap(), bytes);
        }
    }

    #[test]
    fn sorting_is_idempotent_and_retains_raw_lines(
        entries in prop::collection::vec((0u32..10_000, 0u32..10_000, any::<bool>()), 0..80),
        descending in any::<bool>(), natural in any::<bool>(), final_newline in any::<bool>(),
    ) {
        let mut source = String::from("; file header\n[ section10 ]\n");
        for (index, (key, value, disabled)) in entries.iter().enumerate() {
            source.push_str(&format!("; description {index}\n{}key{key}  =  {value} # inline\n", if *disabled { ";" } else { "" }));
        }
        source.push_str("; trailing text\n[ section2 ]\na=1\nz=2\n");
        if !final_newline { source.pop(); }
        let options = Options { descending, natural, ..Options::default() };
        let once = sort_text(&source, &options);
        prop_assert_eq!(sort_text(&once, &options), once.clone());
        prop_assert_eq!(line_counts(&once), line_counts(&source));
        prop_assert_eq!(once.ends_with('\n'), final_newline);
    }

    #[test]
    fn normalization_is_idempotent(
        entries in prop::collection::vec((0u32..10_000, 0u32..10_000), 0..80),
        expand_lines in any::<bool>(), expand_sections in any::<bool>(), descending in any::<bool>(),
    ) {
        let mut source = String::from("  \n[ s10 ]\n");
        for (key, value) in entries { source.push_str(&format!("  key{key}={value} ; note\n\n")); }
        source.push_str("[ s2 ]\na=1\n");
        let options = Options { normalize: true, expand_lines, expand_sections, descending, ..Options::default() };
        let once = sort_text(&source, &options);
        prop_assert_eq!(sort_text(&once, &options), once);
    }

    #[test]
    fn mixed_endings_keep_physical_lines_and_are_idempotent(
        entries in prop::collection::vec((0u32..1000, 0u32..1000, any::<bool>(), 0usize..3, 0usize..3), 0..60),
        descending in any::<bool>(),
    ) {
        let endings = ["\n", "\r\n", "\r"];
        let mut source = String::from("[s]\n");
        for (key, value, blank, entry_eol, blank_eol) in entries {
            if blank { source.push_str(endings[blank_eol]); }
            source.push_str(&format!("key{key}={value}{}", endings[entry_eol]));
        }
        let options = Options { descending, ..Options::default() };
        let once = sort_text(&source, &options);
        prop_assert_eq!(line_counts(&once), line_counts(&source));
        prop_assert_eq!(sort_text(&once, &options), once);
    }
}
