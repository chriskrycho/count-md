use std::{
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use clap::{ArgAction, Parser};
use rayon::prelude::*;

use count_md::{count_with_options, Options};

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let (inputs, output) = args.paths.resolve()?;

    let contents = match inputs {
        Input::Stdin(mut stdin) => {
            let mut buf = String::new();
            stdin
                .read_to_string(&mut buf)
                .map_err(|source| Error::Read {
                    src: String::from("<stdin>"),
                    source,
                })?;
            vec![(String::from("<stdin>"), buf)]
        }
        Input::Files(items) => items
            .into_iter()
            .map(|(path, mut input)| -> Result<(String, String), Error> {
                let mut buf = String::new();
                input
                    .read_to_string(&mut buf)
                    .map_err(|source| Error::Read {
                        src: String::from("<stdin>"),
                        source,
                    })?;
                Ok((path.display().to_string(), buf))
            })
            .collect::<Result<Vec<_>, Error>>()?,
    };

    let resolved_options = options_from(&args);

    // This can be multithreaded, using Rayon to parallelize the counting. That
    // should make it *much* faster, since right now it is single-threaded.
    let (total, pairs) = contents
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

    report(pairs, total, output)
}

// This could in principle be async, but it would not much matter from what I
// can see: it needs to report and flush *all* of the data. (Test it, of course,
// just to be sure!)
fn report(
    pairs: Vec<(&impl std::fmt::Display, u64)>,
    total: u64,
    output: Output,
) -> Result<(), Error> {
    let (dest, mut buf) = match output {
        Output::File { path, buf } => (path.display().to_string(), buf),
        Output::Stdout(stdout) => (String::from("<stdout>"), stdout),
    };

    for (path, count) in pairs {
        writeln!(buf, "{path} has {count} words").map_err(|source| Error::Write {
            dest: dest.clone(),
            source,
        })?;
    }

    writeln!(buf, "Total: {total}").map_err(|source| Error::Write {
        dest: dest.clone(),
        source,
    })?;

    buf.flush()
        .map_err(|source| Error::Flush { dest, source })?;

    Ok(())
}

// Note: this might be able to be eliminated entirely, since there is only the
// one variant and I am otherwise just dumping strings.
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("could not open file at '{path}' {reason}")]
    CouldNotOpenFile {
        path: PathBuf,
        reason: FileOpenReason,
        source: std::io::Error,
    },

    #[error("`--force` is only allowed with `--output`")]
    InvalidArgs,

    #[error("invalid file path with no parent directory: '{path}'")]
    InvalidDirectory { path: PathBuf },

    #[error("could not create directory '{dir}' to write file '{path}")]
    CreateDirectory {
        dir: PathBuf,
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    CheckFileExists { source: std::io::Error },

    #[error("the file '{0}' already exists")]
    FileExists(PathBuf),

    #[error("could not write to '{dest}': {source}")]
    Write {
        dest: String,
        source: std::io::Error,
    },

    #[error("could not flush to '{dest}': {source}")]
    Flush {
        dest: String,
        source: std::io::Error,
    },

    #[error("could not read from '{src}': {source}")]
    Read { src: String, source: io::Error },
}

#[derive(Debug)]
enum FileOpenReason {
    Read,
    Write,
}

impl std::fmt::Display for FileOpenReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOpenReason::Read => write!(f, "to read it"),
            FileOpenReason::Write => write!(f, "to write to it"),
        }
    }
}

fn options_from(args: &Args) -> Options {
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
    #[clap(flatten)]
    paths: Paths,

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

#[derive(clap::Args, Debug, PartialEq, Clone)]
struct Paths {
    /// Files to count text in. Will use `stdin` if none are supplied.
    files: Vec<PathBuf>,

    /// Where to print the output. Will use `stdout` if not supplied.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// If the supplied `output` file is present, overwrite it.
    #[arg(long, default_missing_value("true"), num_args(0..=1), require_equals(true))]
    force: Option<bool>,
}

impl Paths {
    fn resolve(&self) -> Result<(Input, Output), Error> {
        let dest_cfg = match (&self.output, self.force.unwrap_or(false)) {
            (Some(buf), force) => DestCfg::Path { buf, force },
            (None, false) => DestCfg::Stdout,
            (None, true) => return Err(Error::InvalidArgs)?,
        };
        let inputs = if self.files.is_empty() {
            Input::Stdin(Box::new(BufReader::new(io::stdin())) as Box<dyn Read>)
        } else {
            to_input_buffers(&self.files)?
        };
        let output = output_buffer(&dest_cfg)?;
        Ok((inputs, output))
    }
}

enum Output {
    File { path: PathBuf, buf: Box<dyn Write> },
    Stdout(Box<dyn Write>),
}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Output::File { path, .. } => write!(f, "{path:?}"),
            Output::Stdout(..) => f.write_str("stdin"),
        }
    }
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Output::File { path, .. } => write!(f, "{}", path.display()),
            Output::Stdout(..) => f.write_str("stdin"),
        }
    }
}

pub(crate) enum DestCfg<'p> {
    Path { buf: &'p Path, force: bool },
    Stdout,
}

enum Input {
    Files(Vec<(PathBuf, Box<dyn Read>)>),
    Stdin(Box<dyn Read>),
}

fn to_input_buffers(paths: &[PathBuf]) -> Result<Input, Error> {
    paths
        .iter()
        .map(|path| {
            std::fs::File::open(path)
                .map_err(|source| Error::CouldNotOpenFile {
                    path: path.to_owned(),
                    reason: FileOpenReason::Read,
                    source,
                })
                .map(|file| {
                    (
                        path.to_owned(),
                        Box::new(BufReader::new(file)) as Box<dyn Read>,
                    )
                })
        })
        .collect::<Result<Vec<_>, Error>>()
        .map(|inputs| Input::Files(inputs))
}

fn output_buffer(dest_cfg: &DestCfg) -> Result<Output, Error> {
    match *dest_cfg {
        DestCfg::Stdout => Ok(Output::Stdout(Box::new(std::io::stdout()))),

        DestCfg::Path { buf: path, force } => {
            let dir = path.parent().ok_or_else(|| Error::InvalidDirectory {
                path: path.to_owned(),
            })?;

            std::fs::create_dir_all(dir).map_err(|source| Error::CreateDirectory {
                dir: dir.to_owned(),
                path: path.to_owned(),
                source,
            })?;

            // TODO: can I, without doing a TOCTOU, avoid overwriting an existing
            // file? (That's mostly academic, but since the point of this is to
            // learn, I want to learn that.)
            let file_exists = path
                .try_exists()
                .map_err(|source| Error::CheckFileExists { source })?;

            if file_exists && !force {
                return Err(Error::FileExists(path.to_owned()));
            }

            let file = std::fs::File::create(path).map_err(|source| Error::CouldNotOpenFile {
                path: path.to_owned(),
                reason: FileOpenReason::Write,
                source,
            })?;

            Ok(Output::File {
                path: path.to_owned(),
                buf: Box::new(file),
            })
        }
    }
}
