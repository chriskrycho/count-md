use super::*;

#[test]
fn includes_basic_text() {
    let result = count("Hello, world!");
    assert_eq!(result, 2);
}

#[test]
fn includes_heading_1() {
    let result = count("# Heading 1");
    assert_eq!(result, 2);
}

#[test]
fn includes_heading_2() {
    let result = count("## Heading 2 – More Heading");
    assert_eq!(result, 4);
}

#[test]
fn includes_inline_code() {
    let result = count("This is `code`.");
    assert_eq!(result, 3);
}

#[test]
fn excludes_code_blocks() {
    let result = count(
        "This is code:

```rust
fn main() {}
```",
    );
    assert_eq!(result, 3);
}

#[test]
fn excludes_inline_html_tags() {
    let result = count("This is how we say <em>Hello, world</em>");
    assert_eq!(result, 7);
}

#[test]
fn excludes_block_html_tags() {
    let result = count(
        "<h1>Heading</h1>

<h2>Heading 2</h2>

<ul>
  <li>List item</li>
  <li>List item</li>
  <li>List item</li>
</ul>",
    );
    assert_eq!(result, 9);
}

#[test]
fn excludes_footnote_references() {
    let result = count(
        "This is a paragraph.[^fn]

[^fn]: And a definition!",
    );
    assert_eq!(result, 7);
}

#[test]
fn excludes_rules() {
    let result = count("Hello…\n\n---\n\n…world!");
    assert_eq!(result, 2);
}

#[test]
fn excludes_metadata() {
    let result = count("---\ntitle: What a Cool Thing\n---\n\nNeato!");
    assert_eq!(result, 1);
}

#[test]
fn excludes_blockquotes() {
    let result = count("Not a quote\n\n> This is a blockquote\n\nNot again");
    assert_eq!(result, 5);
}

#[test]
fn includes_tables() {
    let result = count(
        r#"Some text

| thead 1 | thead 2 |
| ------- | ------- |
| foo     | bar     |
| baz     | quux    |

More stuff"#,
    );

    assert_eq!(result, 12);
}

#[test]
fn includes_footnotes() {
    let result = count(
        r#"This is some text.[^fn-def]

[^fn-def]: It defines a footnote.

Yay!"#,
    );

    assert_eq!(result, 9);
}

#[test]
fn handles_footnotes_with_continued_content() {
    let result = count(
        r#"This is some text.[^fn-def]

[^fn-def]: It defines a footnote.

    This paragraph continues the footnote!"#,
    );

    assert_eq!(result, 13);
}

mod options {
    use super::*;

    mod inline_code {
        use super::*;

        #[test]
        fn enabled() {
            let result = count_with_options("This is `code`.", Options::IncludeInlineCode);
            assert_eq!(result, 3);
        }

        #[test]
        fn disabled() {
            let result = count_with_options("This is `code`.", Options::empty());
            assert_eq!(result, 2);
        }
    }

    mod block_code {
        use super::*;

        #[test]
        fn enabled() {
            let result = count_with_options(
                "This is code:

```rust
fn main() { }
```
",
                Options::IncludeBlockCode,
            );
            assert_eq!(result, 5);
        }

        #[test]
        fn disabled() {
            let result = count_with_options(
                "This is code:

    ```rust
    fn main() { }
    ```
    ",
                Options::empty(),
            );
            assert_eq!(result, 3);
        }
    }

    mod tables {
        use super::*;

        #[test]
        fn enabled() {
            let result = count_with_options(
                r#"Some text

| thead 1 | thead 2 |
| ------- | ------- |
| foo     | bar     |
| baz     | quux    |

More stuff"#,
                Options::IncludeTables,
            );

            assert_eq!(result, 12);
        }

        #[test]
        fn disabled() {
            let result = count_with_options(
                r#"Some text

| thead 1 | thead 2 |
| ------- | ------- |
| foo     | bar     |
| baz     | quux    |

More stuff"#,
                Options::empty(),
            );

            assert_eq!(result, 4);
        }
    }

    mod footnotes {
        use super::*;

        #[test]
        fn enabled() {
            let result = count_with_options(
                "Some text.[^fn]\n\n[^fn]: Footnote definition text!",
                Options::IncludeFootnotes,
            );
            assert_eq!(result, 5);
        }

        #[test]
        fn disabled() {
            let result = count_with_options(
                "Some text.[^fn]\n\n[^fn]: Footnote definition text!",
                Options::empty(),
            );
            assert_eq!(result, 2);
        }
    }

    mod block_html {
        use super::*;

        #[test]
        fn enabled() {
            let result = count_with_options(
                "Some text.\n\n<div>Block HTML content.\n\nWith newlines!</div>\n\nMore text.",
                Options::IncludeBlockHtml,
            );
            assert_eq!(result, 9);
        }

