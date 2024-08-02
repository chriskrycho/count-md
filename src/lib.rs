use bitflags::bitflags;
use pulldown_cmark::{Event, Options as CmarkOptions, Parser, Tag, TagEnd};
use unicode_segmentation::UnicodeSegmentation;
use xmlparser::{Token, Tokenizer};

/// Count some Markdown, using the default [`Options`].
pub fn count(text: &str) -> u64 {
    count_with_options(text, Options::DEFAULT)
}

/// Count some Markdown, using the supplied [`Options`].
pub fn count_with_options(text: &str, options: Options) -> u64 {
    let mut state = State {
        in_code_block: false,
        blockquote_level: 0,
        in_metadata_block: false,
        in_footnote: false,
        in_table: false,
        in_heading: false,
    };

    // Turn on everything…
    let cmark_options = CmarkOptions::all()
        // …then turn off *old* footnotes…
        & !CmarkOptions::ENABLE_OLD_FOOTNOTES
        // …and finally turn back on *new* footnotes.
        | CmarkOptions::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(text, cmark_options);

    // TODO: check whether items other than blockquotes can be nested!
    let mut count = 0;
    for event in parser {
        use Event::*;
        match event {
            Text(text) => {
                if state.allowed_for(&options) {
                    count += text.unicode_words().count() as u64;
                }
            }

            Code(text) => {
                if options.contains(Options::IncludeInlineCode) {
                    count += text.unicode_words().count() as u64;
                }
            }

            Start(tag) => match tag {
                Tag::CodeBlock(_) => state.in_code_block = true,
                Tag::BlockQuote => state.blockquote_level += 1,
                Tag::MetadataBlock(_) => state.in_metadata_block = true,
                Tag::FootnoteDefinition(_) => state.in_footnote = true,
                Tag::Table(_) => state.in_table = true,
                Tag::Heading { .. } => state.in_heading = true,
                _ => {}
            },

            End(tag) => match tag {
                TagEnd::CodeBlock => state.in_code_block = false,
                TagEnd::BlockQuote => state.blockquote_level -= 1,
                TagEnd::MetadataBlock(_) => state.in_metadata_block = false,
                TagEnd::FootnoteDefinition => state.in_footnote = false,
                TagEnd::Table => state.in_table = false,
                TagEnd::Heading(_) => state.in_heading = false,
                _ => {}
            },

            Html(html) => {
                if options.contains(Options::IncludeBlockHtml) {
                    for token in Tokenizer::from(html.as_ref()).flatten() {
                        if let Token::Text { text } = token {
                            count += text.unicode_words().count() as u64;
                        }
                    }
                }
            }

            // None of these contribute to the final count.
            InlineHtml(_tag) => {}
            FootnoteReference(_) => {}
            SoftBreak => {}
            HardBreak => {}
            Rule => {}
            TaskListMarker(_) => {}
        }
    }

    count
}

pub struct State {
    in_code_block: bool,
    blockquote_level: u8,
    in_metadata_block: bool,
    in_footnote: bool,
    in_table: bool,
    in_heading: bool,
}

impl State {
    fn allowed_for(&self, options: &Options) -> bool {
        (!self.in_code_block || options.contains(Options::IncludeBlockCode))
            && (!self.in_blockquote() || options.contains(Options::IncludeBlockquotes))
            && (!self.in_metadata_block || options.contains(Options::IncludeMetadata))
            && (!self.in_footnote || options.contains(Options::IncludeFootnotes))
            && (!self.in_table || options.contains(Options::IncludeTables))
            && (!self.in_heading || options.contains(Options::IncludeHeadings))
    }

    #[inline(always)]
    fn in_blockquote(&self) -> bool {
        self.blockquote_level > 0
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct Options: u16 {
        const IncludeInlineCode =  1;
        const IncludeBlockCode =   1 << 2;
        const IncludeTables =      1 << 3;
        const IncludeFootnotes =   1 << 4;
        const IncludeBlockHtml =   1 << 5;
        const IncludeBlockquotes = 1 << 6;
        const IncludeMetadata =    1 << 7;
        const IncludeHeadings =    1 << 8;

        const DEFAULT =
              Options::IncludeInlineCode.bits()
            | Options::IncludeTables.bits()
            | Options::IncludeFootnotes.bits()
            | Options::IncludeBlockHtml.bits()
            | Options::IncludeHeadings.bits()
            ;
    }
}

#[cfg(test)]
mod tests;
