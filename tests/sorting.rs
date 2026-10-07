use sortini::{
    Options,
    encoding::{EncodedText, Encoding},
    sort_bytes, sort_text,
};

fn options(args: &[&str]) -> Options {
    sortini::cli::Command::parse(args.iter().map(Into::into), |_| None)
        .unwrap()
        .options
}

macro_rules! sorting_case {
    ($name:ident, $args:expr, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(sort_text($input, &options($args)), $expected);
        }
    };
}

sorting_case!(
    sections_and_entries,
    &[],
    "[Zebra]\nz=3\na=1\n[Apple]\nb=2\na=1\n",
    "[Apple]\na=1\nb=2\n[Zebra]\na=1\nz=3\n"
);
sorting_case!(
    descending,
    &["-sort-desc"],
    "[A]\na=1\nz=2\n[Z]\nb=3\na=4\n",
    "[Z]\nb=3\na=4\n[A]\nz=2\na=1\n"
);
sorting_case!(
    sections_only,
    &["-!sort-entries"],
    "[Z]\nz=1\na=2\n[A]\nz=3\na=4\n",
    "[A]\nz=3\na=4\n[Z]\nz=1\na=2\n"
);
sorting_case!(
    entries_only,
    &["-!sort-sections"],
    "[Z]\nz=1\na=2\n[A]\nz=3\na=4\n",
    "[Z]\na=2\nz=1\n[A]\na=4\nz=3\n"
);
sorting_case!(
    disabled_sort,
    &["-!sort"],
    "; hi\n[ Z ]\n z = 1 \n a=2\n",
    "; hi\n[ Z ]\n z = 1 \n a=2\n"
);
sorting_case!(
    natural_suffix,
    &[],
    "[s]\nitem10=v\nitem2=v\nitem1=v\nitem20=v\n",
    "[s]\nitem1=v\nitem2=v\nitem10=v\nitem20=v\n"
);
sorting_case!(
    natural_sections,
    &[],
    "[s10]\n[s2]\n[s1]\n",
    "[s1]\n[s2]\n[s10]\n"
);
sorting_case!(
    whole_numbers,
    &[],
    "[s]\n10=v\n2=v\n1=v\n20=v\n",
    "[s]\n1=v\n2=v\n10=v\n20=v\n"
);
sorting_case!(
    exact_decimals,
    &[],
    "[s]\n-1.5=x\n-20=x\n1.001=x\n1=x\n",
    "[s]\n-20=x\n-1.5=x\n1=x\n1.001=x\n"
);
sorting_case!(
    decimal_precision,
    &[],
    "[s]\n9007199254740993=x\n9007199254740992=x\n",
    "[s]\n9007199254740992=x\n9007199254740993=x\n"
);
sorting_case!(
    ordinal,
    &["-ordinal"],
    "[s]\nitem2=v\nitem10=v\nitem1=v\n",
    "[s]\nitem1=v\nitem10=v\nitem2=v\n"
);
sorting_case!(
    negative_suffix,
    &[],
    "[s]\nitem-2=v\nitem-10=v\n",
    "[s]\nitem-10=v\nitem-2=v\n"
);
sorting_case!(
    value_tiebreak,
    &[],
    "[s]\nk=10\nk=2\nk=1\n",
    "[s]\nk=1\nk=2\nk=10\n"
);
sorting_case!(
    stable_equal_entries,
    &[],
    "[s]\n; first\na=1\n; second\na=1\n",
    "[s]\n; first\na=1\n; second\na=1\n"
);
sorting_case!(
    stable_duplicate_sections,
    &[],
    "[s]\nx=1\n[s]\nx=2\n",
    "[s]\nx=1\n[s]\nx=2\n"
);
sorting_case!(
    case_sensitive_entries,
    &["-case-sensitive-entries"],
    "[s]\nA=1\na=1\n",
    "[s]\na=1\nA=1\n"
);
sorting_case!(
    case_sensitive_sections,
    &["-case-sensitive-sections"],
    "[A]\nk=1\n[a]\nk=2\n",
    "[a]\nk=2\n[A]\nk=1\n"
);
sorting_case!(
    case_insensitive_stable,
    &[],
    "[s]\nA=1\na=1\n",
    "[s]\nA=1\na=1\n"
);
sorting_case!(
    unicode_collation,
    &[],
    "[s]\nz=1\né=1\ne=1\n",
    "[s]\ne=1\né=1\nz=1\n"
);
sorting_case!(
    disabled_entries,
    &[],
    "[s]\n;z=off\n#b=off\na=1\n",
    "[s]\na=1\n#b=off\n;z=off\n"
);
sorting_case!(
    comment_attachment,
    &[],
    "[s]\n; z description\n\nz=1\n# a description\na=2\n; trailing\n",
    "[s]\n# a description\na=2\n; z description\n\nz=1\n; trailing\n"
);
sorting_case!(
    opaque_line_attachment,
    &[],
    "[s]\n[broken\nplain text\nz=1\na=2\n",
    "[s]\na=2\n[broken\nplain text\nz=1\n"
);
sorting_case!(
    preamble_not_sorted,
    &[],
    "z=1\na=2\n; header\n\n[s]\nz=3\na=4\n",
    "z=1\na=2\n; header\n\n[s]\na=4\nz=3\n"
);
sorting_case!(
    raw_header_inline_spacing,
    &[],
    "  [ s ] ; stay\n  z  =  café # stay\n  a = 1 ; stay\n",
    "  [ s ] ; stay\n  a = 1 ; stay\n  z  =  café # stay\n"
);
sorting_case!(
    expand,
    &["-expand"],
    "; header\n\n[ s ]\nz=2\na=1\n\n[t]\nb=3\n",
    "; header\n\n[s]\na = 1\nz = 2\n\n[t]\nb = 3\n"
);
sorting_case!(
    compact,
    &["-compact"],
    "\n[ s ]\nz = 2\n\na = 1\n\n[t]\nb = 3\n",
    "[s]\na=1\nz=2\n[t]\nb=3\n"
);
sorting_case!(
    quoted_comments,
    &["-expand"],
    "[s]\na='v # kept' ; trailing\nb=\"v ; kept\" # trailing\nc=v#kept\n",
    "[s]\na = 'v # kept' ; trailing\nb = \"v ; kept\" # trailing\nc = v#kept\n"
);
sorting_case!(
    disabled_normalization,
    &["-expand"],
    "[s]\n;z = v # note\n#b=2\n; a=comment, not entry\na=1\n",
    "[s]\n; a=comment, not entry\na = 1\n#b = 2\n;z = v # note\n"
);
sorting_case!(
    blank_whitespace,
    &["-!sort"],
    " \t\n[s]\n  \nkey=value\n\t",
    " \t\n[s]\n  \nkey=value\n\t"
);
sorting_case!(no_final_newline, &[], "[s]\nz=2\na=1", "[s]\na=1\nz=2");
sorting_case!(
    no_final_newline_section_moves,
    &[],
    "[z]\nx=1\n[a]\nx=2",
    "[a]\nx=2\n[z]\nx=1"
);
sorting_case!(
    mixed_eol_noop,
    &["-!sort"],
    "[s]\r\nz=2\na=1\r",
    "[s]\r\nz=2\na=1\r"
);
sorting_case!(
    mixed_eol_sort,
    &[],
    "[s]\r\nz=2\na=1\r",
    "[s]\r\na=1\rz=2\n"
);

