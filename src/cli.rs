//! The small table-driven parser retains sortini's long single-dash and slash flags.
//! Paths remain OsStrings; no lossy Unicode conversion is used for filenames.

use crate::{OS_EOL, Options, encoding::Encoding};
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Pause {
    #[default]
    Never,
    Always,
    IfError,
}

#[derive(Debug, Default)]
pub struct Command {
    pub options: Options,
    pub files: Vec<PathBuf>,
    pub out_file: Option<PathBuf>,
    pub stdout: bool,
    pub quiet: bool,
    pub pause: Pause,
    pub information: Option<Information>,
}

#[derive(Debug)]
pub enum Information {
    Help(String),
    Examples,
    Envars,
    About,
    Version,
    VersionFull,
}

#[derive(Clone, Copy, PartialEq)]
enum Value {
    Bool,
    Required,
    Help,
    Pause,
}

struct Spec {
    names: &'static [&'static str],
    value: Value,
    env: bool,
    description: &'static str,
}

macro_rules! spec {
    ($kind:ident, $env:literal, [$($name:literal),+], $description:literal) => {
        Spec { names: &[$($name),+], value: Value::$kind, env: $env, description: $description }
    };
}

// Keep environment application in the C# AppOptions declaration order. Combined
// switches apply before their more specific section/entry counterparts.
static SPECS: &[Spec] = &[
    spec!(
        Help,
        false,
        ["help", "?", "h"],
        "Show help, or help for a named option/topic."
    ),
    spec!(Bool, false, ["about"], "Show full app details."),
    spec!(Bool, false, ["v"], "Show version (short)."),
    spec!(Bool, false, ["version"], "Show full version details."),
    spec!(
        Bool,
        true,
        ["quiet"],
        "Limits informational output; errors remain visible."
    ),
    spec!(
        Pause,
        true,
        ["pause"],
        "Pause before exiting: never, always, iferror (bare flag: always)."
    ),
    spec!(Bool, false, ["show-examples", "examples"], "Show examples."),
    spec!(
        Bool,
        false,
        ["show-envars", "envars"],
        "Show supported environment variables and current values."
    ),
    spec!(
        Required,
        false,
        ["o", "out-file"],
        "Write to a separate file; only one input file is allowed."
    ),
    spec!(
        Bool,
        false,
        ["stdout", "redirect-to-stdout"],
        "Write sorted text to stdout without changing the input file."
    ),
    spec!(
        Bool,
        true,
        ["sort"],
        "Sort sections and entries (default: true)."
    ),
    spec!(
        Bool,
        true,
        ["sort-sections"],
        "Sort sections (default: true)."
    ),
    spec!(
        Bool,
        true,
        ["sort-entries"],
        "Sort entries in each section (default: true)."
    ),
    spec!(Bool, false, ["sort-asc"], "Sort ascending (default)."),
    spec!(Bool, false, ["sort-desc"], "Sort descending."),
    spec!(
        Bool,
        true,
        ["natural"],
        "Natural sorting of numbers and numeric suffixes (default: true)."
    ),
    spec!(Bool, true, ["ordinal"], "Disable natural numeric sorting."),
    spec!(
        Bool,
        true,
        ["ignore-case", "i"],
        "Ignore case for sections and entries (default)."
    ),
    spec!(
        Bool,
        true,
        ["case-sensitive", "t"],
        "Compare sections and entries with case sensitivity."
    ),
    spec!(
        Bool,
        true,
        ["ignore-case-sections"],
        "Ignore case for sections only."
    ),
    spec!(
        Bool,
        true,
        ["ignore-case-entries"],
        "Ignore case for entries only."
    ),
    spec!(
        Bool,
        true,
        ["case-sensitive-sections"],
        "Use case-sensitive section comparison."
    ),
    spec!(
        Bool,
        true,
        ["case-sensitive-entries"],
        "Use case-sensitive entry comparison."
    ),
    spec!(
        Bool,
        false,
        ["expand"],
        "Add spaces around '=' and blank lines between sections."
    ),
    spec!(
        Bool,
        false,
        ["compact"],
        "Remove spaces around '=' and blank lines between sections."
    ),
    spec!(Bool, true, ["expand-lines"], "Add spaces around '='."),
    spec!(
        Bool,
        false,
        ["compact-lines"],
        "Remove spaces around '=' (same as -!expand-lines)."
    ),
    spec!(
        Bool,
        true,
        ["expand-sections"],
        "Add blank lines between sections."
    ),
    spec!(
        Bool,
        false,
        ["compact-sections"],
        "Remove blank lines between sections (same as -!expand-sections)."
    ),
    spec!(
        Required,
        true,
        ["encoding"],
        "Output encoding: utf8, ascii, unicode, utf32, os; default: source."
    ),
    spec!(
        Required,
        true,
        ["eol", "end-of-line"],
        "Output EOL: \\n, \\r\\n, \\r, os; default: source."
    ),
];

