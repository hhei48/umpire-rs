use std::process::ExitCode;
use std::thread;

use anyhow::{bail, Result};

use crate::cli::RunArgs;
use crate::style::{Color, Status, Style};
use crate::verdict_exit_code;
use crate::{compile, judge, report, testcase};

const SOURCE_FILE: &str = "main.cpp";

pub fn run(args: RunArgs) -> Result<ExitCode> {
    let source = args.folder.join(SOURCE_FILE);
    let testcase_dir = testcase::dir_of(&args.folder);

    if !testcase_dir.is_dir() {
        bail!(
            "{} not found, create it with `um add`",
            testcase_dir.display()
        );
    }
    if !source.is_file() {
        bail!("{} not found, only C++ is supported", source.display());
    }

    let discovery = testcase::discover(&testcase_dir)?;
    let stderr_style = Style::stderr();
    for problem in &discovery.incomplete {
        eprintln!(
            "{}: skipped, {problem}",
            stderr_style.paint("Warning", Color::Yellow)
        );
    }
    if discovery.cases.is_empty() {
        bail!(
            "no test cases found in {}, create one with `um add`",
            testcase_dir.display()
        );
    }

    let status = Status::new();
    status.show("Compiling program...");
    let compiled = compile::compile(&source, stderr_style.enabled());
    status.clear();
    let (program, warnings) = compiled?;
    if !warnings.trim().is_empty() {
        eprint!("{warnings}");
    }

    let jobs = args
        .jobs
        .or_else(|| thread::available_parallelism().ok())
        .map_or(1, usize::from);
    status.show("Running...");
    let outcomes = judge::run_all(&program.binary, &discovery.cases, args.time_limit, jobs);
    status.clear();
    let outcomes = outcomes?;

    let style = Style::stdout();
    let summary = report::summarize(&outcomes);
    report::print_table(&outcomes, args.time_limit, style);
    report::print_notes(&outcomes);
    report::print_summary(&summary, args.time_limit, style);
    if args.output {
        report::print_details(&outcomes, style);
    }

    Ok(verdict_exit_code(summary.verdict))
}