sorting_case!(
    mixed_eol_blank_group_does_not_merge_newlines,
    &[],
    "[s]\na=1\rz=2\n\nb=2\n",
    "[s]\na=1\r\n\nb=2\nz=2\n"
);
sorting_case!(
    eol_override,
    &["-eol", "\\r"],
    "[s]\r\nz=2\na=1\n",
    "[s]\ra=1\rz=2\r"
);
sorting_case!(
    line_only_csharp_normalization,
    &["-expand-lines"],
    "[ s ]\na=1\n\n[t]\nb=2\n",
    "[s]\na = 1\n[t]\nb = 2\n"
);
sorting_case!(
    section_only_csharp_normalization,
    &["-expand-sections"],
    "[ s ]\na = 1\n\n[t]\nb = 2\n",
    "[s]\na=1\n\n[t]\nb=2\n"
);
sorting_case!(empty, &[], "", "");
sorting_case!(
    malformed_and_empty_keys,
    &["-expand"],
    "[broken\n=empty\n; comment = value\n",
    "[broken\n=empty\n; comment = value\n"
);

#[test]
fn all_csharp_golden_fixtures() {
    for name in ["basic", "natural_sort", "preamble", "with_comments"] {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let input = std::fs::read_to_string(root.join(format!("{name}.ini"))).unwrap();
        let expected = std::fs::read_to_string(root.join(format!("{name}_sorted.ini"))).unwrap();
        let normalize = |s: &str| {
            s.trim_start_matches('\u{feff}')
                .replace("\r\n", "\n")
                .trim_end()
                .to_owned()
        };
        assert_eq!(
            normalize(&sort_text(&input, &options(&["-expand"]))),
            normalize(&expected),
            "{name}"
        );
    }
}

