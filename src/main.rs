#[cfg(not(unix))]
compile_error!("markdownlector supports Unix-like systems only; Windows is not supported.");

mod input;
mod markdown;
mod plain;

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use input::InputSource;

const PROGRAM: &str = "markdownlector";

const HELP: &str = "\
markdownlector — a reading appliance for Markdown

Usage:
    markdownlector <file>
    markdownlector -            read from stdin
    markdownlector --help
    markdownlector --version

Options:
    -h, --help       Show this message
    -V, --version    Show version
";

/// A parsed command line.
enum Command {
    Render(InputSource),
    Help,
    Version,
}

/// A command-line usage error.
enum ArgError {
    MissingInput,
    TooManyArgs,
    UnknownFlag(String),
}

impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgError::MissingInput => {
                write!(f, "no input given; pass a file, '-' for stdin, or --help")
            }
            ArgError::TooManyArgs => write!(f, "too many arguments; expected a single file or '-'"),
            ArgError::UnknownFlag(flag) => write!(f, "unknown option '{flag}'; see --help"),
        }
    }
}

fn parse_args(args: &[String]) -> Result<Command, ArgError> {
    // Flags take precedence over positional input and may appear anywhere;
    // --help is checked before --version, so it wins if both are present.
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        return Ok(Command::Help);
    }
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-V" | "--version"))
    {
        return Ok(Command::Version);
    }

    let mut source: Option<InputSource> = None;
    for arg in args {
        let candidate = match arg.as_str() {
            "-" => InputSource::Stdin,
            flag if flag.starts_with('-') => return Err(ArgError::UnknownFlag(arg.clone())),
            path => InputSource::File(PathBuf::from(path)),
        };
        if source.is_some() {
            return Err(ArgError::TooManyArgs);
        }
        source = Some(candidate);
    }

    source.map(Command::Render).ok_or(ArgError::MissingInput)
}

/// Write `text` to stdout, treating a closed pipe (e.g. `… | head`) as a clean
/// stop rather than a panic. A reading appliance must pipe without crashing.
fn emit(text: &str) -> ExitCode {
    let mut stdout = io::stdout().lock();
    match stdout
        .write_all(text.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{PROGRAM}: {error}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match parse_args(&args) {
        Ok(Command::Help) => emit(HELP),
        Ok(Command::Version) => emit(&format!(
            "{} {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )),
        Ok(Command::Render(source)) => match input::load(&source) {
            Ok(text) => emit(&plain::render(&text)),
            Err(error) => {
                eprintln!("{PROGRAM}: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("{PROGRAM}: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command, ArgError> {
        let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        parse_args(&owned)
    }

    #[test]
    fn parses_file() {
        assert!(matches!(
            parse(&["README.md"]),
            Ok(Command::Render(InputSource::File(_)))
        ));
    }

    #[test]
    fn parses_stdin_dash() {
        assert!(matches!(
            parse(&["-"]),
            Ok(Command::Render(InputSource::Stdin))
        ));
    }

    #[test]
    fn parses_help_long_and_short() {
        assert!(matches!(parse(&["--help"]), Ok(Command::Help)));
        assert!(matches!(parse(&["-h"]), Ok(Command::Help)));
    }

    #[test]
    fn parses_version_long_and_short() {
        assert!(matches!(parse(&["--version"]), Ok(Command::Version)));
        assert!(matches!(parse(&["-V"]), Ok(Command::Version)));
    }

    #[test]
    fn rejects_too_many_arguments() {
        assert!(matches!(
            parse(&["a.md", "b.md"]),
            Err(ArgError::TooManyArgs)
        ));
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(matches!(parse(&["--nope"]), Err(ArgError::UnknownFlag(_))));
    }

    #[test]
    fn rejects_no_arguments() {
        assert!(matches!(parse(&[]), Err(ArgError::MissingInput)));
    }
}
