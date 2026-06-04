//! Load input from a file or standard input and validate the input boundary.
//!
//! This is the system boundary: raw bytes become text here, or fail with a
//! readable error. Nothing downstream ever sees invalid UTF-8, binary data, or
//! a raw I/O error.

use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// Where Markdown is read from.
pub enum InputSource {
    File(PathBuf),
    Stdin,
}

/// A failure while loading input. `path` is `None` for standard input.
#[derive(Debug)]
pub enum InputError {
    Read {
        path: Option<PathBuf>,
        source: io::Error,
    },
    Binary {
        path: Option<PathBuf>,
    },
    NotUtf8 {
        path: Option<PathBuf>,
    },
}

/// Read `source` fully and return its text, or a readable error.
pub fn load(source: &InputSource) -> Result<String, InputError> {
    match source {
        InputSource::File(path) => {
            let bytes = fs::read(path).map_err(|source| InputError::Read {
                path: Some(path.clone()),
                source,
            })?;
            decode(bytes, Some(path.clone()))
        }
        InputSource::Stdin => {
            let mut bytes = Vec::new();
            io::stdin()
                .read_to_end(&mut bytes)
                .map_err(|source| InputError::Read { path: None, source })?;
            decode(bytes, None)
        }
    }
}

/// Validate raw bytes as text.
///
/// A NUL byte means we treat the input as binary and refuse it, rather than
/// printing control bytes to the terminal. NUL is itself valid UTF-8
/// (`U+0000`), so this check must run *before* UTF-8 validation to catch
/// otherwise-valid text that contains a NUL. The whole buffer is scanned: a
/// NUL anywhere is binary.
fn decode(bytes: Vec<u8>, path: Option<PathBuf>) -> Result<String, InputError> {
    if bytes.contains(&0) {
        return Err(InputError::Binary { path });
    }
    let mut text = String::from_utf8(bytes).map_err(|_| InputError::NotUtf8 { path })?;
    // Strip a leading UTF-8 byte-order mark. Left in place it prints as mojibake
    // and, worse, glues to a leading `#` so the parser no longer sees a heading.
    if text.starts_with('\u{feff}') {
        text.remove(0);
    }
    Ok(text)
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::Read { path, source } => match path {
                Some(path) => write!(f, "cannot read {}: {source}", path.display()),
                None => write!(f, "cannot read standard input: {source}"),
            },
            InputError::Binary { path } => match path {
                Some(path) => write!(
                    f,
                    "{}: input looks binary (contains a NUL byte); refusing to print it",
                    path.display()
                ),
                None => write!(
                    f,
                    "standard input looks binary (contains a NUL byte); refusing to print it"
                ),
            },
            InputError::NotUtf8 { path } => match path {
                Some(path) => write!(f, "{}: input is not valid UTF-8", path.display()),
                None => write!(f, "standard input is not valid UTF-8"),
            },
        }
    }
}

impl std::error::Error for InputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InputError::Read { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_utf8_text() {
        assert_eq!(decode(b"hello".to_vec(), None).unwrap(), "hello");
    }

    #[test]
    fn empty_input_is_valid_and_empty() {
        assert_eq!(decode(Vec::new(), None).unwrap(), "");
    }

    #[test]
    fn accepts_multibyte_utf8_without_false_rejecting() {
        // Guard against a sloppy "high byte = binary" heuristic rejecting
        // legitimate non-ASCII Markdown.
        let text = "日本語 café 🎉";
        assert_eq!(decode(text.as_bytes().to_vec(), None).unwrap(), text);
    }

    #[test]
    fn strips_leading_bom() {
        assert_eq!(
            decode("\u{feff}hello".as_bytes().to_vec(), None).unwrap(),
            "hello"
        );
    }

    #[test]
    fn rejects_invalid_utf8_without_nul() {
        assert!(matches!(
            decode(vec![0xFF, 0xFE], None),
            Err(InputError::NotUtf8 { .. })
        ));
        assert!(matches!(
            decode(vec![0x80], None),
            Err(InputError::NotUtf8 { .. })
        ));
    }

    #[test]
    fn rejects_nul_at_first_byte() {
        assert!(matches!(
            decode(vec![0x00], None),
            Err(InputError::Binary { .. })
        ));
    }

    #[test]
    fn rejects_nul_at_last_byte() {
        // A first-chunk-only scan would miss this; we scan the whole buffer.
        assert!(matches!(
            decode(b"ok\x00".to_vec(), None),
            Err(InputError::Binary { .. })
        ));
    }

    #[test]
    fn rejects_nul_even_in_valid_utf8() {
        // Contract: a NUL byte is always binary, even though U+0000 is valid
        // UTF-8. The binary check therefore precedes UTF-8 validation.
        assert!(matches!(
            decode(b"a\x00b".to_vec(), None),
            Err(InputError::Binary { .. })
        ));
    }

    #[test]
    fn load_reads_a_file() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/basic.md");
        let text = load(&InputSource::File(PathBuf::from(path))).unwrap();
        assert!(text.contains("Markdownlector"));
    }

    #[test]
    fn load_reports_missing_file() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/does-not-exist.md"
        );
        assert!(matches!(
            load(&InputSource::File(PathBuf::from(path))),
            Err(InputError::Read { .. })
        ));
    }
}
