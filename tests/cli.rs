use sortini::cli::{Command, Information, Pause, information};
use std::{collections::HashMap, ffi::OsString};

fn parse(args: &[&str], env: &[(&str, &str)]) -> Result<Command, String> {
    let environment: HashMap<_, _> = env.iter().copied().collect();
    Command::parse(args.iter().map(OsString::from), |n| {
        environment.get(n).map(OsString::from)
    })
}

#[test]
fn prefixes_aliases_and_negation() {
    for prefix in ["-", "--", "/"] {
        for alias in ["stdout", "redirect-to-stdout"] {
            assert!(parse(&[&format!("{prefix}{alias}")], &[]).unwrap().stdout);
        }
        assert!(
            !parse(&[&format!("{prefix}!sort")], &[])
                .unwrap()
                .options
                .sort_entries
        );
        assert!(
            !parse(&[&format!("{prefix}!natural")], &[])
                .unwrap()
                .options
                .natural
        );
        for alias in ["h", "?", "help"] {
            assert!(matches!(
                parse(&[&format!("{prefix}{alias}")], &[])
                    .unwrap()
                    .information,
                Some(Information::Help(_))
            ));
        }
        assert!(
            !parse(&[&format!("{prefix}t")], &[])
                .unwrap()
                .options
                .ignore_case_entries
        );
        assert!(
            parse(&[&format!("{prefix}i")], &[])
                .unwrap()
                .options
                .ignore_case_entries
        );
    }
}

#[test]
fn defaults_do_not_enable_formatting() {
    let command = parse(&[], &[]).unwrap();
    assert!(
        command.options.sort_entries && command.options.sort_sections && command.options.natural
    );
    assert!(!command.options.normalize);
    assert!(!command.stdout && !command.quiet);
    assert_eq!(command.pause, Pause::Never);
}

#[test]
fn later_options_win() {
    let command = parse(
        &[
            "-sort-desc",
            "-sort-asc",
            "-!sort",
            "-sort-entries",
            "-case-sensitive",
            "-ignore-case-sections",
            "-expand",
            "-compact-lines",
        ],
        &[],
    )
    .unwrap();
    assert!(
        !command.options.descending
            && !command.options.sort_sections
            && command.options.sort_entries
    );
    assert!(command.options.ignore_case_sections && !command.options.ignore_case_entries);
    assert!(!command.options.expand_lines && command.options.expand_sections);
}

#[test]
fn cli_overrides_environment_and_invalid_shadowed_values() {
    let command = parse(
        &["-sort", "-!natural", "-encoding", "utf8", "-pause", "never"],
        &[
            ("sortini_sort", "false"),
            ("sortini_natural", "true"),
            ("sortini_encoding", "bogus"),
            ("sortini_pause", "bogus"),
        ],
    )
    .unwrap();
    assert!(command.options.sort_entries && command.options.sort_sections);
    assert!(!command.options.natural);
    assert_eq!(command.pause, Pause::Never);
}

#[test]
fn environment_specific_overrides() {
    let command = parse(
        &[],
        &[
            ("sortini_sort", "false"),
            ("sortini_sort_entries", "yes"),
            ("sortini_case_sensitive", "1"),
            ("sortini_ignore_case_sections", "true"),
            ("sortini_expand_lines", "false"),
            ("sortini_eol", "\\r"),
        ],
    )
    .unwrap();
    assert!(!command.options.sort_sections && command.options.sort_entries);
    assert!(command.options.ignore_case_sections && !command.options.ignore_case_entries);
    assert!(command.options.normalize && !command.options.expand_lines);
    assert_eq!(command.options.eol.as_deref(), Some("\r"));
}

#[test]
fn boolean_environment_values() {
    for v in ["true", "t", "yes", "y", "1", " TRUE "] {
        assert!(parse(&[], &[("sortini_quiet", v)]).unwrap().quiet);
    }
    for v in ["false", "f", "no", "n", "0"] {
        assert!(
            !parse(&[], &[("sortini_sort", v)])
                .unwrap()
                .options
                .sort_entries
        );
    }
    assert!(parse(&[], &[("sortini_sort", "typo")]).is_err());
}

#[test]
fn literal_paths_and_inline_values() {
    let command = parse(
        &[
            "--out-file=/tmp/output.ini",
            "--sort=false",
            "--",
            "--help",
            "/help",
        ],
        &[],
    )
    .unwrap();
    assert_eq!(
        command.files,
        [
            std::path::PathBuf::from("--help"),
            std::path::PathBuf::from("/help")
        ]
    );
    assert!(!command.options.sort_entries);
    assert_eq!(
        command.out_file.unwrap(),
        std::path::PathBuf::from("/tmp/output.ini")
    );
    assert_eq!(parse(&["/tmp/input.ini"], &[]).unwrap().files.len(), 1);
}

#[test]
fn pause_values() {
    for (value, expected) in [
        ("never", Pause::Never),
        ("always", Pause::Always),
        ("iferror", Pause::IfError),
    ] {
        assert_eq!(parse(&["-pause", value], &[]).unwrap().pause, expected);
    }
    assert_eq!(parse(&["-pause"], &[]).unwrap().pause, Pause::Always);
}

#[test]
fn malformed_options_fail_before_io() {
    for args in [
        vec!["-o"],
        vec!["-encoding", "bogus"],
        vec!["-eol", "bogus"],
        vec!["-pause", "sometimes"],
        vec!["-abc"],
        vec!["-!encoding", "utf8"],
        vec!["--sort=typo"],
        vec!["--!sort=false"],
    ] {
        assert!(parse(&args, &[]).is_err(), "{args:?}");
    }
}

#[test]
fn help_topics_and_environment_listing() {
    assert!(information(&Information::Help("natural".into()), |_| None).contains("-natural"));
    let output = information(&Information::Envars, |name| Some(name.into()));
    assert!(output.contains("sortini_sort_entries"));
    assert!(!output.contains("sortini_compact"));
    assert!(matches!(
        parse(&["-help", "examples"], &[]).unwrap().information,
        Some(Information::Examples)
    ));
    assert!(matches!(
        parse(&["-help", "envars"], &[]).unwrap().information,
        Some(Information::Envars)
    ));
}
