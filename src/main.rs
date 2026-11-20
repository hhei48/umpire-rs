mod add;
mod clear;
mod cli;
mod compile;
mod judge;
mod report;
mod run;
mod style;
mod testcase;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command};
use judge::Verdict;
use style::{Color, Style};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Command::Add(args)) => add::run(args).map(|()| ExitCode::SUCCESS),
        Some(Command::Clear(args)) => clear::run(args).map(|()| ExitCode::SUCCESS),
        None => run::run(cli.run),
    };

    match result {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}: {err:#}", Style::stderr().paint("Error", Color::Red));
            ExitCode::from(2)
        }
    }
}

/// 0 when every test case passed, 1 when any failed. Errors use 2.
fn verdict_exit_code(verdict: Verdict) -> ExitCode {
    if verdict == Verdict::Accepted {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
