//! Optional live oracle check; the normal suite is independent of the C# checkout.
use sha2::{Digest, Sha256};
use sortini::encoding::Encoding;
use std::{
    ffi::OsStr,
    fs,
    process::{Command, Stdio},
};

fn csharp(
    executable: &OsStr,
    path: &OsStr,
    args: &[&str],
    env: &[(&str, &str)],
) -> std::process::Output {
    let mut command = Command::new(executable);
    clear_environment(&mut command);
    command
        .arg("--stdout")
        .arg("--expand")
        .args(args)
        .arg(path)
        .stdin(Stdio::null());
    for (name, value) in env {
        command.env(name, value);
    }
    command.output().unwrap()
}

fn clear_environment(command: &mut Command) {
    for (name, _) in std::env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_lowercase()
            .starts_with("sortini_")
        {
            command.env_remove(name);
        }
    }
}

#[test]
#[ignore = "set SORTINI_CSHARP_EXE to the Windows C# executable; see docs/development.md"]
fn csharp_output_parity() {
    let reference = std::env::var_os("SORTINI_CSHARP_EXE")
        .expect("Set SORTINI_CSHARP_EXE to the Windows C# executable");
    assert_eq!(
        format!("{:x}", Sha256::digest(fs::read(&reference).unwrap())),
        "0752682ea7e13b49c798df03e9aa7961fbbd5f33f5785e7a2c1fb87f519fd714",
        "This oracle profile pins the supplied June C# build; update documented expectations for another build"
    );
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("input.ini");
    let mut cases: Vec<String> = ["basic", "natural_sort", "preamble", "with_comments"]
        .iter()
        .map(|name| {
            fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("tests/fixtures/{name}.ini")),
            )
            .unwrap()
            .replace("\r\n", "\n")
        })
        .collect();
    cases.push("; header\n[ s10 ]\n; before z\nz=1\n#b=2\na=\"v ; keep\" # inline\n; trailing\n[s2]\nz=3\na=2\n".into());
    cases.push(
        "[s]\nz=1\né=1\ne=1\nÉ=2\nA=1\na=1\n_=1\n-=1\n'=1\nß=1\nss=2\nΩ=1\nω=2\n🦀=1\n".into(),
    );
    // The legacy mixed-prefix comparator is nontransitive. Compare supported
    // numeric families independently; pin the corrected mixed order separately.
    cases.push("[negative]\nitem-2=v\nitem-10=v\n[positive]\nitem1.2=v\nitem1.10=v\n[numbers]\n-1.5=v\n-20=v\n1,000=v\n10=v\n2=v\n2-=v\n9007199254740993=v\n9007199254740992=v\n".into());
    // Unique values avoid the legacy List.Sort instability for equal groups.
    let mut generated = String::from("; deterministic generated coverage\n");
    for section in (0..7).rev() {
        generated.push_str(&format!("[part{section}]\n"));
        for item in (0..23).rev() {
            generated.push_str(&format!(
                "; item description {item}\nkey{item} = {} # note\n",
                item * 17
            ));
        }
    }
    cases.push(generated);
    struct Configuration<'a> {
        args: Vec<&'a str>,
        environment: Vec<(&'a str, &'a str)>,
    }
    let configurations: Vec<Configuration<'_>> = vec![
        (vec![], vec![]),
        (vec!["-sort-desc"], vec![]),
        (vec!["-!sort"], vec![]),
        (vec!["-!sort-sections"], vec![]),
        (vec!["-!sort-entries"], vec![]),
        (vec!["-ordinal"], vec![]),
        (vec!["-case-sensitive"], vec![]),
        (vec!["-case-sensitive-sections"], vec![]),
        (vec!["-case-sensitive-entries"], vec![]),
        (vec!["-case-sensitive", "-ignore-case-sections"], vec![]),
        (vec!["-expand"], vec![]),
        (vec!["-compact"], vec![]),
        (vec!["-expand-lines"], vec![]),
        (vec!["-compact-lines"], vec![]),
        (vec!["-expand-sections"], vec![]),
        (vec!["-compact-sections"], vec![]),
        (vec!["-expand", "-!expand-lines"], vec![]),
        (vec!["-compact", "-!compact-sections"], vec![]),
        (vec!["-eol", "\\r\\n"], vec![]),
        (vec!["/sort-desc", "/!natural"], vec![]),
        (vec![], vec![("sortini_sort", "false")]),
        (vec!["-sort"], vec![("sortini_sort", "false")]),
        (vec![], vec![("sortini_expand_lines", "true")]),
        (
            vec![],
            vec![
                ("sortini_case_sensitive", "true"),
                ("sortini_ignore_case_entries", "true"),
            ],
        ),
        (
            vec![],
            vec![("sortini_sort", "false"), ("sortini_sort_entries", "true")],
        ),
    ]
    .into_iter()
    .map(|(args, environment)| Configuration { args, environment })
    .collect();
    let mut checked = 0;
    let mut legacy_bom_differences = 0;
    let mut legacy_eol_differences = 0;
    let mut different_cases = 0;
    let mut mismatches = Vec::new();
    for (case, input) in cases.iter().enumerate() {
        fs::write(&path, input).unwrap();
        for Configuration {
            args,
            environment: env,
        } in &configurations
        {
            let expected = csharp(&reference, path.as_os_str(), args, env);
            let mut command = Command::new(env!("CARGO_BIN_EXE_sortini"));
            clear_environment(&mut command);
            command
                .arg("--stdout")
                .arg("--expand")
                .args(args)
                .arg(&path)
                .stdin(Stdio::null());
            for (name, value) in env {
                command.env(name, value);
            }
            let actual = command.output().unwrap();
            assert!(
                expected.status.success(),
                "C# failed: {:?}",
                expected.stderr
            );
            assert!(actual.status.success(), "Rust failed: {:?}", actual.stderr);
            if actual.stdout != expected.stdout {
                mismatches.push(format!(
                    "case {case}, args {args:?}, env {env:?}\nC#: {}\nRust: {}",
                    String::from_utf8_lossy(&expected.stdout),
                    String::from_utf8_lossy(&actual.stdout)
                ));
            }
            checked += 1;
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} / {checked} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
    // Compare file bytes as well: stdout deliberately does not carry source BOMs.
    for encoding in [
        Encoding::Utf8 { bom: false },
        Encoding::Utf8 { bom: true },
        Encoding::Utf16 {
            big_endian: false,
            bom: true,
        },
        Encoding::Utf16 {
            big_endian: true,
            bom: true,
        },
        Encoding::Utf32 {
            big_endian: false,
            bom: true,
        },
        Encoding::Utf32 {
            big_endian: true,
            bom: true,
        },
    ] {
        for eol in ["\n", "\r\n", "\r"] {
            for override_args in [
                vec![],
                vec!["--encoding", "utf8"],
                vec!["--encoding", "ascii"],
                vec!["--encoding", "unicode"],
                vec!["--encoding", "utf32"],
                vec!["--encoding", "os", "--eol", "\\r\\n"],
            ] {
                let input = encoding
                    .encode(&["; header", "[ s ]", " z  = café # keep", "a=1", ""].join(eol))
                    .unwrap();
                let rust_path = temp.path().join("rust.ini");
                fs::write(&path, &input).unwrap();
                fs::write(&rust_path, &input).unwrap();
                let mut oracle = Command::new(&reference);
                clear_environment(&mut oracle);
                let result = oracle
                    .arg("--expand")
                    .args(&override_args)
                    .arg(&path)
                    .stdin(Stdio::null())
                    .output()
                    .unwrap();
                assert!(result.status.success(), "{:?}", result.stderr);
                let mut rust = Command::new(env!("CARGO_BIN_EXE_sortini"));
                clear_environment(&mut rust);
                assert!(
                    rust.args(&override_args)
                        .arg("--expand")
                        .arg(&rust_path)
                        .stdin(Stdio::null())
                        .output()
                        .unwrap()
                        .status
                        .success()
                );
                let actual = fs::read(rust_path).unwrap();
                // This archived build uses Encoding.UTF8/UTF32 for overrides,
                // which add BOMs. The refreshed v4 source and Rust deliberately
                // use BOM-less overrides. Assert that exact legacy difference;
                // do not normalize away arbitrary byte mismatches.
                let override_name = override_args.get(1).copied();
                let output_encoding =
                    override_name.map_or(encoding, |name| Encoding::from_name(name).unwrap());
                let has_eol_override = override_args.contains(&"--eol");
                let rust_eol = if has_eol_override { "\r\n" } else { eol };
                let lines = ["; header", "", "[s]", "a = 1", "z = café # keep", ""];
                assert_eq!(
                    actual,
                    output_encoding.encode(&lines.join(rust_eol)).unwrap(),
                    "Rust bytes: {encoding:?} {eol:?} {override_args:?}"
                );
                let legacy_encoding = match override_name {
                    Some("utf8") => Encoding::Utf8 { bom: true },
                    Some("utf32") => Encoding::Utf32 {
                        big_endian: false,
                        bom: true,
                    },
                    _ => output_encoding,
                };
                // The archived EOL helper scans bytes, so CRLF in UTF-16/32
                // appears to be bare CR because the next byte is zero.
                let legacy_eol_bug = !matches!(encoding, Encoding::Utf8 { .. })
                    && eol == "\r\n"
                    && !has_eol_override;
                let expected_reference = legacy_encoding
                    .encode(&lines.join(if legacy_eol_bug { "\r" } else { rust_eol }))
                    .unwrap();
                assert_eq!(
                    fs::read(&path).unwrap(),
                    expected_reference,
                    "{encoding:?} {eol:?} {override_args:?}"
                );
                legacy_bom_differences += usize::from(legacy_encoding != output_encoding);
                legacy_eol_differences += usize::from(legacy_eol_bug);
                different_cases += usize::from(expected_reference != actual);
                checked += 1;
            }
        }
    }
    assert_eq!(legacy_bom_differences, 36);
    assert_eq!(legacy_eol_differences, 20);
    assert_eq!(different_cases, 48);
    println!(
        "{checked} direct C# comparisons checked: {} exact matches; {different_cases} pinned legacy difference cases ({legacy_bom_differences} BOM, {legacy_eol_differences} EOL, with overlap)",
        checked - different_cases
    );
}
