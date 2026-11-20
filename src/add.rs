use std::io::{self, BufRead, IsTerminal, Write};

use anyhow::{bail, Context, Result};

use crate::cli::AddArgs;
use crate::testcase;

pub fn run(args: AddArgs) -> Result<()> {
    let dir = testcase::dir_of(&args.dir);
    let name = match args.name {
        Some(name) => {
            testcase::validate_name(&name)?;
            name
        }
        None => testcase::next_free_name(&dir),
    };
    if !args.force && testcase::exists(&dir, &name) {
        bail!("test case `{name}` already exists, use --force to overwrite it");
    }

    let stdin = io::stdin();
    if !stdin.is_terminal() {
        bail!("`um add` reads from the terminal, run it without redirecting stdin");
    }
    let mut stdin = stdin.lock();

    let input = read_section(&mut stdin, "input", &name)?;
    let expected = read_section(&mut stdin, "expected output", &name)?;
    let (input_path, output_path) =
        testcase::write_case(&dir, &name, &input, &expected, args.force)?;

    println!(
        "Created {} and {}",
        input_path.display(),
        output_path.display()
    );
    Ok(())
}

/// Reads from the terminal until the user ends the section with Ctrl-D.
fn read_section(reader: &mut impl BufRead, what: &str, name: &str) -> Result<Vec<u8>> {
    eprintln!("Paste the {what} for `{name}`, then press Ctrl-D on an empty line:");
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .with_context(|| format!("cannot read the {what}"))?;
    let _ = io::stderr().flush();
    Ok(finish(bytes))
}

/// Pasted text must end with a newline, like a file written by an editor.
fn finish(mut bytes: Vec<u8>) -> Vec<u8> {
    if !bytes.is_empty() && bytes.last() != Some(&b'\n') {
        bytes.push(b'\n');
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_a_missing_trailing_newline() {
        assert_eq!(finish(b"1 2".to_vec()), b"1 2\n");
        assert_eq!(finish(b"1 2\n".to_vec()), b"1 2\n");
        assert_eq!(finish(Vec::new()), b"");
    }

    #[test]
    fn a_section_ends_at_end_of_input() {
        let mut reader = &b"3\n1 2 3"[..];
        let section = read_section(&mut reader, "input", "1").unwrap();
        assert_eq!(section, b"3\n1 2 3\n");
    }
}
