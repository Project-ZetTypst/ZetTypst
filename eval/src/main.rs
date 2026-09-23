use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand, ValueHint};
use typst::foundations::{Dict, Value};
use zettyp_eval::{Runtime, WorldOptions};

#[cfg(unix)]
mod rpc;

#[derive(Parser)]
#[command(
    name = "zettyp-eval",
    subcommand_negates_reqs = true,
    args_conflicts_with_subcommands = true
)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Command>,

    /// Path to the input Typst file (one-shot evaluation).
    #[arg(value_name = "INPUT", value_hint = ValueHint::FilePath, required = true)]
    input: Option<PathBuf>,

    /// Configure the project root.
    #[arg(long, value_name = "DIR", value_hint = ValueHint::DirPath)]
    root: Option<PathBuf>,

    /// Add a string key-value pair visible through `sys.inputs`.
    #[arg(
        long = "input",
        value_name = "key=value",
        action = ArgAction::Append,
        value_parser = parse_sys_input_pair,
    )]
    inputs: Vec<(String, String)>,

    #[command(flatten)]
    environment: EnvironmentArgs,
}

#[derive(Args)]
struct EnvironmentArgs {
    /// Additional font directories, searched before system and embedded fonts.
    #[arg(long = "font-path", env = "TYPST_FONT_PATHS", value_delimiter = if cfg!(windows) { ';' } else { ':' }, value_hint = ValueHint::DirPath)]
    font_paths: Vec<PathBuf>,

    /// Disable system font discovery (embedded fonts remain available).
    #[arg(long)]
    ignore_system_fonts: bool,

    /// Override the local Typst package directory.
    #[arg(long, env = "TYPST_PACKAGE_PATH", value_hint = ValueHint::DirPath)]
    package_path: Option<PathBuf>,

    /// Override the directory for cached and downloaded Typst packages.
    #[arg(long, env = "TYPST_PACKAGE_CACHE_PATH", value_hint = ValueHint::DirPath)]
    package_cache_path: Option<PathBuf>,
}

impl From<EnvironmentArgs> for WorldOptions {
    fn from(args: EnvironmentArgs) -> Self {
        Self {
            font_paths: args.font_paths,
            ignore_system_fonts: args.ignore_system_fonts,
            package_path: args.package_path,
            package_cache_path: args.package_cache_path,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Keep a project's Typst state alive and accept local JSON-RPC requests.
    Serve {
        #[arg(long, default_value = ".", value_name = "DIR", value_hint = ValueHint::DirPath)]
        root: PathBuf,
        #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        socket: PathBuf,
        #[command(flatten)]
        environment: EnvironmentArgs,
    },
}

fn main() -> Result<()> {
    let arguments = Arguments::parse();
    if let Some(Command::Serve {
        root,
        socket,
        environment,
    }) = arguments.command
    {
        #[cfg(unix)]
        return rpc::serve(&root, &socket, environment.into());
        #[cfg(not(unix))]
        {
            let _ = (root, socket, environment);
            anyhow::bail!("Unix socket serving is only available on Unix platforms");
        }
    }
    let input = arguments.input.context("input file is required")?;
    let input = input
        .canonicalize()
        .with_context(|| format!("failed to resolve input {}", input.display()))?;
    let root = arguments
        .root
        .as_deref()
        .or_else(|| input.parent())
        .context("input has no parent directory")?
        .canonicalize()
        .context("failed to resolve project root")?;
    let entry = input.strip_prefix(&root).with_context(|| {
        format!(
            "input {} is outside project root {}",
            input.display(),
            root.display(),
        )
    })?;

    let inputs = arguments
        .inputs
        .into_iter()
        .map(|(key, value)| (key.into(), Value::Str(value.into())))
        .collect::<Dict>();
    let mut runtime = Runtime::new_with_options(root, arguments.environment.into())?;
    let evaluation = runtime.evaluate(entry, inputs)?;
    for warning in &evaluation.result.warnings {
        eprintln!("warning: {}", warning.message);
    }

    let output = evaluation
        .result
        .output
        .as_ref()
        .map_err(|error| anyhow::anyhow!("{error:#}"))?;
    serde_json::to_writer_pretty(std::io::stdout(), output)?;
    println!();
    Ok(())
}

fn parse_sys_input_pair(raw: &str) -> Result<(String, String), String> {
    let (key, value) = raw
        .split_once('=')
        .ok_or("input must be a key and a value separated by an equal sign")?;
    let key = key.trim().to_owned();
    if key.is_empty() {
        return Err("the key was missing or empty".to_owned());
    }
    Ok((key, value.trim().to_owned()))
}
