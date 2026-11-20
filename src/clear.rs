use std::io::{self, BufRead, IsTerminal, Write};

use anyhow::{bail, Result};

use crate::cli::ClearArgs;
use crate::testcase;

pub fn run(args: ClearArgs) -> Result<()> {
    let dir = testcase::dir_of(&args.dir);
    if !dir.is_dir() {
        println!("Nothing to clear, {} does not exist.", dir.display());
        return Ok(());
    }

    let files = testcase::list_files(&dir)?;
    if files.is_empty() {
        println!("Nothing to clear, {} has no test cases.", dir.display());
        return Ok(());
    }

    if !args.yes {
        if !io::stdin().is_terminal() {
            bail!("refusing to delete without confirmation, pass --yes to confirm");
        }
        let question = format!(
            "Delete {} test case files in {}?",
            files.len(),
            dir.display()
        );
        if !confirm(&question)? {
            println!("Nothing was deleted.");
            return Ok(());
        }
    }

    testcase::remove_files(&files)?;
    println!(
        "Deleted {} test case files from {}",
        files.len(),
        dir.display()
    );
    Ok(())
}

fn confirm(question: &str) -> Result<bool> {
    eprint!("{question} [y/N] ");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