fn named(text: &str) -> Option<(&Spec, bool, Option<&str>)> {
    let rest = text
        .strip_prefix("--")
        .or_else(|| text.strip_prefix('-'))
        .or_else(|| text.strip_prefix('/'))?;
    let (rest, positive) = rest.strip_prefix('!').map_or((rest, true), |s| (s, false));
    let (name, inline) = rest
        .split_once('=')
        .map_or((rest, None), |(n, v)| (n, Some(v)));
    SPECS
        .iter()
        .find(|s| s.names.iter().any(|n| n.eq_ignore_ascii_case(name)))
        .map(|s| (s, positive, inline))
}

impl Command {
    /// Environment lookup is injected so callers and tests need not mutate process state.
    pub fn parse(
        args: impl IntoIterator<Item = OsString>,
        environment: impl Fn(&str) -> Option<OsString>,
    ) -> Result<Self, String> {
        let args: Vec<_> = args.into_iter().collect();
        let mut command = Self::default();
        let mut actions = Vec::new();
        let mut i = 0;
        let mut literal = false;
        while i < args.len() {
            let token = &args[i];
            if !literal && token == "--" {
                literal = true;
                i += 1;
                continue;
            }
            let parsed = if literal {
                None
            } else {
                token.to_str().and_then(named)
            };
            if let Some((spec, positive, inline)) = parsed {
                if !positive && spec.value != Value::Bool {
                    return Err(format!(
                        "'!' can only negate boolean options: {}",
                        spec.names[0]
                    ));
                }
                let value = if let Some(inline) = inline {
                    OsString::from(inline)
                } else if spec.value == Value::Bool {
                    OsString::from(if positive { "true" } else { "false" })
                } else if i + 1 < args.len()
                    && (spec.value == Value::Required || !looks_like_option(&args[i + 1]))
                {
                    i += 1;
                    args[i].clone()
                } else {
                    match spec.value {
                        Value::Help => OsString::new(),
                        Value::Pause => OsString::from("always"),
                        _ => return Err(format!("Missing value for argument: {}", spec.names[0])),
                    }
                };
                if !positive && inline.is_some() {
                    return Err("Use either '!' or '=false', not both".into());
                }
                actions.push((spec, value));
            } else {
                if !literal && token.to_str().is_some_and(|t| t.starts_with('-')) {
                    return Err(format!(
                        "Unknown option: {} (options cannot be chained)",
                        token.to_string_lossy()
                    ));
                }
                command.files.push(PathBuf::from(token));
            }
            i += 1;
        }
        for spec in SPECS.iter().filter(|s| s.env) {
            if actions
                .iter()
                .any(|(explicit, _)| std::ptr::eq(*explicit, spec))
            {
                continue;
            }
            let name = env_name(spec);
            if let Some(value) = environment(&name).filter(|v| !v.is_empty()) {
                command
                    .apply(spec, &value)
                    .map_err(|e| format!("{name}: {e}"))?;
            }
        }
        for (spec, value) in actions {
            command.apply(spec, &value)?;
        }
        Ok(command)
    }

