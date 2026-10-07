//! Optional live oracle check; the normal suite is independent of the C# checkout.
use sortini::encoding::Encoding;
use std::{
    ffi::OsStr,
    fs,
    process::{Command, Stdio},
};

fn csharp(dll: &OsStr, path: &OsStr, args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new("dotnet");
    clear_environment(&mut command);
    command
        .arg(dll)
        .arg("--stdout")
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
#[ignore = "set SORTINI_CSHARP_DLL to a built C# sortini.dll; see docs/development.md"]
fn csharp_output_parity() {
    let dll = std::env::var_os("SORTINI_CSHARP_DLL")
        .expect("Set SORTINI_CSHARP_DLL to a built C# sortini.dll");
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
    cases.push("[s]\nitem-2=v\nitem-10=v\nitem1.2=v\nitem1.10=v\n-1.5=v\n-20=v\n1,000=v\n10=v\n2=v\n2-=v\n9007199254740993=v\n9007199254740992=v\n".into());
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
    let mut mismatches = Vec::new();
    for (case, input) in cases.iter().enumerate() {
        fs::write(&path, input).unwrap();
        for Configuration {
            args,
            environment: env,
        } in &configurations
        {
            let expected = csharp(&dll, path.as_os_str(), args, env);
            let mut command = Command::new(env!("CARGO_BIN_EXE_sortini"));
            clear_environment(&mut command);
            command
                .arg("--stdout")
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
                let mut reference = Command::new("dotnet");
                clear_environment(&mut reference);
                let result = reference
                    .arg(&dll)
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
                        .arg(&rust_path)
                        .stdin(Stdio::null())
                        .output()
                        .unwrap()
                        .status
                        .success()
                );
                assert_eq!(
                    fs::read(&path).unwrap(),
                    fs::read(rust_path).unwrap(),
                    "{encoding:?} {eol:?} {override_args:?}"
                );
                checked += 1;
            }
        }
    }
    println!("{checked} direct C# comparisons passed");
}
