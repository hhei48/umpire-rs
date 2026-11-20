use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

pub const DIR_NAME: &str = "testcase";
const INPUT_EXT: &str = "in";
const OUTPUT_EXT: &str = "out";

#[derive(Debug)]
pub struct TestCase {
    pub name: String,
    pub input: PathBuf,
    pub expected: PathBuf,
}

#[derive(Debug, Default)]
pub struct Discovery {
    /// Test cases that have both an input and an expected output, in natural order.
    pub cases: Vec<TestCase>,
    /// Human readable descriptions of files that have no partner.
    pub incomplete: Vec<String>,
}

pub fn dir_of(folder: &Path) -> PathBuf {
    folder.join(DIR_NAME)
}

fn input_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.{INPUT_EXT}"))
}

fn output_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.{OUTPUT_EXT}"))
}

/// Every regular `.in` and `.out` file in `dir`, as `(name, path)` pairs.
fn files_in(dir: &Path) -> Result<Vec<(String, bool, PathBuf)>> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).with_context(|| format!("cannot read {}", dir.display()))?;
    for entry in entries {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let is_input = match path.extension().and_then(|ext| ext.to_str()) {
            Some(INPUT_EXT) => true,
            Some(OUTPUT_EXT) => false,
            _ => continue,
        };
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        files.push((name.to_string(), is_input, path));
    }
    Ok(files)
}

pub fn discover(dir: &Path) -> Result<Discovery> {
    #[derive(Default)]
    struct Pair {
        input: bool,
        output: bool,
    }

    let mut pairs: BTreeMap<String, Pair> = BTreeMap::new();
    for (name, is_input, _) in files_in(dir)? {
        let pair = pairs.entry(name).or_default();
        if is_input {
            pair.input = true;
        } else {
            pair.output = true;
        }
    }

    let mut discovery = Discovery::default();
    for (name, pair) in pairs {
        match (pair.input, pair.output) {
            (true, true) => discovery.cases.push(TestCase {
                input: input_path(dir, &name),
                expected: output_path(dir, &name),
                name,
            }),
            (true, false) => discovery
                .incomplete
                .push(format!("{name}.{INPUT_EXT} has no {name}.{OUTPUT_EXT}")),
            (false, _) => discovery
                .incomplete
                .push(format!("{name}.{OUTPUT_EXT} has no {name}.{INPUT_EXT}")),
        }
    }
    discovery
        .cases
        .sort_by(|a, b| natural_cmp(&a.name, &b.name));
    Ok(discovery)
}

/// Compares strings so that `2` sorts before `10`. Ties such as `02` and `2`
/// fall back to plain string order so the result is always deterministic.
fn natural_cmp(a: &str, b: &str) -> Ordering {
    compare_chunks(a, b).then_with(|| a.cmp(b))
}

fn compare_chunks(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let x = take_number(&mut a);
                let y = take_number(&mut b);
                let ordering = x.len().cmp(&y.len()).then_with(|| x.cmp(&y));
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            (Some(x), Some(y)) => {
                if x != y {
                    return x.cmp(&y);
                }
                a.next();
                b.next();
            }
        }
    }
}

/// Consumes a run of digits and returns it without leading zeros.
fn take_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut digits = String::new();
    while let Some(digit) = chars.next_if(char::is_ascii_digit) {
        digits.push(digit);
    }
    let trimmed = digits.trim_start_matches('0');
    trimmed.to_string()
}

pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("the test case name cannot be empty");
    }
    if name.starts_with('.') {
        bail!("the test case name cannot start with a dot");
    }
    if let Some(bad) = name
        .chars()
        .find(|c| matches!(c, '/' | '\\' | '\0') || c.is_control())
    {
        bail!("the test case name cannot contain {bad:?}");
    }
    Ok(())
}

pub fn exists(dir: &Path, name: &str) -> bool {
    input_path(dir, name).exists() || output_path(dir, name).exists()
}

/// The smallest positive number that is not used by any test case yet.
pub fn next_free_name(dir: &Path) -> String {
    (1u64..)
        .map(|n| n.to_string())
        .find(|name| !exists(dir, name))
        .expect("an unused number exists")
}

