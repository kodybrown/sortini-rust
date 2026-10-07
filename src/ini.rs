use crate::{OS_EOL, Options, compare::Comparison};

#[derive(Debug)]
enum Kind<'a> {
    Blank,
    Comment,
    Header(&'a str),
    Entry {
        key: &'a str,
        value: &'a str,
        disabled: Option<char>,
        comment: Option<(char, &'a str)>,
    },
    Plain,
    Malformed,
}

#[derive(Debug)]
struct Line<'a> {
    raw: &'a str,
    ending: &'a str,
    kind: Kind<'a>,
}

impl<'a> Line<'a> {
    fn parse(raw: &'a str, ending: &'a str) -> Self {
        let trimmed = raw.trim();
        let kind = if trimmed.is_empty() {
            Kind::Blank
        } else if let Some(rest) = trimmed.strip_prefix('[') {
            rest.find(']')
                .map_or(Kind::Malformed, |i| Kind::Header(rest[..i].trim()))
        } else if trimmed.starts_with([';', '#']) {
            let prefix = trimmed.chars().next().unwrap();
            let rest = &trimmed[1..];
            if !rest.starts_with(' ') && rest.contains('=') {
                entry(rest, Some(prefix))
            } else {
                Kind::Comment
            }
        } else if trimmed.find('=').is_some_and(|i| i > 0) {
            entry(trimmed, None)
        } else {
            Kind::Plain
        };
        Self { raw, ending, kind }
    }

    fn rendered(&self, options: &Options) -> String {
        if !options.normalize {
            return self.raw.to_owned();
        }
        match &self.kind {
            Kind::Header(name) => format!("[{name}]"),
            Kind::Entry {
                key,
                value,
                disabled,
                comment,
            } => {
                let equals = if options.expand_lines { " = " } else { "=" };
                let mut result = String::new();
                if let Some(prefix) = disabled {
                    result.push(*prefix);
                }
                result.push_str(&format!("{key}{equals}{value}"));
                if let Some((prefix, text)) = comment {
                    result.push_str(&format!(" {prefix} {text}"));
                }
                result
            }
            _ => self.raw.to_owned(),
        }
    }
}

fn entry(text: &str, disabled: Option<char>) -> Kind<'_> {
    let (key, value) = text.split_once('=').unwrap();
    let value = value.trim_start();
    let mut quote = None;
    let mut comment_at = None;
    for (i, c) in value.char_indices() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '\'' || c == '"' {
            quote = Some(c);
        } else if c == ' ' && value[i + 1..].starts_with([';', '#']) {
            comment_at = Some(i);
            break;
        }
    }
    let (value, comment) = if let Some(i) = comment_at {
        let rest = &value[i + 1..];
        (
            value[..i].trim_end(),
            Some((rest.chars().next().unwrap(), rest[1..].trim_start())),
        )
    } else {
        (value, None)
    };
    Kind::Entry {
        key: key.trim_end(),
        value,
        disabled,
        comment,
    }
}

struct Section<'a> {
    name: &'a str,
    header: Line<'a>,
    body: Vec<Line<'a>>,
}

pub(crate) struct Document<'a> {
    preamble: Vec<Line<'a>>,
    sections: Vec<Section<'a>>,
    eol: &'a str,
    final_newline: bool,
}

impl<'a> Document<'a> {
    pub fn parse(text: &'a str) -> Self {
        let mut result = Self {
            preamble: Vec::new(),
            sections: Vec::new(),
            eol: OS_EOL,
            final_newline: text.ends_with(['\r', '\n']),
        };
        let mut start = 0;
        let mut first_eol = true;
        while start < text.len() {
            let end = text[start..]
                .find(['\r', '\n'])
                .map_or(text.len(), |i| start + i);
            let next = if text[end..].starts_with("\r\n") {
                end + 2
            } else if end < text.len() {
                end + 1
            } else {
                end
            };
            let ending = &text[end..next];
            if first_eol && !ending.is_empty() {
                result.eol = ending;
                first_eol = false;
            }
            let line = Line::parse(&text[start..end], ending);
            if let Kind::Header(name) = line.kind {
                result.sections.push(Section {
                    name,
                    header: line,
                    body: Vec::new(),
                });
            } else if let Some(section) = result.sections.last_mut() {
                section.body.push(line);
            } else {
                result.preamble.push(line);
            }
            start = next;
        }
        result
    }

