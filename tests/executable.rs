mod support;

use sortini::encoding::{EncodedText, Encoding};
use std::{ffi::OsStr, fs};
use support::run;

#[test]
fn information_options_with_redirected_output() {
    for (arg, expected) in [
        ("--help", "USAGE"),
        ("/help", "OPTIONS"),
        ("--envars", "sortini_"),
        ("--examples", "EXAMPLES"),
        ("--version", "sortini"),
        ("-v", "4.0.0"),
        ("--about", "Rust"),
    ] {
        let result = run(&[arg.as_ref()], b"", &[]);
        assert!(result.status.success(), "{arg}: {:?}", result.stderr);
        assert!(String::from_utf8(result.stdout).unwrap().contains(expected));
    }
}

#[test]
fn stdin_defaults_to_stdout() {
    let result = run(&[], b"[s]\nz = 2\na=1\n", &[]);
    assert!(result.status.success());
    assert_eq!(result.stdout, b"[s]\na=1\nz = 2\n");
    assert!(result.stderr.is_empty());
}

#[test]
fn empty_stdin_is_valid_empty_document() {
    let result = run(&[], b"", &[]);
    assert!(result.status.success());
    assert!(result.stdout.is_empty() && result.stderr.is_empty());
}

#[test]
fn explicit_options_and_prefixes() {
    for (option, expected) in [
        ("--expand", "[s]\na = 1\nz = 2\n"),
        ("/compact", "[s]\na=1\nz=2\n"),
        ("-expand-lines", "[s]\na = 1\nz = 2\n"),
        ("--compact-lines", "[s]\na=1\nz=2\n"),
        ("-!sort", "[s]\nz = 2\na=1\n"),
        ("--sort-desc", "[s]\nz = 2\na=1\n"),
    ] {
        let result = run(&[option.as_ref()], b"[s]\nz = 2\na=1\n", &[]);
        assert!(result.status.success());
        assert_eq!(result.stdout, expected.as_bytes(), "{option}");
    }
}

#[test]
fn environment_precedence_in_process() {
    let result = run(
        &["--sort".as_ref()],
        b"[s]\nz=2\na=1\n",
        &[("sortini_sort", "false")],
    );
    assert!(result.status.success());
    assert_eq!(result.stdout, b"[s]\na=1\nz=2\n");
}

#[test]
fn environment_sort_and_format() {
    let result = run(
        &[],
        b"[s]\nz=2\na=1\n",
        &[("sortini_sort", "false"), ("sortini_expand_lines", "true")],
    );
    assert!(result.status.success());
    assert_eq!(result.stdout, b"[s]\nz = 2\na = 1\n");
}

#[test]
fn stdin_outfile_encoding_and_eol_overrides() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("sorted.ini");
    let result = run(
        &[
            "--out-file".as_ref(),
            path.as_os_str(),
            "--encoding".as_ref(),
            "unicode".as_ref(),
            "--eol".as_ref(),
            "\\r\\n".as_ref(),
        ],
        b"[s]\nz=2\na=1\n",
        &[],
    );
    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(result.stdout.is_empty());
    assert_eq!(
        fs::read(path).unwrap(),
        Encoding::Utf16 {
            big_endian: false,
            bom: true
        }
        .encode("[s]\r\na=1\r\nz=2\r\n")
        .unwrap()
    );
}

#[test]
fn stdin_detects_unicode_bytes_without_tempfile() {
    let source = Encoding::Utf32 {
        big_endian: true,
        bom: true,
    }
    .encode("[s]\nz=é\na=🦀\r")
    .unwrap();
    let result = run(&[], &source, &[]);
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "[s]\na=🦀\rz=é\n"
    );
}

#[test]
fn stdout_does_not_modify_input_even_with_outfile() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("source.ini");
    let output = temp.path().join("output.ini");
    let source = b"[s]\nz=2\na=1\n";
    fs::write(&path, source).unwrap();
    let result = run(
        &[
            path.as_os_str(),
            "--stdout".as_ref(),
            "-o".as_ref(),
            output.as_os_str(),
            "--encoding".as_ref(),
            "unicode".as_ref(),
        ],
        b"",
        &[],
    );
    assert!(result.status.success());
    assert_eq!(result.stdout, b"[s]\na=1\nz=2\n");
    assert_eq!(fs::read(path).unwrap(), source);
    assert!(!output.exists());
}