    fn apply(&mut self, spec: &Spec, value: &OsStr) -> Result<(), String> {
        let name = spec.names[0];
        if name == "o" {
            if value.is_empty() {
                return Err("Output filename cannot be empty".into());
            }
            self.out_file = Some(value.into());
            return Ok(());
        }
        let value = value
            .to_str()
            .ok_or_else(|| format!("Value for {name} must be valid Unicode"))?;
        let boolean = if spec.value == Value::Bool {
            parse_bool(value)?
        } else {
            false
        };
        let options = &mut self.options;
        match name {
            "help" => {
                self.information = Some(match value {
                    "examples" | "show-examples" => Information::Examples,
                    "envars" | "show-envars" => Information::Envars,
                    _ => Information::Help(value.into()),
                })
            }
            "about" if boolean => self.information = Some(Information::About),
            "v" if boolean => self.information = Some(Information::Version),
            "version" if boolean => self.information = Some(Information::VersionFull),
            "show-examples" if boolean => self.information = Some(Information::Examples),
            "show-envars" if boolean => self.information = Some(Information::Envars),
            "quiet" => self.quiet = boolean,
            "pause" => {
                self.pause = match value.to_ascii_lowercase().as_str() {
                    "never" => Pause::Never,
                    "always" => Pause::Always,
                    "iferror" => Pause::IfError,
                    _ => {
                        return Err(format!(
                            "Invalid value '{value}' for pause. Allowed values: never, always, iferror"
                        ));
                    }
                }
            }
            "stdout" => self.stdout = boolean,
            "sort" => {
                options.sort_sections = boolean;
                options.sort_entries = boolean;
            }
            "sort-sections" => options.sort_sections = boolean,
            "sort-entries" => options.sort_entries = boolean,
            "sort-asc" if boolean => options.descending = false,
            "sort-desc" if boolean => options.descending = true,
            "natural" => options.natural = boolean,
            "ordinal" if boolean => options.natural = false,
            "ignore-case" if boolean => {
                options.ignore_case_sections = true;
                options.ignore_case_entries = true;
            }
            "case-sensitive" if boolean => {
                options.ignore_case_sections = false;
                options.ignore_case_entries = false;
            }
            "ignore-case-sections" if boolean => options.ignore_case_sections = true,
            "ignore-case-entries" if boolean => options.ignore_case_entries = true,
            "case-sensitive-sections" if boolean => options.ignore_case_sections = false,
            "case-sensitive-entries" if boolean => options.ignore_case_entries = false,
            "expand" | "compact" => {
                let expanded = if name == "compact" { !boolean } else { boolean };
                options.normalize = true;
                options.expand_lines = expanded;
                options.expand_sections = expanded;
            }
            "expand-lines" | "compact-lines" => {
                options.normalize = true;
                options.expand_lines = if name == "compact-lines" {
                    !boolean
                } else {
                    boolean
                };
            }
            "expand-sections" | "compact-sections" => {
                options.normalize = true;
                options.expand_sections = if name == "compact-sections" {
                    !boolean
                } else {
                    boolean
                };
            }
            "encoding" => options.encoding = Some(Encoding::from_name(value)?),
            "eol" => options.eol = Some(
                match value.to_ascii_lowercase().as_str() {
                    "\\n" => "\n",
                    "\\r\\n" => "\r\n",
                    "\\r" => "\r",
                    "os" => OS_EOL,
                    _ => {
                        return Err(format!(
                            "Invalid value '{value}' for eol. Allowed values: \\n, \\r\\n, \\r, os"
                        ));
                    }
                }
                .into(),
            ),
            _ => {} // False one-way switches intentionally have no effect, as in C#.
        }
        Ok(())
    }
}