/// Writes `<name>.in` and `<name>.out`, creating the folder if needed.
/// Returns the two paths.
pub fn write_case(
    dir: &Path,
    name: &str,
    input: &[u8],
    expected: &[u8],
    overwrite: bool,
) -> Result<(PathBuf, PathBuf)> {
    validate_name(name)?;
    if !overwrite && exists(dir, name) {
        bail!("test case `{name}` already exists, use --force to overwrite it");
    }
    fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;

    let input_path = input_path(dir, name);
    let output_path = output_path(dir, name);
    write_file(&input_path, input, overwrite)?;
    if let Err(err) = write_file(&output_path, expected, overwrite) {
        let _ = fs::remove_file(&input_path);
        return Err(err);
    }
    Ok((input_path, output_path))
}

fn write_file(path: &Path, contents: &[u8], overwrite: bool) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    options
        .open(path)
        .and_then(|mut file| file.write_all(contents))
        .map_err(|err| match err.kind() {
            io::ErrorKind::AlreadyExists => {
                anyhow::anyhow!(
                    "{} already exists, use --force to overwrite it",
                    path.display()
                )
            }
            _ => anyhow::Error::new(err).context(format!("cannot write {}", path.display())),
        })
}

/// All `.in` and `.out` files in `dir`, sorted by path.
pub fn list_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = files_in(dir)?
        .into_iter()
        .map(|(_, _, path)| path)
        .collect();
    paths.sort();
    Ok(paths)
}

pub fn remove_files(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        fs::remove_file(path).with_context(|| format!("cannot delete {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(dir: &Path, name: &str) {
        fs::write(dir.join(name), "x\n").unwrap();
    }

    #[test]
    fn natural_order() {
        let mut names = vec!["10", "2", "1-2", "1-10", "a", "02", "b1"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["1-2", "1-10", "02", "2", "10", "a", "b1"]);
    }

    #[test]
    fn discovery_pairs_files_and_reports_strays() {
        let tmp = tempfile::tempdir().unwrap();
        for file in ["1.in", "1.out", "10.in", "10.out", "2.in", "2.out"] {
            touch(tmp.path(), file);
        }
        for file in ["lonely.in", "orphan.out", "notes.txt", "in", ".out", "x.IN"] {
            touch(tmp.path(), file);
        }
        fs::create_dir(tmp.path().join("dir.in")).unwrap();

        let found = discover(tmp.path()).unwrap();
        let names: Vec<_> = found.cases.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["1", "2", "10"]);
        assert_eq!(found.cases[0].input, tmp.path().join("1.in"));
        assert_eq!(found.cases[0].expected, tmp.path().join("1.out"));
        assert_eq!(found.incomplete.len(), 2);
    }

    #[test]
    fn names_with_dots_keep_their_stem() {
        let tmp = tempfile::tempdir().unwrap();
        touch(tmp.path(), "a.b.in");
        touch(tmp.path(), "a.b.out");
        let found = discover(tmp.path()).unwrap();
        assert_eq!(found.cases[0].name, "a.b");
    }

    #[test]
    fn name_validation() {
        assert!(validate_name("1-1").is_ok());
        assert!(validate_name("sample 2").is_ok());
        for bad in ["", ".hidden", "a/b", "..\\x", "a\nb"] {
            assert!(validate_name(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn next_free_name_skips_used_numbers() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(next_free_name(tmp.path()), "1");
        touch(tmp.path(), "1.in");
        touch(tmp.path(), "3.out");
        touch(tmp.path(), "2.in");
        assert_eq!(next_free_name(tmp.path()), "4");
    }

    #[test]
    fn write_refuses_to_overwrite_unless_forced() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(DIR_NAME);
        write_case(&dir, "1", b"in\n", b"out\n", false).unwrap();
        assert!(write_case(&dir, "1", b"new\n", b"new\n", false).is_err());
        assert_eq!(fs::read(dir.join("1.in")).unwrap(), b"in\n");

        write_case(&dir, "1", b"new\n", b"newer\n", true).unwrap();
        assert_eq!(fs::read(dir.join("1.in")).unwrap(), b"new\n");
        assert_eq!(fs::read(dir.join("1.out")).unwrap(), b"newer\n");
    }

    #[test]
    fn clear_only_lists_test_case_files() {
        let tmp = tempfile::tempdir().unwrap();
        for file in ["1.in", "1.out", "keep.txt", "lonely.in"] {
            touch(tmp.path(), file);
        }
        let files = list_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 3);
        remove_files(&files).unwrap();
        assert!(tmp.path().join("keep.txt").exists());
        assert!(list_files(tmp.path()).unwrap().is_empty());
    }
}