#[test]
fn outfile_preserves_original() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("source.ini");
    let output = temp.path().join("sorted.ini");
    let source = b"[s]\nz=2\na=1\n";
    fs::write(&path, source).unwrap();
    let result = run(
        &[path.as_os_str(), "-o".as_ref(), output.as_os_str()],
        b"",
        &[],
    );
    assert!(result.status.success());
    assert_eq!(fs::read(path).unwrap(), source);
    assert_eq!(fs::read(output).unwrap(), b"[s]\na=1\nz=2\n");
}

#[test]
fn in_place_preserves_encodings_and_eols() {
    let temp = tempfile::tempdir().unwrap();
    let mut encodings = vec![Encoding::Utf8 { bom: false }, Encoding::Utf8 { bom: true }];
    for big_endian in [false, true] {
        encodings.extend([
            Encoding::Utf16 {
                big_endian,
                bom: true,
            },
            Encoding::Utf32 {
                big_endian,
                bom: true,
            },
        ]);
    }
    for encoding in encodings {
        for eol in ["\n", "\r\n", "\r"] {
            let path = temp.path().join("preserve.ini");
            fs::write(
                &path,
                encoding
                    .encode(
                        &[
                            "; keep",
                            "[ s ]",
                            "  z  = café # keep",
                            ";b=off",
                            " a = 1",
                            "",
                        ]
                        .join(eol),
                    )
                    .unwrap(),
            )
            .unwrap();
            let result = run(&[path.as_os_str()], b"", &[]);
            assert!(result.status.success(), "{:?}", result.stderr);
            assert_eq!(
                fs::read(path).unwrap(),
                encoding
                    .encode(
                        &[
                            "; keep",
                            "[ s ]",
                            " a = 1",
                            ";b=off",
                            "  z  = café # keep",
                            ""
                        ]
                        .join(eol)
                    )
                    .unwrap()
            );
        }
    }
    assert_eq!(
        fs::read_dir(temp.path()).unwrap().count(),
        1,
        "successful writes remove their backups"
    );
}

#[test]
fn multiple_inputs_are_processed() {
    let temp = tempfile::tempdir().unwrap();
    let paths = [temp.path().join("one.ini"), temp.path().join("two.ini")];
    for path in &paths {
        fs::write(path, b"[s]\nz=2\na=1\n").unwrap();
    }
    let result = run(&[paths[0].as_os_str(), paths[1].as_os_str()], b"", &[]);
    assert!(result.status.success());
    for path in paths {
        assert_eq!(fs::read(path).unwrap(), b"[s]\na=1\nz=2\n");
    }
}

#[test]
fn multiple_inputs_to_stdout() {
    let temp = tempfile::tempdir().unwrap();
    let paths = [temp.path().join("one.ini"), temp.path().join("two.ini")];
    for path in &paths {
        fs::write(path, b"[s]\nz=2\na=1\n").unwrap();
    }
    let result = run(
        &[
            "--stdout".as_ref(),
            paths[0].as_os_str(),
            paths[1].as_os_str(),
        ],
        b"",
        &[],
    );
    assert!(result.status.success());
    assert_eq!(result.stdout, b"[s]\na=1\nz=2\n[s]\na=1\nz=2\n");
}

#[test]
fn repeated_load_errors_continue_to_valid_input() {
    let temp = tempfile::tempdir().unwrap();
    let one = temp.path().join("missing1");
    let two = temp.path().join("missing2");
    let valid = temp.path().join("valid.ini");
    fs::write(&valid, b"[s]\nz=2\na=1\n").unwrap();
    let result = run(
        &[one.as_os_str(), two.as_os_str(), valid.as_os_str()],
        b"",
        &[],
    );
    assert_eq!(result.status.code(), Some(10));
    let errors = String::from_utf8(result.stderr).unwrap();
    assert!(errors.contains("missing1") && errors.contains("missing2"));
    assert_eq!(fs::read(valid).unwrap(), b"[s]\na=1\nz=2\n");
}

