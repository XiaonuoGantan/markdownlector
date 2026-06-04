//! The single point of contact with the `pulldown-cmark` dependency.
//!
//! Every consumer receives the parser's event stream and never constructs the
//! parser itself, so swapping the Markdown parser later is a one-file change.
//! The plain renderer consumes these events today; the layout pipeline will
//! become a second consumer in a later milestone.
//!
//! The re-export below isolates the crate *name*, but consumers still match on
//! pulldown-cmark's event *shapes*. A future parser swap therefore means
//! adapting the replacement to this event vocabulary here, inside this file.

pub use pulldown_cmark::{Event, Tag, TagEnd};
use pulldown_cmark::{Options, Parser};

/// Parse Markdown `source` into the pull-parser event stream.
pub fn parse(source: &str) -> Parser<'_> {
    // The lightweight GitHub-flavored extensions a README commonly uses. Tables
    // are deliberately left to the layout renderer in a later milestone.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    Parser::new_ext(source, options)
}
