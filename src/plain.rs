//! Render Markdown to plain text.
//!
//! A deliberately conservative `events -> text` pass: it strips inline markup,
//! keeps list markers and indentation, drops link targets, and emits exactly
//! one trailing newline. This is a dead-end sibling of the future layout
//! pipeline, not a foundation for it.

use crate::markdown::{self, Event, Tag, TagEnd};

/// Render Markdown `source` to plain text with a single trailing newline.
/// Empty input renders to an empty string.
pub fn render(source: &str) -> String {
    let mut renderer = Renderer::default();
    for event in markdown::parse(source) {
        renderer.handle(event);
    }
    renderer.finish()
}

#[derive(Default)]
struct Renderer {
    out: String,
    /// One entry per open list. `Some(n)` is an ordered list whose next item is
    /// numbered `n`; `None` is a bullet list. The length is the nesting depth.
    lists: Vec<Option<u64>>,
}

impl Renderer {
    fn handle(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.out.push_str(&text),
            Event::Code(code) => self.out.push_str(&code),
            Event::SoftBreak => self.out.push(' '),
            Event::HardBreak => self.out.push('\n'),
            Event::Html(html) | Event::InlineHtml(html) => self.out.push_str(&html),
            Event::TaskListMarker(done) => {
                self.out.push_str(if done { "[x] " } else { "[ ] " });
            }
            Event::Rule => self.ensure_blank_line(),
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { .. } | Tag::CodeBlock(_) => self.ensure_blank_line(),
            Tag::Paragraph => {
                if self.lists.is_empty() {
                    self.ensure_blank_line();
                } else if self.out.ends_with('\n') {
                    // A later paragraph in the same list item: indent it under
                    // the marker so it does not collapse flush-left. The first
                    // paragraph flows right after the marker, which does not end
                    // in a newline, so it is left alone.
                    self.out.push_str(&"  ".repeat(self.lists.len()));
                }
            }
            Tag::List(first_number) => {
                if self.lists.is_empty() {
                    self.ensure_blank_line();
                } else {
                    self.ensure_newline();
                }
                self.lists.push(first_number);
            }
            Tag::Item => self.start_item(),
            // Emphasis, Strong, Strikethrough, Link, Image, BlockQuote: the
            // inline markers and link targets are dropped; inner text remains.
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) | TagEnd::Paragraph | TagEnd::CodeBlock | TagEnd::Item => {
                self.ensure_newline();
            }
            TagEnd::List(_) => {
                self.lists.pop();
            }
            _ => {}
        }
    }

    fn start_item(&mut self) {
        self.ensure_newline();
        let indent = "  ".repeat(self.lists.len().saturating_sub(1));
        let marker = match self.lists.last_mut() {
            Some(Some(number)) => {
                let marker = format!("{number}. ");
                *number += 1;
                marker
            }
            _ => "- ".to_string(),
        };
        self.out.push_str(&indent);
        self.out.push_str(&marker);
    }

    /// Ensure the output ends with a newline (no-op at the start of output).
    fn ensure_newline(&mut self) {
        if !self.out.is_empty() && !self.out.ends_with('\n') {
            self.out.push('\n');
        }
    }

    /// Ensure blocks are separated by exactly one blank line (no-op at start).
    fn ensure_blank_line(&mut self) {
        if self.out.is_empty() {
            return;
        }
        while self.out.ends_with('\n') {
            self.out.pop();
        }
        self.out.push_str("\n\n");
    }

    fn finish(mut self) -> String {
        while self.out.ends_with('\n') {
            self.out.pop();
        }
        if !self.out.is_empty() {
            self.out.push('\n');
        }
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn strips_heading_markers_keeps_text() {
        assert_eq!(render("# Title"), "Title\n");
        assert_eq!(render("### Deep heading"), "Deep heading\n");
    }

    #[test]
    fn preserves_paragraph_text() {
        assert_eq!(render("Hello world."), "Hello world.\n");
    }

    #[test]
    fn soft_break_is_space_hard_break_is_newline() {
        assert_eq!(render("a\nb"), "a b\n");
        assert_eq!(render("a  \nb"), "a\nb\n");
    }

    #[test]
    fn strips_inline_emphasis_and_code_markers() {
        assert_eq!(render("**bold** and _italic_"), "bold and italic\n");
        assert_eq!(render("use `code` here"), "use code here\n");
    }

    #[test]
    fn drops_link_target_keeps_text() {
        assert_eq!(
            render("see [the docs](https://example.com)"),
            "see the docs\n"
        );
    }

    #[test]
    fn preserves_fenced_code_and_keeps_markers_inside() {
        assert_eq!(render("```\n# not a heading\n```"), "# not a heading\n");
    }

    #[test]
    fn keeps_bullet_markers_and_nested_indent() {
        assert_eq!(
            render("- apple\n  - gala\n- pear"),
            "- apple\n  - gala\n- pear\n"
        );
    }

    #[test]
    fn indents_continuation_paragraphs_in_list_items() {
        assert_eq!(
            render("- para one\n\n  para two\n- next"),
            "- para one\n  para two\n- next\n"
        );
    }

    #[test]
    fn numbers_ordered_lists() {
        assert_eq!(render("1. one\n2. two"), "1. one\n2. two\n");
    }

    #[test]
    fn empty_input_renders_empty() {
        assert_eq!(render(""), "");
    }

    #[test]
    fn separates_blocks_with_one_blank_line() {
        assert_eq!(render("a\n\nb"), "a\n\nb\n");
    }
}
