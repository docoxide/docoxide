use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Fast, browser-free HTML to PDF converter.
#[derive(Debug, Parser)]
#[command(name = "docoxide", version, about)]
struct Cli {
    /// HTML input: file path, URL (http/https/file), or - for stdin.
    /// Reads from stdin if omitted.
    #[arg(short, long)]
    input: Option<String>,

    /// Additional CSS stylesheet file. Can be repeated.
    #[arg(short = 's', long = "stylesheet")]
    stylesheets: Vec<PathBuf>,

    /// Base URL for resolving relative links and images.
    #[arg(short, long)]
    base_url: Option<String>,

    /// PDF metadata as key=value. Can be repeated.
    /// Keys: title, author, subject, keywords, creation_date.
    /// Example: --metadata title="My Doc" --metadata author="Jane"
    #[arg(short, long = "metadata")]
    metadata: Vec<String>,

    /// Custom font file. Can be repeated.
    #[arg(long = "font")]
    fonts: Vec<PathBuf>,

    /// Output PDF file. Writes to stdout if omitted.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Suppress progress output.
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("docoxide: error: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(cli: Cli) -> docoxide::Result<()> {
    use docoxide::{Config, Html};

    let metadata = parse_metadata(&cli.metadata).map_err(docoxide::Error::PdfGeneration)?;

    let mut config = Config::new().with_metadata(metadata);

    for font_path in &cli.fonts {
        config = config.with_font(font_path.as_path());
    }

    let mut html = match &cli.input {
        Some(input) if input == "-" => Html::new(read_stdin()?),
        Some(input) if looks_like_url(input) => {
            let url: url::Url = input
                .parse()
                .map_err(|e| docoxide::Error::Network(format!("invalid URL '{input}': {e}")))?;
            Html::new(url)
        }
        Some(input) => {
            let abs = std::fs::canonicalize(input)?;
            let url = url::Url::from_file_path(&abs)
                .map_err(|_| docoxide::Error::Network(format!("could not convert path to URL: {input}")))?;
            Html::new(url)
        }
        None => Html::new(read_stdin()?),
    };

    for css_path in &cli.stylesheets {
        html = html.with_stylesheet(css_path.as_path());
    }

    if let Some(ref base_url) = cli.base_url {
        match base_url.parse::<url::Url>() {
            Ok(url) => html = html.with_base_url(url),
            Err(e) => eprintln!("docoxide: warning: invalid base URL '{base_url}': {e}"),
        }
    }

    html = html.with_config(&config);

    let pdf = html.write_pdf()?;

    match cli.output {
        Some(ref path) => {
            pdf.write_pdf(path)?;
            if !cli.quiet {
                eprintln!("docoxide: written {} page(s) to {}", pdf.page_count(), path.display());
            }
        }
        None => {
            let mut out = io::stdout().lock();
            out.write_all(pdf.as_bytes())?;
            out.flush()?;
        }
    }

    Ok(())
}

fn parse_metadata(entries: &[String]) -> std::result::Result<docoxide::Metadata, String> {
    let mut meta = docoxide::Metadata::default();
    for entry in entries {
        let (key, value) = entry
            .split_once('=')
            .ok_or_else(|| format!("metadata must be key=value, got '{entry}'"))?;
        match key.trim() {
            "title" => meta.title = Some(value.to_owned()),
            "author" => meta.author = Some(value.to_owned()),
            "subject" => meta.subject = Some(value.to_owned()),
            "keywords" => meta.keywords.push(value.to_owned()),
            "creation_date" => meta.creation_date = Some(value.to_owned()),
            other => return Err(format!("unknown metadata key '{other}'")),
        }
    }
    Ok(meta)
}

fn read_stdin() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn looks_like_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("file://")
}