        #[test]
        fn disabled() {
            let result = count_with_options(
                "Some text.\n\n<div>Block HTML content.\n\nWith newlines!</div>\n\nMore text.",
                Options::empty(),
            );
            assert_eq!(result, 4);
        }
    }

    mod blockquotes {
        use super::*;

        #[test]
        fn enabled() {
            let result = count_with_options(
                "Text.\n\n> Blockquote text.\n\nMore text.",
                Options::IncludeBlockquotes,
            );
            assert_eq!(result, 5);
        }

        #[test]
        fn disabled() {
            let result = count_with_options(
                "Text.\n\n> Blockquote text.\n\nMore text.",
                Options::empty(),
            );
            assert_eq!(result, 3);
        }
    }

    mod metadata {
        use super::*;

        #[test]
        fn enabled() {
            let result = count_with_options(
                "---\ntitle: What a Cool Thing\n---\n\nNeato!",
                Options::IncludeMetadata,
            );
            assert_eq!(result, 6);
        }

        #[test]
        fn disabled() {
            let result = count_with_options(
                "---\ntitle: What a Cool Thing\n---\n\nNeato!",
                Options::empty(),
            );
            assert_eq!(result, 1);
        }
    }

    mod headings {
        use super::*;

        #[test]
        fn enabled() {
            let result = count_with_options(
                "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6\n\nwhoa",
                Options::IncludeHeadings,
            );
            assert_eq!(result, 7);
        }

        #[test]
        fn disabled() {
            let result = count_with_options(
                "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6\n\nwhoa",
                Options::empty(),
            );
            assert_eq!(result, 1);
        }
    }

    mod all {
        use super::*;

        #[test]
        fn includes_basic_text() {
            let result = count_with_options("Hello, world!", Options::all());
            assert_eq!(result, 2);
        }

        #[test]
        fn includes_heading_1() {
            let result = count_with_options("# Heading 1", Options::all());
            assert_eq!(result, 2);
        }

        #[test]
        fn includes_heading_2() {
            let result = count_with_options("## Heading 2 – More Heading", Options::all());
            assert_eq!(result, 4);
        }

        #[test]
        fn includes_inline_code() {
            let result = count_with_options("This is `code`.", Options::all());
            assert_eq!(result, 3);
        }

        #[test]
        fn includes_code_blocks() {
            let result = count_with_options(
                "```rust
        fn main() {}
        ```",
                Options::all(),
            );
            assert_eq!(result, 2);
        }

        #[test]
        fn excludes_inline_html_tags() {
            let result =
                count_with_options("This is how we say <em>Hello, world</em>", Options::all());
            assert_eq!(result, 7);
        }

        #[test]
        fn excludes_block_html_tags() {
            let result = count_with_options(
                "<h1>Heading</h1>

<h2>Heading 2</h2>

<ul>
    <li>List item</li>
    <li>List item</li>
    <li>List item</li>
</ul>",
                Options::all(),
            );
            assert_eq!(result, 9);
        }

        #[test]
        fn excludes_footnote_references() {
            let result = count_with_options(
                "This is a paragraph.[^fn]

[^fn]: And a definition!",
                Options::all(),
            );
            assert_eq!(result, 7);
        }

        #[test]
        fn excludes_rules() {
            let result = count_with_options("Hello…\n\n---\n\n…world!", Options::all());
            assert_eq!(result, 2);
        }

        #[test]
        fn includes_metadata() {
            let result = count_with_options(
                "---\ntitle: What a Cool Thing\n---\n\nNeato!",
                Options::all(),
            );
            assert_eq!(result, 6);
        }

        #[test]
        fn excludes_blockquotes() {
            let result = count_with_options(
                "Not a quote\n\n> This is a blockquote\n\nNot again",
                Options::all(),
            );
            assert_eq!(result, 9);
        }

        #[test]
        fn includes_tables() {
            let result = count_with_options(
                r#"Some text

| thead 1 | thead 2 |
| ------- | ------- |
| foo     | bar     |
| baz     | quux    |

More stuff"#,
                Options::all(),
            );

            assert_eq!(result, 12);
        }

        #[test]
        fn includes_footnotes() {
            let result = count_with_options(
                r#"This is some text.[^fn-def]

[^fn-def]: It defines a footnote.

Yay!"#,
                Options::all(),
            );

            assert_eq!(result, 9);
        }

        #[test]
        fn includes_headers() {
            let result = count_with_options(
                "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6\n\nwhoa",
                Options::all(),
            );
            assert_eq!(result, 7);
        }
    }
}
