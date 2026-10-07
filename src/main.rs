use sortini::{
    cli::{Command, Pause},
    encoding::EncodedText,
    persistence, sort_bytes, sort_text,
};
use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::Path,
};

fn main() -> std::process::ExitCode {
    let command = match Command::parse(std::env::args_os().skip(1), |name| std::env::var_os(name)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("sortini: {error}");
            return std::process::ExitCode::from(2);
        }
    };
    let code = run(&command);
    if (command.pause == Pause::Always || (command.pause == Pause::IfError && code != 0))
        && io::stdin().is_terminal()
    {
        // Never wait on piped INI input or put prompts into sorted stdout.
        eprintln!("Press Enter to exit.");
        let _ = io::stdin().read_line(&mut String::new());
    }
    std::process::ExitCode::from(code)
}

fn run(command: &Command) -> u8 {
    if let Some(info) = &command.information {
        let text = sortini::cli::information(info, |name| std::env::var_os(name));
        return match write_stdout(text.as_bytes()) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("sortini: {error}");
                13
            }
        };
    }
    if command.out_file.is_some() && command.files.len() > 1 {
        eprintln!("Cannot specify more than one input file when using the --out-file option.");
        return 99;
    }
    if command.files.is_empty() {
        if io::stdin().is_terminal() {
            eprintln!("Missing file. Use --help, or pipe INI data via stdin.");
            return 1;
        }
        let mut bytes = Vec::new();
        if let Err(error) = io::stdin().read_to_end(&mut bytes) {
            eprintln!("Error reading stdin: {error}");
            return 10;
        }
        return process(command, &bytes, command.out_file.as_deref(), true).unwrap_or_else(
            |(code, error)| {
                eprintln!("Error processing stdin: {error}");
                code
            },
        );
    }
    let mut first_error = 0;
    for path in &command.files {
        let result = fs::read(path)
            .map_err(|e| (10, e.to_string()))
            .and_then(|bytes| {
                process(
                    command,
                    &bytes,
                    Some(command.out_file.as_deref().unwrap_or(path)),
                    false,
                )
            });
        if let Err((code, error)) = result {
            if first_error == 0 {
                first_error = code;
            }
            eprintln!("Error processing '{}': {error}", path.display());
        }
    }
    first_error
}

fn process(
    command: &Command,
    bytes: &[u8],
    destination: Option<&Path>,
    stdin: bool,
) -> Result<u8, (u8, String)> {
    if command.stdout || (stdin && destination.is_none()) {
        let source = EncodedText::decode(bytes).map_err(|e| (10, e.to_string()))?;
        let output = sort_text(&source.text, &command.options);
        write_stdout(output.as_bytes()).map_err(|e| (13, e.to_string()))?;
    } else {
        let output = sort_bytes(bytes, &command.options).map_err(|e| (10, e.to_string()))?;
        persistence::save(
            destination.expect("file output requires a destination"),
            &output,
        )
        .map_err(|e| (13, e.to_string()))?;
    }
    Ok(0)
}

fn write_stdout(bytes: &[u8]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes)?;
    stdout.flush()
}
