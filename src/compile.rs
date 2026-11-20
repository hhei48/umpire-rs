use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use tempfile::TempDir;

const DEFAULT_COMPILER: &str = "g++";
const FLAGS: [&str; 2] = ["-std=c++17", "-O2"];

/// A compiled program. The binary lives in a temporary folder that is
/// deleted when this value is dropped.
pub struct Program {
    pub binary: PathBuf,
    _dir: TempDir,
}

/// Compiles `source` with `$CXX` (default `g++`).
/// Compiler warnings are returned so the caller can decide how to show them.
pub fn compile(source: &Path, color: bool) -> Result<(Program, String)> {
    let compiler = env::var("CXX").unwrap_or_default();
    let mut words = compiler.split_whitespace();
    let executable = words.next().unwrap_or(DEFAULT_COMPILER);

    let dir = tempfile::Builder::new()
        .prefix("umpire-")
        .tempdir()
        .context("cannot create a temporary folder")?;
    let binary = dir.path().join(format!("main{}", env::consts::EXE_SUFFIX));

    let mut command = Command::new(executable);
    command.args(words).args(FLAGS);
    if color {
        command.arg("-fdiagnostics-color=always");
    }
    command.arg(source).arg("-o").arg(&binary);

    let output = command.output().with_context(|| {
        format!("cannot run `{executable}`, install a C++ compiler or set the CXX variable")
    })?;
    let diagnostics = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        bail!(
            "cannot compile {}\n{}",
            source.display(),
            diagnostics.trim_end()
        );
    }
    Ok((Program { binary, _dir: dir }, diagnostics))
}
