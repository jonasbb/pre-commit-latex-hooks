use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use slug::slugify;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

type Error = Box<dyn std::error::Error + 'static>;

static RE_SECTIONS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?mx) # Enable multiline and ignore whitespace mode

        # Match whitespace but no newline
        # https://stackoverflow.com/questions/3469080/match-whitespace-but-not-newlines
        ^[^\S\n]* # Eat leading whitespace

        \\(?P<section_type>(?:sub|subsub)?section)\*?\ *
        (?:
            \{
                # Section content
                (?P<section_content>
                    (?:
                    [^\{\}]* |
                    # Parse single nested {} blocks
                    (?:\{[^\{\}]*\})* |
                    # Parse double nested {} blocks
                    (?:\{ [^\{\}]*
                        (?:\{[^\{\}]*\} [^\{\}]*)*
                    \})*
                    )+
                )
            \}
            [^\S\n]* # Eat trailing spaces
            (?P<comment>%[^\n]*)? # Eat optional comment
            (?:$\n^)? # Optional linebreak

            (?:
                [^\S\n]* # Eat leading whitespace
                \\label\{
                    # Label content
                    (?P<label>.*)
                \}$
            )?
        |
            (?P<unparsable_section>.+$)?
        )
        "#,
    )
    .unwrap()
});

/// Match a LaTeX Command with 1 or 2 required arquments.
static RE_LATEX_COMMAND: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?x) # Ignore whitespace mode
        # Parse \ ident {
        \\\w+\{
            (?P<first_arg>
            [^\{\}]*
            (?:\{[^\{\}]*\} [^\{\}]*)*
            )
        \}
        # Optional second argument to LaTeX command
        (?:\{
            [^\{\}]*
            (?:\{[^\{\}]*\} [^\{\}]*)*
        \})?
        "#,
    )
    .unwrap()
});

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Capture<'a> {
    offset: usize,

    /// String matching the section command, e.g., "subsection"
    section_type: Option<&'a str>,
    /// String matching the content of the section command
    section_content: Option<&'a str>,
    /// Optional comment on the same line as the section command
    comment: Option<&'a str>,
    ///
    label: Option<&'a str>,
    unparsable_section: Option<&'a str>,
}

impl<'a> From<regex::Captures<'a>> for Capture<'a> {
    fn from(capture: regex::Captures<'a>) -> Self {
        Self {
            offset: capture
                .get(0)
                .expect("A capture group 0 always exists as the full match.")
                .start(),
            section_type: capture.name("section_type").map(|m| m.as_str()),
            section_content: capture.name("section_content").map(|m| m.as_str()),
            comment: capture.name("comment").map(|m| m.as_str()),
            label: capture.name("label").map(|m| m.as_str()),
            unparsable_section: capture.name("unparsable_section").map(|m| m.as_str()),
        }
    }
}


#[derive(Clone, Debug, StructOpt)]
#[structopt(global_settings(&[
    structopt::clap::AppSettings::ColoredHelp,
    structopt::clap::AppSettings::VersionlessSubcommands
]))]
struct CliArgs {
    files: Vec<PathBuf>,
}

enum FileStatus {
    FoundLabelMismatch,
    AllLabelsMatch,
}

fn slugify_label(section_type: &str, content: String) -> String {
    let prefix = match section_type {
        "section" => "sec",
        "subsection" => "ssec",
        "subsubsection" => "sssec",
        _ => "unknwn",
    };

    // Remove embedded LaTeX commands in the content part.
    // Iterate until we reach a fixpoint
    let mut new_content = content;
    let mut content = String::new();
    while content != new_content {
        content = new_content;
        new_content = RE_LATEX_COMMAND
            .replace_all(&content, |capture: &Captures| -> String {
                capture.name("first_arg").unwrap().as_str().to_string()
            })
            .to_string();
    }
    content = new_content;

    format!("{}:{}", prefix, slugify(content))
}

fn main() {
    let cli_args = CliArgs::from_args();

    let mut has_error = false;

    for path in &cli_args.files {
        match process_file(path) {
            Ok(FileStatus::FoundLabelMismatch) => has_error = true,
            Ok(FileStatus::AllLabelsMatch) => {}
            Err(err) => {
                has_error = true;
                eprintln!("Error in file {}\n  {}", path.display(), err);

                let mut err = &*err;
                while let Some(cause) = err.source() {
                    eprintln!("  Caused by: {}", cause);
                    err = cause;
                }
            }
        }
    }

    if has_error {
        std::process::exit(1);
    }
}

