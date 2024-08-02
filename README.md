# count-md

A simple, configurable command-line tool and Rust library for Unicode-aware, Markdown-aware, HTML-aware word counting in Markdown documents.

That is: this tool will correctly count words in a Unicode-aware way, *without* incorrectly including Markdown syntax or HTML tags. It can include or exclude content like blockquotes, footnotes, code blocks, and so on, and ships with reasonable defaults out of the box for each!

## Install

To get the command line tool:

```sh
cargo install count-md
```

To use the library:

- On the CLI:

    ```sh
    cargo add count-md
    ```

- In Cargo.toml:

    ```toml
    [dependencies]
    count-md = "0.1"
    ```

## Example

You might have a file with content like this:

```markdown
# Title

This is some text!

> Here is a quote from someone else.

Here is more text.
```

If you wanted to know the number of non-quoted words, including the title but not including the blockquote, you would simply run `count-md <path to the file>`, and it will helpfully report that there are 9 words total. By contrast, `wc -w` will report that there are *18* words: it includes the blockquote, of course, but it also includes the `#` for the title and the `>` for the blockquote, neither of which is desirable!

## Status

Support for including or or excluding the following Markdown features:

- [x] Headings
- [x] Blockquotes
    - [x] Nested blockquotes
    - [ ] Admonitions[^admonitions]
- [x] Code blocks
- [x] Inline code
- [ ] Block HTML 🚧 **Partial**
- [x] Footnotes
- [x] Tables
- [ ] Math

[^admonitions]: Admonitions are not blockquotes, but they are listed here because that is how they work *syntactically*.

## Library

The core functionality here can be used as a Rust library.[^c] There are two main entry points:

- `count`: accepts a `&str` and counts it with the default set of options, equivalent to running `count-md` with zero options on the command line.

- `count_with_options`: accepts a `&str` and an `Options` value (a bitmask), which allows you to configure each option directly. For the equivalent to running `count-md` with some option, use `Options::DEFAULT` and combine it with other flags:

    - With bitmasking directly: `Options::DEFAULT | Options::IncludeBlockquotes`
    - With the methods supplied by the `bitflags` library, `insert` and `remove`:

        ```rust
        let mut options = Options::DEFAULT;
        options.insert(Options::IncludeBlockquotes);
        options.remove(Options::IncludeHeadings);
        ```

See the documentation for more!

[^c]: In the future, I may also supply C bindings, but those need quite a bit of vetting before I am comfortable doing that!