pub fn encodings() -> Vec<Encoding> {
    let mut result = vec![Encoding::Utf8 { bom: false }, Encoding::Utf8 { bom: true }];
    for big_endian in [false, true] {
        for bom in [false, true] {
            result.push(Encoding::Utf16 { big_endian, bom });
            result.push(Encoding::Utf32 { big_endian, bom });
        }
    }
    result
}

#[test]
fn encoding_bom_eol_matrix() {
    for encoding in encodings() {
        for eol in ["\n", "\r\n", "\r"] {
            let input = [
                "; header",
                "[ s ]",
                "  z  =  café 🦀 # keep",
                ";b=off",
                " a = 1 ; keep",
                "",
            ]
            .join(eol);
            let expected = [
                "; header",
                "[ s ]",
                " a = 1 ; keep",
                ";b=off",
                "  z  =  café 🦀 # keep",
                "",
            ]
            .join(eol);
            let bytes = encoding.encode(&input).unwrap();
            assert_eq!(
                sort_bytes(&bytes, &Options::default()).unwrap(),
                encoding.encode(&expected).unwrap(),
                "{encoding:?} {eol:?}"
            );
        }
    }
}

#[test]
fn invalid_encodings_are_rejected() {
    for bytes in [
        vec![0xff],
        vec![0xff, 0xfe, 1],
        vec![0xff, 0xfe, 0, 0xd8],
        vec![0xff, 0xfe, 0, 0, 1],
        vec![0, 0, 0xfe, 0xff, 0, 0x11, 0, 0],
    ] {
        assert!(EncodedText::decode(&bytes).is_err(), "{bytes:?}");
    }
}

#[test]
fn encoding_overrides() {
    let input = Encoding::Utf16 {
        big_endian: true,
        bom: true,
    }
    .encode("[s]\na=café\n")
    .unwrap();
    for (name, encoding) in [
        ("utf8", Encoding::Utf8 { bom: false }),
        (
            "unicode",
            Encoding::Utf16 {
                big_endian: false,
                bom: true,
            },
        ),
        (
            "utf32",
            Encoding::Utf32 {
                big_endian: false,
                bom: false,
            },
        ),
        ("os", Encoding::Utf8 { bom: false }),
    ] {
        assert_eq!(
            sort_bytes(&input, &options(&["-encoding", name])).unwrap(),
            encoding.encode("[s]\na=café\n").unwrap()
        );
    }
    assert_eq!(
        sort_bytes(&input, &options(&["-encoding", "ascii"])).unwrap(),
        b"[s]\na=caf?\n"
    );
}

#[test]
fn sorting_is_idempotent() {
    let source = "; header\n[Z10]\n; c\nk10=2\nk2=1\n[Z2]\nA=1\n;a=2\n\n";
    for args in [
        vec![],
        vec!["-expand"],
        vec!["-compact"],
        vec!["-sort-desc"],
        vec!["-ordinal", "-case-sensitive"],
    ] {
        let options = options(&args);
        let once = sort_text(source, &options);
        assert_eq!(sort_text(&once, &options), once);
    }
}

#[test]
fn mixed_numeric_prefixes_have_a_pinned_transitive_order() {
    let input = "[s]\nitem-2=v\nitem-10=v\nitem1.2=v\nitem1.10=v\n-1a=v\n-1=v\n-2=v\n";
    let expected = "[s]\n-2=v\n-1=v\n-1a=v\nitem1.10=v\nitem1.2=v\nitem-10=v\nitem-2=v\n";
    assert_eq!(sort_text(input, &Options::default()), expected);
    assert_eq!(sort_text(expected, &Options::default()), expected);
}
