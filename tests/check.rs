mod support;
use sortini::encoding::Encoding;
use std::fs;
use support::run;

#[test]
fn check_reports_changes_without_writing_or_creating_backups() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("input.ini");
    let source = b"[s]\nz=2\na=1\n";
    fs::write(&path, source).unwrap();
    let before = fs::metadata(&path).unwrap().modified().unwrap();
    let result = run(&["--check".as_ref(), path.as_os_str()], b"", &[]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("Would change")
    );
    assert_eq!(fs::read(&path).unwrap(), source);
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), before);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn check_sorted_input_succeeds_for_all_prefixes() {
    for flag in ["-check", "--check", "/check"] {
        let result = run(&[flag.as_ref()], b"[s]\na=1\nz=2\n", &[]);
        assert!(result.status.success());
        assert!(result.stdout.is_empty());
        assert!(
            String::from_utf8(result.stderr)
                .unwrap()
                .contains("Unchanged: stdin")
        );
    }
}

#[test]
fn check_stdin_and_empty_stdin() {
    assert_eq!(
        run(&["--check".as_ref()], b"[s]\nz=2\na=1\n", &[])
            .status
            .code(),
        Some(1)
    );
    assert!(run(&["--check".as_ref()], b"", &[]).status.success());
}

#[test]
fn quiet_check_only_sets_exit_status() {
    for (source, code) in [
        (b"[s]\nz=2\na=1\n".as_slice(), 1),
        (b"[s]\na=1\nz=2\n".as_slice(), 0),
    ] {
        let result = run(&["--check".as_ref(), "--quiet".as_ref()], source, &[]);
        assert_eq!(result.status.code(), Some(code));
        assert!(result.stdout.is_empty() && result.stderr.is_empty());
    }
}

#[test]
fn check_honors_sort_and_format_options() {
    let source = b"[s]\nz = 2\na=1\n";
    assert!(
        run(&["--check".as_ref(), "-!sort".as_ref()], source, &[])
            .status
            .success()
    );
    assert_eq!(
        run(
            &["--check".as_ref(), "-!sort".as_ref(), "--compact".as_ref()],
            source,
            &[]
        )
        .status
        .code(),
        Some(1)
    );
    let result = run(&["--check".as_ref()], source, &[("sortini_sort", "false")]);
    assert!(result.status.success());
}

#[test]
fn check_compares_encoding_bom_and_eol() {
    let source = Encoding::Utf16 {
        big_endian: true,
        bom: true,
    }
    .encode("[s]\na=é\r\n")
    .unwrap();
    assert!(run(&["--check".as_ref()], &source, &[]).status.success());
    assert_eq!(
        run(
            &["--check".as_ref(), "--encoding".as_ref(), "utf8".as_ref()],
            &source,
            &[]
        )
        .status
        .code(),
        Some(1)
    );
    assert_eq!(
        run(
            &["--check".as_ref(), "--eol".as_ref(), "\\n".as_ref()],
            &source,
            &[]
        )
        .status
        .code(),
        Some(1)
    );
}

#[test]
fn check_preserves_blank_whitespace_and_no_final_newline() {
    assert!(
        run(&["--check".as_ref()], b" \t\n[s]\na=1\n\tz=2", &[])
            .status
            .success()
    );
}

#[test]
fn check_rejects_output_options_before_io() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output.ini");
    for args in [
        vec!["--check".as_ref(), "--stdout".as_ref()],
        vec!["--check".as_ref(), "-o".as_ref(), output.as_os_str()],
    ] {
        let result = run(&args, b"[s]\nz=2\na=1\n", &[]);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(!output.exists());
    }
}

#[test]
fn check_processes_every_file_and_errors_override_changes_in_either_order() {
    let temp = tempfile::tempdir().unwrap();
    let changed = temp.path().join("changed.ini");
    let unchanged = temp.path().join("unchanged.ini");
    let missing = temp.path().join("missing.ini");
    fs::write(&changed, b"[s]\nz=2\na=1\n").unwrap();
    fs::write(&unchanged, b"[s]\na=1\nz=2\n").unwrap();
    let result = run(
        &[
            "--check".as_ref(),
            changed.as_os_str(),
            unchanged.as_os_str(),
        ],
        b"",
        &[],
    );
    assert_eq!(result.status.code(), Some(1));
    let diagnostics = String::from_utf8(result.stderr).unwrap();
    assert!(diagnostics.contains("changed.ini") && diagnostics.contains("unchanged.ini"));
    for paths in [[&changed, &missing], [&missing, &changed]] {
        let result = run(
            &[
                "--check".as_ref(),
                paths[0].as_os_str(),
                paths[1].as_os_str(),
            ],
            b"",
            &[],
        );
        assert_eq!(result.status.code(), Some(10));
        let diagnostics = String::from_utf8(result.stderr).unwrap();
        assert!(diagnostics.contains("changed.ini") && diagnostics.contains("missing.ini"));
    }
    assert_eq!(fs::read(changed).unwrap(), b"[s]\nz=2\na=1\n");
}

#[test]
fn check_invalid_encoding_is_an_error() {
    let result = run(&["--check".as_ref(), "--quiet".as_ref()], &[0xff], &[]);
    assert_eq!(result.status.code(), Some(10));
    assert!(result.stdout.is_empty() && !result.stderr.is_empty());
}

#[test]
fn negated_check_restores_normal_stdout() {
    let result = run(
        &["--check".as_ref(), "-!check".as_ref()],
        b"[s]\nz=2\na=1\n",
        &[],
    );
    assert!(result.status.success());
    assert_eq!(result.stdout, b"[s]\na=1\nz=2\n");
}

#[cfg(windows)]
#[test]
fn check_reads_readonly_files_without_attempting_a_write() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("readonly.ini");
    fs::write(&path, b"[s]\nz=2\na=1\n").unwrap();
    let original = fs::metadata(&path).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&path, readonly).unwrap();
    let result = run(&["--check".as_ref(), path.as_os_str()], b"", &[]);
    fs::set_permissions(&path, original).unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}
