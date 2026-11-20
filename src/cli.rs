use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "um",
    version,
    about = "A blazingly fast competitive programming helper",
    after_help = "More help available at https://github.com/udontur/umpire.",
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub run: RunArgs,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a test case by pasting its input and output into the terminal
    Add(AddArgs),
    /// Delete every test case file in the testcase folder
    Clear(ClearArgs),
}

/// Compile main.cpp and judge it against every test case (the default action).
#[derive(Args)]
pub struct RunArgs {
    /// Folder that contains main.cpp and the testcase folder
    #[arg(value_name = "FOLDER", default_value = ".")]
    pub folder: PathBuf,

    /// Time limit per test case in seconds, decimals are allowed
    #[arg(
        short,
        long = "time",
        value_name = "SECONDS",
        default_value = "1",
        value_parser = parse_time_limit
    )]
    pub time_limit: Duration,

    /// Print the input, output and expected output of every test case
    #[arg(short, long)]
    pub output: bool,

    /// Number of test cases to run at the same time [default: number of CPUs]
    #[arg(short, long, value_name = "N")]
    pub jobs: Option<NonZeroUsize>,
}

#[derive(Args)]
pub struct AddArgs {
    /// Name of the test case [default: the next free number]
    #[arg(value_name = "NAME")]
    pub name: Option<String>,

    /// Folder that contains the testcase folder
    #[arg(short, long, value_name = "FOLDER", default_value = ".")]
    pub dir: PathBuf,

    /// Overwrite the test case if it already exists
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct ClearArgs {
    /// Folder that contains the testcase folder
    #[arg(short, long, value_name = "FOLDER", default_value = ".")]
    pub dir: PathBuf,

    /// Do not ask for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

fn parse_time_limit(raw: &str) -> Result<Duration, String> {
    let seconds: f64 = raw
        .parse()
        .map_err(|_| format!("`{raw}` is not a number"))?;
    if !(seconds.is_finite() && seconds > 0.0) {
        return Err("the time limit must be greater than zero".into());
    }
    Duration::try_from_secs_f64(seconds).map_err(|_| "the time limit is too large".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_limit_accepts_decimals() {
        assert_eq!(parse_time_limit("0.25"), Ok(Duration::from_millis(250)));
        assert_eq!(parse_time_limit("3"), Ok(Duration::from_secs(3)));
    }

    #[test]
    fn time_limit_rejects_nonsense() {
        for raw in ["0", "-1", "abc", "NaN", "inf", "1e400"] {
            assert!(parse_time_limit(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn cli_definition_is_valid() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
