//! Windows process benchmark. Run a release binary, not `cargo run` per sample.

#[cfg(not(windows))]
fn main() {
    eprintln!("This benchmark currently supports Windows only.");
    std::process::exit(2);
}

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    windows::run()
}

#[cfg(windows)]
mod windows {
    use sha2::{Digest, Sha256};
    use std::{
        error::Error,
        ffi::OsString,
        fs,
        os::windows::io::AsRawHandle,
        path::{Path, PathBuf},
        process::{Child, Command, Stdio},
        thread,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::{Foundation::WAIT_FAILED, System::Threading::WaitForSingleObject};

    struct Sample {
        milliseconds: f64,
        peak_bytes: Option<usize>,
    }

    fn configured(executable: &Path) -> Command {
        let mut command = Command::new(executable);
        for (name, _) in std::env::vars_os() {
            if name
                .to_string_lossy()
                .to_ascii_lowercase()
                .starts_with("sortini_")
            {
                command.env_remove(name);
            }
        }
        command.stdin(Stdio::null());
        command
    }

    fn peak_memory(child: &Child) -> Option<usize> {
        let mut counters = PROCESS_MEMORY_COUNTERS {
            cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        // SAFETY: Child owns the live process handle throughout this call.
        // The initialized counters buffer and its size match the Win32 ABI.
        let success =
            unsafe { GetProcessMemoryInfo(child.as_raw_handle(), &mut counters, counters.cb) };
        (success != 0).then_some(counters.PeakWorkingSetSize)
    }

    fn sample(executable: &Path, args: &[OsString]) -> Result<Sample, Box<dyn Error>> {
        let start = Instant::now();
        let mut child = configured(executable)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let mut peak = None;
        loop {
            if let Some(bytes) = peak_memory(&child) {
                peak = Some(peak.unwrap_or(0).max(bytes));
            }
            if let Some(status) = child.try_wait()? {
                if !status.success() {
                    return Err(format!("{} exited with {status}", executable.display()).into());
                }
                return Ok(Sample {
                    milliseconds: start.elapsed().as_secs_f64() * 1000.0,
                    peak_bytes: peak,
                });
            }
            if start.elapsed() > Duration::from_secs(30) {
                child.kill()?;
                child.wait()?;
                return Err(format!(
                    "{} exceeded the 30-second sample limit",
                    executable.display()
                )
                .into());
            }
            // Waiting on the process handle wakes immediately on exit. A Sleep(1)
            // polling loop can add a whole Windows scheduler tick to short runs.
            // SAFETY: Child owns a valid waitable process handle for this call.
            if unsafe { WaitForSingleObject(child.as_raw_handle(), 1) } == WAIT_FAILED {
                let error = std::io::Error::last_os_error();
                child.kill()?;
                child.wait()?;
                return Err(error.into());
            }
        }
    }

    fn generate(entries: usize) -> String {
        let mut text = String::from("; deterministic Windows benchmark\r\n");
        for section in (0..entries.div_ceil(100)).rev() {
            text.push_str(&format!("[section{section}]\r\n"));
            for item in (section * 100..((section + 1) * 100).min(entries)).rev() {
                text.push_str(&format!("key{item} = value{item} ; retained comment\r\n"));
            }
        }
        text
    }

    fn median(values: &mut [f64]) -> f64 {
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    }

    fn summary(samples: &[Sample], input_bytes: usize) -> (f64, String, String) {
        let mut times: Vec<_> = samples.iter().map(|s| s.milliseconds).collect();
        let time = median(&mut times);
        let mut memory: Vec<_> = samples
            .iter()
            .filter_map(|s| s.peak_bytes)
            .map(|b| b as f64 / 1_048_576.0)
            .collect();
        let memory = if memory.is_empty() {
            "N/A".into()
        } else {
            format!("{:.2}", median(&mut memory))
        };
        let throughput = if input_bytes == 0 {
            "N/A".into()
        } else {
            format!("{:.2}", input_bytes as f64 / 1_048_576.0 / (time / 1000.0))
        };
        (time, throughput, memory)
    }

    pub fn run() -> Result<(), Box<dyn Error>> {
        let mut reference = None;
        let mut rust = std::env::current_exe()?
            .parent()
            .and_then(Path::parent)
            .ok_or("Cannot find release directory")?
            .join("sortini.exe");
        let mut report_path = None;
        let mut iterations = 7;
        let mut args = std::env::args_os().skip(1);
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--reference") => {
                    reference = Some(PathBuf::from(
                        args.next().ok_or("Missing --reference executable")?,
                    ))
                }
                Some("--rust") => rust = args.next().ok_or("Missing --rust executable")?.into(),
                Some("--report") => {
                    report_path = Some(PathBuf::from(args.next().ok_or("Missing --report path")?))
                }
                Some("--iterations") => {
                    iterations = args
                        .next()
                        .ok_or("Missing --iterations count")?
                        .to_str()
                        .ok_or("Invalid count")?
                        .parse::<usize>()?
                }
                Some("--help") => {
                    println!(
                        "cargo run --release --example benchmark -- --reference C:\\Users\\kodyb\\Bin\\sortini_cs.exe [--rust path] [--iterations 7] [--report file.md]"
                    );
                    return Ok(());
                }
                _ => return Err(format!("Unknown argument: {}", arg.to_string_lossy()).into()),
            }
        }
        if !(3..=31).contains(&iterations) || iterations % 2 == 0 {
            return Err("Iterations must be odd and between 3 and 31".into());
        }
        let reference = reference.ok_or("Specify --reference executable")?;
        let reference_version = configured(&reference).arg("-v").output()?;
        if !reference_version.status.success() {
            return Err("Reference version command failed".into());
        }
        let rust_version = configured(&rust).arg("-v").output()?;
        if !rust_version.status.success() {
            return Err("Rust version command failed; build the release binary first".into());
        }
        let mut report = format!(
            "# Windows benchmark\n\nC#: `{}`\n\nRust: `{}`\n\nC# SHA-256: `{:x}`\n\nRust SHA-256: `{:x}`\n\nArchitecture: {}; logical CPUs available: {}.\n\n{} warmed samples per executable/workload, alternating executable order.\nSort output is validated byte-for-byte once before timing; timed stdout goes to NUL.\nBoth apps use explicit `--expand --stdout` because this C# build expands by default.\n\n| Workload | Input KiB | App | Median ms | MiB/s | Sampled peak working set MiB |\n| --- | ---: | --- | ---: | ---: | ---: |\n",
            String::from_utf8_lossy(&reference_version.stdout).trim(),
            String::from_utf8_lossy(&rust_version.stdout).trim(),
            Sha256::digest(fs::read(&reference)?),
            Sha256::digest(fs::read(&rust)?),
            std::env::consts::ARCH,
            thread::available_parallelism()?,
            iterations
        );
        let temporary = tempfile::tempdir()?;
        for (workload, entries) in [
            ("startup (-v)", 0),
            ("small", 16),
            ("medium", 2048),
            ("large", 100_000),
        ] {
            let input = generate(entries);
            let path = temporary.path().join("benchmark.ini");
            let arguments: Vec<OsString> = if entries == 0 {
                vec!["-v".into()]
            } else {
                fs::write(&path, &input)?;
                let oracle = configured(&reference)
                    .args(["--expand", "--stdout"])
                    .arg(&path)
                    .output()?;
                let actual = configured(&rust)
                    .args(["--expand", "--stdout"])
                    .arg(&path)
                    .output()?;
                if !oracle.status.success()
                    || !actual.status.success()
                    || oracle.stdout != actual.stdout
                {
                    return Err(format!(
                        "Output validation failed for {workload}; no timing comparison is valid"
                    )
                    .into());
                }
                vec!["--expand".into(), "--stdout".into(), path.into_os_string()]
            };
            // Explicit warmups exclude first extraction/JIT/file-cache effects.
            sample(&reference, &arguments)?;
            sample(&rust, &arguments)?;
            let mut reference_samples = Vec::new();
            let mut rust_samples = Vec::new();
            for iteration in 0..iterations {
                if iteration % 2 == 0 {
                    reference_samples.push(sample(&reference, &arguments)?);
                    rust_samples.push(sample(&rust, &arguments)?);
                } else {
                    rust_samples.push(sample(&rust, &arguments)?);
                    reference_samples.push(sample(&reference, &arguments)?);
                }
            }
            for (app, samples) in [("C#", reference_samples), ("Rust", rust_samples)] {
                let (time, throughput, memory) =
                    summary(&samples, if entries == 0 { 0 } else { input.len() });
                report.push_str(&format!(
                    "| {workload} | {:.2} | {app} | {time:.3} | {throughput} | {memory} |\n",
                    if entries == 0 {
                        0.0
                    } else {
                        input.len() as f64 / 1024.0
                    }
                ));
            }
        }
        report.push_str("\nWall time includes process startup and file reading; MiB/s is end-to-end, not an isolated parser benchmark. Memory is process working-set peak observed through GetProcessMemoryInfo at 1 ms intervals; short processes can exit between polls, so the reported peak can be an underestimate. Child-process memory is excluded. File writes and backup/fsync costs are not measured. Each sample is bounded to 30 seconds. These are synthetic, warmed local results, not universal speed claims.\n");
        if let Some(path) = report_path {
            fs::write(path, &report)?;
        }
        print!("{report}");
        Ok(())
    }
}
