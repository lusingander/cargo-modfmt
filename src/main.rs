mod batch;
mod config;
mod file;
mod format;
mod organize;
mod settings;
mod targets;

use std::{
    error::Error,
    io::{self, Read, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::{Parser, Subcommand};

use crate::{
    batch::{Input, Mode},
    config::{Config, parse_exclude},
    format::{EditionOverrides, FormatResolver},
    organize::{FormatOptions, SourceEdition, StyleEdition, organize},
    settings::{Origin, Resolved},
};

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum CargoCommand {
    #[command(name = "modfmt")]
    Modfmt(Args),
}

#[derive(clap::Args)]
#[command(about = "Sort external mod and supported use declarations in Rust source")]
struct Args {
    /// Rust edition used to parse the input (2015, 2018, 2021, or 2024).
    #[arg(long, global = true, value_name = "EDITION")]
    edition: Option<SourceEdition>,

    /// Rust style edition used to sort imports (defaults to the parsing edition).
    #[arg(long, global = true, value_name = "EDITION")]
    style_edition: Option<StyleEdition>,

    /// Use this configuration file instead of searching from the working directory.
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Report files needing changes (exit code 3), without modifying them.
    Check(FileArgs),
    /// Update files in place, reporting successfully updated paths to stderr.
    Write(FileArgs),
    /// List selected files without reading their source.
    Files(FileArgs),
    /// Read Rust source from stdin and write the result to stdout.
    Stdin,
}

#[derive(clap::Args)]
struct FileArgs {
    /// Files or directories to process; directories are searched for .rs files.
    #[arg(required = true, value_name = "PATH")]
    paths: Vec<PathBuf>,

    /// Exclude matching paths, relative to the configuration directory (or cwd).
    #[arg(long, value_name = "PATTERN", value_parser = parse_exclude)]
    exclude: Vec<String>,
}

fn main() -> ExitCode {
    let CargoCommand::Modfmt(args) = CargoCommand::parse();
    match run(args) {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<ExitCode, Box<dyn Error>> {
    let cwd = std::env::current_dir()?;
    let exclude = match &args.command {
        Command::Check(inputs) | Command::Write(inputs) | Command::Files(inputs) => {
            inputs.exclude.as_slice()
        }
        Command::Stdin => &[],
    };
    let config = Config::load(
        &cwd,
        args.config.as_deref(),
        EditionOverrides {
            edition: args
                .edition
                .map(|value| Resolved::new(value, Origin::CommandLine("--edition"))),
            style_edition: args
                .style_edition
                .map(|value| Resolved::new(value, Origin::CommandLine("--style-edition"))),
        },
        exclude,
    )?;
    let mut formats = FormatResolver::new(config.editions.clone());
    let (mode, inputs) = match args.command {
        Command::Check(inputs) => (Mode::Check, inputs),
        Command::Write(inputs) => (Mode::Write, inputs),
        Command::Files(inputs) => (Mode::Files, inputs),
        Command::Stdin => {
            format_stdin(formats.for_directory(&cwd)?.options())?;
            return Ok(ExitCode::SUCCESS);
        }
    };
    let paths = targets::collect(&inputs.paths, &config)?;
    let inputs = paths
        .into_iter()
        .map(|path| {
            formats.for_file(&path).map(|resolved| Input {
                path,
                options: resolved.options(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(batch::run(&inputs, mode)?)
}

fn format_stdin(options: FormatOptions) -> Result<(), Box<dyn Error>> {
    let mut input = Vec::new();
    io::stdin().lock().read_to_end(&mut input)?;
    let source = String::from_utf8(input)?;
    let output = organize(&source, options)?;
    io::stdout().lock().write_all(output.as_bytes())?;
    Ok(())
}