fn process_file(file: &Path) -> Result<FileStatus, Error> {
    let mut found_mismatch = false;
    let text = std::fs::read_to_string(file)?;

    RE_SECTIONS.captures_iter(&text).for_each(|capture| {
        let capture: Capture = capture.into();
        let line_number = offset_to_line_number(&*text, capture.offset);

        if let Some(_unparsable_section) = capture.unparsable_section {
            println!("{}:{} Unprocessable Section", file.display(), line_number,);
        } else {
            let section_type = capture
                .section_type
                .expect("A section_type must exist if the regex is parsable.");
            let section_content = capture
                .section_content
                .expect("A section_type must exist if the regex is parsable.");
            let slug = slugify_label(section_type, section_content.to_string());

            match capture.label {
                None => {
                    found_mismatch = true;
                    println!(
                        "{}:{} Missing Label, use \\label{{{}}}",
                        file.display(),
                        line_number,
                        slug
                    );
                }
                Some(label) => {
                    if label != slug
                        && !capture
                            .comment
                            .map(|cmt| cmt.contains("skip-label"))
                            .unwrap_or(false)
                    {
                        let line_number = offset_to_line_number(&*text, capture.offset);
                        found_mismatch = true;
                        println!(
                            "{}:{} Wrong Label '{}', use \\label{{{}}}",
                            file.display(),
                            line_number,
                            label,
                            slug
                        );
                    }
                }
            }
        }
    });

    if found_mismatch {
        Ok(FileStatus::FoundLabelMismatch)
    } else {
        Ok(FileStatus::AllLabelsMatch)
    }
}

#[cfg(test)]
mod test_regex {
    use super::*;
    use pretty_assertions::assert_eq;

    /// Parse a lone section
    #[test]
    fn only_section() {
        let text = r##"\section{Hello World}"##;
        let captures: Capture = RE_SECTIONS
            .captures(text)
            .expect("Regex needs to match")
            .into();
        let expected = Capture {
            offset: 0,
            section_type: Some("section"),
            section_content: Some("Hello World"),
            comment: None,
            label: None,
            unparsable_section: None,
        };
        assert_eq!(captures, expected);
    }

    /// Parse a section with comment
    #[test]
    fn only_section_with_comment() {
        let text = r##"\section{Hello World} % Comment"##;
        let captures: Capture = RE_SECTIONS
            .captures(text)
            .expect("Regex needs to match")
            .into();
        let expected = Capture {
            offset: 0,
            section_type: Some("section"),
            section_content: Some("Hello World"),
            comment: Some("% Comment"),
            label: None,
            unparsable_section: None,
        };
        assert_eq!(captures, expected);
    }

    #[test]
    fn section_and_label() {
        let text = r##"\section{Hello World}
\label{Label-ABC}"##;
        let captures: Capture = RE_SECTIONS
            .captures(text)
            .expect("Regex needs to match")
            .into();
        let expected = Capture {
            offset: 0,
            section_type: Some("section"),
            section_content: Some("Hello World"),
            comment: None,
            label: Some("Label-ABC"),
            unparsable_section: None,
        };
        assert_eq!(captures, expected);
    }

    /// Parse a section and comment and label
    #[test]
    fn section_with_comment_and_label() {
        let text = r##"\section{Hello World} % Another Comment
\label{Here}"##;
        let captures: Capture = RE_SECTIONS
            .captures(text)
            .expect("Regex needs to match")
            .into();
        let expected = Capture {
            offset: 0,
            section_type: Some("section"),
            section_content: Some("Hello World"),
            comment: Some("% Another Comment"),
            label: Some("Here"),
            unparsable_section: None,
        };
        assert_eq!(captures, expected);
    }

    /// Put section and label on the same line
    #[test]
    fn section_and_label_same_line() {
        let text = r##"\section{Hello World} \label{Label-123}"##;
        let captures: Capture = RE_SECTIONS
            .captures(text)
            .expect("Regex needs to match")
            .into();
        let expected = Capture {
            offset: 0,
            section_type: Some("section"),
            section_content: Some("Hello World"),
            comment: None,
            label: Some("Label-123"),
            unparsable_section: None,
        };
        assert_eq!(captures, expected);
    }

    /// Check for `\section*`
    #[test]
    fn section_star_and_label() {
        let text = r##"

\section*{Hello World}
\label{Label-ABC}"##;
        let captures: Capture = RE_SECTIONS
            .captures(text)
            .expect("Regex needs to match")
            .into();
        let expected = Capture {
            offset: 2,
            section_type: Some("section"),
            section_content: Some("Hello World"),
            comment: None,
            label: Some("Label-ABC"),
            unparsable_section: None,
        };
        assert_eq!(captures, expected);
    }