fn parse_bool(text: &str) -> Result<bool, String> {
    match text.trim().to_ascii_lowercase().as_str() {
        "true" | "t" | "yes" | "y" | "1" => Ok(true),
        "false" | "f" | "no" | "n" | "0" => Ok(false),
        _ => Err(format!("Invalid boolean value '{text}'; use true or false")),
    }
}

fn looks_like_option(text: &OsStr) -> bool {
    text.to_str()
        .is_some_and(|s| s.starts_with('-') || named(s).is_some())
}

fn env_name(spec: &Spec) -> String {
    format!("sortini_{}", spec.names[0].replace('-', "_"))
}

pub fn information(info: &Information, environment: impl Fn(&str) -> Option<OsString>) -> String {
    let header = format!(
        "sortini v{} (Rust)\nCopyright (c) 2013-2026 Kody Brown\n",
        env!("CARGO_PKG_VERSION")
    );
    match info {
        Information::Version => format!("{}\n", env!("CARGO_PKG_VERSION")),
        Information::VersionFull => {
            format!("{header}Repository: {}\n", env!("CARGO_PKG_REPOSITORY"))
        }
        Information::About => format!(
            "{header}A Rust command-line utility that sorts INI files.\nPreserves comments, spacing, encoding, BOM, and EOL by default.\nAuthor: Kody Brown (@kodybrown)\nLicense: MIT\nRepository: {}\n",
            env!("CARGO_PKG_REPOSITORY")
        ),
        Information::Examples => format!(
            "{header}\nEXAMPLES:\n---------\n\nsortini settings.ini\nsortini settings.ini -o sorted.ini\nsortini -stdout -sort-desc settings.ini\nsortini -!sort-sections settings.ini\nsortini -expand settings.ini\nsortini -case-sensitive-entries settings.ini\nsortini -encoding utf8 -eol \\n settings.ini\ntype settings.ini | sortini\n"
        ),
        Information::Envars => {
            let mut output =
                format!("{header}\nENVIRONMENT VARIABLES (CLI overrides environment):\n");
            for spec in SPECS.iter().filter(|s| s.env) {
                let name = env_name(spec);
                let value = environment(&name)
                    .map_or("(unset)".into(), |v| v.to_string_lossy().into_owned());
                output.push_str(&format!("  {name} = {value}\n    {}\n", spec.description));
            }
            output
        }
        Information::Help(topic) => {
            let mut output = format!(
                "{header}\nUSAGE:\n------\nsortini [options] \"files\" [\"files\"] [...]\n\nFiles are optional when INI data is piped via stdin.\n\nOPTIONS:\n"
            );
            for spec in SPECS {
                if !topic.is_empty()
                    && topic != "files"
                    && !spec
                        .names
                        .iter()
                        .any(|n| n.eq_ignore_ascii_case(topic.trim_start_matches(['-', '/', '!'])))
                {
                    continue;
                }
                let names = spec
                    .names
                    .iter()
                    .map(|n| format!("-{n}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let value = match spec.value {
                    Value::Bool => "",
                    Value::Required => " [string]",
                    Value::Help => " [topic]",
                    Value::Pause => " [never|always|iferror]",
                };
                output.push_str(&format!("  {names}{value}\n    {}\n", spec.description));
            }
            output.push_str("\nNotes:\n- Option prefixes: '-', '--', or '/' (e.g. -help, --help, /help).\n- Options cannot be chained; use separate flags.\n- Use '!' to set a boolean false (e.g. -!sort or /!natural).\n- '--' ends option parsing for filenames that resemble flags.\n- CLI flags override sortini_* environment variables; later flags win.\n- Stdin defaults to stdout; --out-file writes to disk.\n- --stdout emits UTF-8 text; --encoding applies to file output.\n");
            output
        }
    }
}
