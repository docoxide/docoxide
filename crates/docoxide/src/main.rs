use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

/// Convert HTML (with optional CSS) to PDF.
#[derive(Debug, Parser)]
#[command(name = "docoxide", version, about)]
struct Cli {
    /// Path to the input HTML file. Reads from stdin if omitted or set to `-`.
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Optional path to a CSS file.
    #[arg(short, long)]
    css: Option<PathBuf>,

    /// Path to the output PDF file.
    #[arg(short, long)]
    output: PathBuf,
}

fn read_input(path: Option<&Path>) -> io::Result<String> {
    match path {
        None => read_stdin(),
        Some(p) if p == Path::new("-") => read_stdin(),
        Some(p) => fs::read_to_string(p),
    }
}

fn read_stdin() -> io::Result<String> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    Ok(s)
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let html = match read_input(cli.input.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to read input: {e}");
            return ExitCode::FAILURE;
        }
    };

    let css = match cli.css.as_ref().map(fs::read_to_string).transpose() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to read css: {e}");
            return ExitCode::FAILURE;
        }
    };

    let pdf = docoxide::convert(&html, css.as_deref());

    if let Err(e) = fs::write(&cli.output, pdf) {
        eprintln!("failed to write {}: {e}", cli.output.display());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