    pub fn sort(&mut self, options: &Options) {
        let sections = Comparison::new(options.ignore_case_sections, options.natural);
        let entries = Comparison::new(options.ignore_case_entries, options.natural);
        let direction = |order: std::cmp::Ordering| {
            if options.descending {
                order.reverse()
            } else {
                order
            }
        };
        if options.sort_sections {
            // Stable sorts deliberately preserve equal-key ordering and duplicate sections.
            self.sections
                .sort_by(|a, b| direction(sections.compare(a.name, b.name)));
        }
        if options.sort_entries {
            for section in &mut self.sections {
                let mut groups = Vec::new();
                let mut leading = Vec::new();
                for line in std::mem::take(&mut section.body) {
                    if matches!(line.kind, Kind::Entry { .. }) {
                        groups.push((std::mem::take(&mut leading), line));
                    } else {
                        leading.push(line);
                    }
                }
                groups.sort_by(|(_, a), (_, b)| {
                    let Kind::Entry {
                        key: ak, value: av, ..
                    } = a.kind
                    else {
                        unreachable!()
                    };
                    let Kind::Entry {
                        key: bk, value: bv, ..
                    } = b.kind
                    else {
                        unreachable!()
                    };
                    direction(
                        entries
                            .compare(ak, bk)
                            .then_with(|| entries.compare(av, bv)),
                    )
                });
                for (before, line) in groups {
                    section.body.extend(before);
                    section.body.push(line);
                }
                section.body.extend(leading);
            }
        }
    }

    pub fn export(&self, options: &Options) -> String {
        let eol = options.eol.as_deref().unwrap_or(self.eol);
        let mut rendered = Vec::new();
        let mut add = |line: &Line<'a>| {
            if !options.normalize || !matches!(line.kind, Kind::Blank) {
                let ending = if options.normalize || options.eol.is_some() || line.ending.is_empty()
                {
                    eol
                } else {
                    line.ending
                };
                rendered.push((line.rendered(options), ending));
            }
        };
        for line in &self.preamble {
            add(line);
        }
        // End the closure borrow before adding synthetic section separators.
        for (i, section) in self.sections.iter().enumerate() {
            if options.normalize && options.expand_sections && (i > 0 || !rendered.is_empty()) {
                rendered.push((String::new(), eol));
            }
            for line in std::iter::once(&section.header).chain(&section.body) {
                if options.normalize && matches!(line.kind, Kind::Blank) {
                    continue;
                }
                let ending = if options.normalize || options.eol.is_some() || line.ending.is_empty()
                {
                    eol
                } else {
                    line.ending
                };
                rendered.push((line.rendered(options), ending));
            }
        }
        let mut output = String::new();
        let mut lines = rendered.into_iter().peekable();
        while let Some((raw, ending)) = lines.next() {
            output.push_str(&raw);
            // EOF is a document property: moving the original last line must not
            // concatenate two entries or introduce a final newline in raw mode.
            if options.normalize || self.final_newline || lines.peek().is_some() {
                output.push_str(ending);
                // Moving an LF blank line after a CR-ending entry can otherwise
                // merge two physical endings into one CRLF and lose that blank.
                // Widen just this boundary's CR to CRLF, leaving the blank's LF.
                if ending == "\r"
                    && lines.peek().is_some_and(|(next_raw, next_ending)| {
                        next_raw.is_empty() && *next_ending == "\n"
                    })
                {
                    output.push('\n');
                }
            }
        }
        output
    }
}
