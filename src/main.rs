use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::{ArgAction, Parser};
use rayon::prelude::*;

use count_md::{count_with_options, Options};

fn main() -> Result<(), String> {
    let args = Args::parse();

    // This can probably be a stream/done asynchronously? Not clear whether that
    // will improve or degrade performance, but may be worth experimenting with.
    let file_contents = load(&args.files).map_err(|errs| {
        // Might reach for something for nicer error reporting here?
        errs.iter()
            .map(|e| format!("{e}"))
            .collect::<Vec<String>>()
            .join("\n")
    })?;

    let resolved_options = options(&args);

    // This can be multithreaded, using Rayon to parallelize the counting. That
    // should make it *much* faster, since right now it is single-threaded.
    let (total, pairs) = file_contents
        .par_iter()
        .fold(
            || (0, vec![]),
            |(sum, mut pairs), (path, content)| {
                let count = count_with_options(content, resolved_options);
                let new_sum = sum + count;
                pairs.push((path, count));
                (new_sum, pairs)
            },
        )
        .reduce(
            || (0, vec![]),
            |(total, mut pairs), (subtotal, subpairs)| {
                // This copy should be quite cheap: it copies a reference and a
                // `u64` from `subpairs` into `pairs`. It will be O(N) on the
                // size of the `subpairs`.
                //
                // With enough elements, that could be noticeable. That is the
                // tradeoff for parallelizing this! However, in most cases, the
                // number of files in question will be relatively small; even
                // with *thousands* of files, this should be very fast.
                pairs.extend(&subpairs);
                (total + subtotal, pairs)
            },
        );

    report(pairs, total)?;

    Ok(())
}

// This could in principle be async. Given nothing *else* in the program will be
// doing work concurrently, I do not see why it would particularly help, but it
// is worth testing.
fn load(files: &[impl AsRef<Path>]) -> Result<Vec<(PathBuf, String)>, Vec<Error>> {
    let (oks, errs) = files
        .iter()
        .map(|path| {
            let path = path.as_ref();
            std::fs::read_to_string(path)
                .map(|content| (path.to_owned(), content))
                .map_err(|err| Error::Io {
                    path: path.to_owned(),
                    source: err,
                })
        })
        .fold((vec![], vec![]), |(mut oks, mut errs), item| {
            match item {
                Ok(val) => oks.push(val),
                Err(err) => errs.push(err),
            }
            (oks, errs)
        });

    if errs.is_empty() {
        Ok(oks)
    } else {
        Err(errs)
    }
}

// This could in principle be async, but it would not much matter from what I
// can see: it needs to report and flush *all* of the data. (Test it, of course,
// just to be sure!)
fn report(pairs: Vec<(&PathBuf, u64)>, total: u64) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    for (path, count) in pairs {
        writeln!(stdout, "{} has {count} words", path.display())
            .map_err(|e| format!("Error writing to stdout: {e}"))?;
    }

    writeln!(stdout, "Total: {total}").map_err(|e| format!("Error writing to stdout: {e}"))?;

    stdout
        .flush()
        .map_err(|e| format!("Error flushing stdout: {e}"))?;

    Ok(())
}

// Note: this might be able to be eliminated entirely, since there is only the
// one variant and I am otherwise just dumping strings.
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Could not read '{path}': {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

fn options(args: &Args) -> Options {
    if args.all {
        return Options::all();
    }

    let mut options = Options::empty();

    if args.metadata {
        options |= Options::IncludeMetadata;
    }

    if args.blockquotes {
        options |= Options::IncludeBlockquotes;
    }

    if args.headings {
        options |= Options::IncludeHeadings;
    }

    if args.footnotes {
        options |= Options::IncludeFootnotes;
    }

    if args.tables {
        options |= Options::IncludeTables;
    }

    if args.inline_code {
        options |= Options::IncludeInlineCode;
    }

    if args.block_code {
        options |= Options::IncludeBlockCode;
    }

    if args.block_html {
        options |= Options::IncludeBlockHtml;
    }

    options
}

#[derive(Parser)]
struct Args {
    /// The path to the file to count text in.
    files: Vec<PathBuf>,

    /// Include every possible option.
    #[clap(
        long,
        conflicts_with_all = [
            "metadata",
            "blockquotes",
            "headings",
            "footnotes",
            "tables",
            "inline_code",
            "block_code",
            "block_html"
        ]
    )]
    all: bool,

    /// Include YAML or TOML metadata.
    #[clap(
        short,
        long,
        default_value = "false",
        default_missing_value = "true",
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set
    )]
    metadata: bool,

    /// Include blockquotes.
    #[clap(
        short,
        long,
        default_value = "false",
        default_missing_value = "true",
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set
    )]
    blockquotes: bool,

    /// Include headings.
    #[clap(
        short,
        long,
        default_value = "true",
        default_missing_value = "true",
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set
    )]
    headings: bool,

    /// Include footnotes.
    #[clap(
        short,
        long,
        default_value = "true",
        default_missing_value = "true",
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set
    )]
    footnotes: bool,

    /// Include tables.
    #[clap(
        short,
        long,
        default_value = "true",
        default_missing_value = "true",
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set
    )]
    tables: bool,

    /// Include inline code.
    #[clap(
        long,
        default_value = "true",
        default_missing_value = "true",
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set
    )]
    inline_code: bool,

    /// Include block code.
    #[clap(
        long,
        default_value = "false",
        default_missing_value = "true",
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set
    )]
    block_code: bool,

    /// Include block HTML.
    #[clap(
        long,
        default_value = "true",
        default_missing_value = "true",
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set
    )]
    block_html: bool,
}
