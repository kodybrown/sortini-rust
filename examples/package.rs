//! Build a local Windows ZIP without installing or publishing anything.
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(windows) {
        return Err("Packaging currently supports Windows only".into());
    }
    let release = std::env::current_exe()?
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or("Cannot find release directory")?
        .to_owned();
    let mut binary = release.join("sortini.exe");
    let mut directory = release.join("distribution");
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--binary") => binary = args.next().ok_or("Missing --binary path")?.into(),
            Some("--output-dir") => {
                directory = args.next().ok_or("Missing --output-dir path")?.into()
            }
            Some("--help") => {
                println!(
                    "cargo build --release --locked\ncargo run --release --locked --example package -- [--binary path] [--output-dir path]"
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown argument: {}", arg.to_string_lossy()).into()),
        }
    }
    let payload = fs::read(&binary)?;
    let readme = format!(
        "sortini-rust {} for Windows {}\r\n\r\nRun sortini_rust.exe --help\r\nRun sortini_rust.exe --check settings.ini (0 unchanged, 1 changes needed)\r\nRun sortini_rust.exe --stdout settings.ini to preview sorted output.\r\n\r\nThe archive uses sortini_rust.exe so it can coexist with sortini_cs.exe.\r\nThis package does not install itself or change PATH.\r\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::ARCH
    );
    let license = fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("LICENSE"))?;
    let manifest = format!(
        "{:x}  sortini_rust.exe\r\n{:x}  README.txt\r\n{:x}  LICENSE\r\n",
        Sha256::digest(&payload),
        Sha256::digest(readme.as_bytes()),
        Sha256::digest(&license)
    );
    let files = [
        ("sortini_rust.exe", payload),
        ("README.txt", readme.into_bytes()),
        ("LICENSE", license),
        ("SHA256SUMS.txt", manifest.into_bytes()),
    ];
    fs::create_dir_all(&directory)?;
    let archive = directory.join(format!(
        "sortini-rust-{}-windows-{}.zip",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::ARCH
    ));
    // create_new protects a previously reviewed package from accidental overwrite.
    let output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&archive)?;
    let mut writer = ZipWriter::new(output);
    for (name, bytes) in &files {
        writer.start_file(
            *name,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
        )?;
        writer.write_all(bytes)?;
    }
    writer.finish()?.sync_all()?;
    let mut checked = ZipArchive::new(fs::File::open(&archive)?)?;
    for (name, expected) in &files {
        let mut actual = Vec::new();
        checked.by_name(name)?.read_to_end(&mut actual)?;
        if &actual != expected {
            return Err(format!("Archive verification failed for {name}").into());
        }
    }
    // Exercise the bytes read back from the archive, under the distributed name.
    // This stays in a disposable directory and never installs into the user Bin.
    let restored = tempfile::tempdir()?;
    let restored_executable = restored.path().join("sortini_rust.exe");
    let mut restored_bytes = Vec::new();
    checked
        .by_name("sortini_rust.exe")?
        .read_to_end(&mut restored_bytes)?;
    fs::write(&restored_executable, restored_bytes)?;
    let mut command = std::process::Command::new(&restored_executable);
    for (name, _) in std::env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_lowercase()
            .starts_with("sortini_")
        {
            command.env_remove(name);
        }
    }
    let version = command
        .arg("-v")
        .stdin(std::process::Stdio::null())
        .output()?;
    if !version.status.success()
        || String::from_utf8_lossy(&version.stdout).trim() != env!("CARGO_PKG_VERSION")
    {
        return Err("Restored executable version smoke test failed".into());
    }
    let input = restored.path().join("smoke.ini");
    let source = b"[s]\nz=2\na=1\n";
    fs::write(&input, source)?;
    let mut command = std::process::Command::new(&restored_executable);
    for (name, _) in std::env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_lowercase()
            .starts_with("sortini_")
        {
            command.env_remove(name);
        }
    }
    let check = command
        .arg("--check")
        .arg(&input)
        .stdin(std::process::Stdio::null())
        .output()?;
    if check.status.code() != Some(1) || !check.stdout.is_empty() || fs::read(&input)? != source {
        return Err("Restored executable check smoke test failed".into());
    }
    let checksum = format!(
        "{:x}  {}\n",
        Sha256::digest(fs::read(&archive)?),
        archive
            .file_name()
            .ok_or("Missing archive filename")?
            .to_string_lossy()
    );
    let checksum_path = archive.with_extension("zip.sha256");
    let mut checksum_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(checksum_path)?;
    checksum_file.write_all(checksum.as_bytes())?;
    checksum_file.sync_all()?;
    println!(
        "Verified Windows package: {}\n{checksum}",
        archive.display()
    );
    Ok(())
}