#[test]
fn outfile_rejects_multiple_inputs_before_modifying_anything() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("one.ini");
    let output = temp.path().join("output.ini");
    fs::write(&path, b"[s]\nz=2\na=1\n").unwrap();
    let result = run(
        &[
            path.as_os_str(),
            path.as_os_str(),
            "-o".as_ref(),
            output.as_os_str(),
        ],
        b"",
        &[],
    );
    assert_eq!(result.status.code(), Some(99));
    assert!(!output.exists());
    assert_eq!(fs::read(path).unwrap(), b"[s]\nz=2\na=1\n");
}

#[test]
fn invalid_options_do_not_modify_input() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("one.ini");
    let source = b"[s]\nz=2\na=1\n";
    fs::write(&path, source).unwrap();
    for args in [
        vec![
            path.as_os_str(),
            OsStr::new("--encoding"),
            OsStr::new("bogus"),
        ],
        vec![path.as_os_str(), OsStr::new("--eol"), OsStr::new("bad")],
        vec![path.as_os_str(), OsStr::new("--abc")],
    ] {
        let result = run(&args, b"", &[]);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert_eq!(fs::read(&path).unwrap(), source);
    }
}

#[test]
fn invalid_input_is_not_overwritten() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("invalid.ini");
    fs::write(&path, [0xff]).unwrap();
    let result = run(&[path.as_os_str()], b"", &[]);
    assert_eq!(result.status.code(), Some(10));
    assert_eq!(fs::read(path).unwrap(), [0xff]);
}

#[test]
fn save_error_is_reported() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("missing-parent/output.ini");
    let result = run(&["-o".as_ref(), output.as_os_str()], b"[s]\na=1\n", &[]);
    assert_eq!(result.status.code(), Some(13));
    assert!(!result.stderr.is_empty());
}

#[test]
fn quiet_keeps_data_and_errors_visible() {
    let result = run(&["-quiet".as_ref()], b"[s]\nz=2\na=1\n", &[]);
    assert!(result.status.success());
    assert_eq!(result.stdout, b"[s]\na=1\nz=2\n");
    let result = run(&["-quiet".as_ref()], &[0xff], &[]);
    assert_eq!(result.status.code(), Some(10));
    assert!(!result.stderr.is_empty());
}

#[test]
fn pause_does_not_block_redirected_input() {
    for value in ["always", "iferror", "never"] {
        let result = run(&["-pause".as_ref(), value.as_ref()], b"[s]\na=1\n", &[]);
        assert!(result.status.success());
        assert_eq!(result.stdout, b"[s]\na=1\n");
    }
}

#[test]
fn literal_flag_filename() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("--help");
    fs::write(&path, b"[s]\nz=2\na=1\n").unwrap();
    let result = run(&["--".as_ref(), path.as_os_str()], b"", &[]);
    assert!(result.status.success());
    assert_eq!(
        EncodedText::decode(&fs::read(path).unwrap()).unwrap().text,
        "[s]\na=1\nz=2\n"
    );
}

#[test]
fn unicode_filename() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("café 🦀.ini");
    fs::write(&path, b"[s]\nz=2\na=1\n").unwrap();
    let result = run(&[path.as_os_str()], b"", &[]);
    assert!(result.status.success());
    assert_eq!(fs::read(path).unwrap(), b"[s]\na=1\nz=2\n");
}

#[cfg(unix)]
#[test]
fn non_unicode_filename() {
    use std::os::unix::ffi::OsStringExt;
    let temp = tempfile::tempdir().unwrap();
    let path = temp
        .path()
        .join(std::ffi::OsString::from_vec(vec![b'a', 0xff]));
    fs::write(&path, b"[s]\nz=2\na=1\n").unwrap();
    assert!(run(&[path.as_os_str()], b"", &[]).status.success());
    assert_eq!(fs::read(path).unwrap(), b"[s]\na=1\nz=2\n");
}