    /// Check parsing a single latex command in section
    #[test]
    fn section_with_nested_command() {
        let text = r##"\section{\textbf{bold}}"##;
        let captures: Capture = RE_SECTIONS
            .captures(text)
            .expect("Regex needs to match")
            .into();
        let expected = Capture {
            offset: 0,
            section_type: Some("section"),
            section_content: Some("\\textbf{bold}"),
            comment: None,
            label: None,
            unparsable_section: None,
        };
        assert_eq!(captures, expected);
    }

    /// Check parsing multiple nested latex commands in section
    #[test]
    fn section_with_double_nested_command_and_label() {
        let text = r##"\subsubsection{Formalization of \texorpdfstring{\acs{knn}}{k-NN}}
\label{sssec:formalization-of-knn}"##;
        let captures: Capture = RE_SECTIONS
            .captures(text)
            .expect("Regex needs to match")
            .into();
        let expected = Capture {
            offset: 0,
            section_type: Some("subsubsection"),
            section_content: Some(r"Formalization of \texorpdfstring{\acs{knn}}{k-NN}"),
            comment: None,
            label: Some("sssec:formalization-of-knn"),
            unparsable_section: None,
        };
        assert_eq!(captures, expected);
    }

    /// Check using a subsection
    #[test]
    fn only_subsection() {
        let text = r##"\subsection{SubSec}"##;
        let captures: Capture = RE_SECTIONS.captures(text).unwrap().into();
        let expected = Capture {
            offset: 0,
            section_type: Some("subsection"),
            section_content: Some("SubSec"),
            comment: None,
            label: None,
            unparsable_section: None,
        };
        assert_eq!(captures, expected);
    }

    /// Test if we can handle things outside of our current regex
    #[test]
    fn unsupported_section_content() {
        let text = r##"\subsection{A{B{C{D{EE}D}C}B}A}"##;
        let captures: Capture = RE_SECTIONS.captures(text).unwrap().into();
        let expected = Capture {
            offset: 0,
            section_type: Some("subsection"),
            section_content: None,
            comment: None,
            label: None,
            unparsable_section: Some("{A{B{C{D{EE}D}C}B}A}"),
        };
        assert_eq!(captures, expected);
    }
}

#[cfg(test)]
mod test_slugify_label {
    use super::*;

    #[test]
    fn simple_ascii() {
        assert_eq!(slugify_label("section", "Word".to_string()), "sec:word");
        assert_eq!(
            slugify_label("section", "Hello World".to_string()),
            "sec:hello-world"
        );
        assert_eq!(
            slugify_label("subsubsection", "Many Many words here".to_string()),
            "sssec:many-many-words-here"
        );
    }

    #[test]
    fn nested_commands() {
        assert_eq!(
            slugify_label("section", r"\texttt{Abc}".to_string()),
            "sec:abc"
        );
        assert_eq!(
            slugify_label("subsection", r"Something \emph{very} important".to_string()),
            "ssec:something-very-important"
        );
    }

    #[test]
    fn double_nested_commands() {
        assert_eq!(
            slugify_label(
                "subsubsection",
                r"Formalization of \texorpdfstring{\acs{knn}}{k-NN}".to_string()
            ),
            "sssec:formalization-of-knn"
        );
    }
}

fn offset_to_line_number(text: &str, offset: usize) -> u32 {
    if offset > text.len() {
        panic!("ERROR");
    }

    let mut line_number = 1;
    for (idx, c) in text.char_indices() {
        if idx >= offset {
            return line_number;
        }

        if c == '\n' {
            line_number += 1;
        }
    }

    panic!("This shouldn't happen as we check offset before");
}

#[cfg(test)]
mod test_offset_to_line_number {
    use super::*;

    #[test]
    fn simple_ascii() {
        let text = r#"Hello
Nice
World
"#;
        assert_eq!(offset_to_line_number(text, 0), 1);
        assert_eq!(offset_to_line_number(text, 1), 1);
        assert_eq!(offset_to_line_number(text, 2), 1);
        assert_eq!(offset_to_line_number(text, 3), 1);
        assert_eq!(offset_to_line_number(text, 4), 1);
        assert_eq!(offset_to_line_number(text, 5), 1);

        assert_eq!(offset_to_line_number(text, 6), 2);
        assert_eq!(offset_to_line_number(text, 7), 2);
        assert_eq!(offset_to_line_number(text, 8), 2);
        assert_eq!(offset_to_line_number(text, 9), 2);
        assert_eq!(offset_to_line_number(text, 10), 2);

        assert_eq!(offset_to_line_number(text, 11), 3);
        assert_eq!(offset_to_line_number(text, 12), 3);
        assert_eq!(offset_to_line_number(text, 13), 3);
        assert_eq!(offset_to_line_number(text, 14), 3);
        assert_eq!(offset_to_line_number(text, 15), 3);
        assert_eq!(offset_to_line_number(text, 16), 3);
    }
}
