use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use wait_timeout::ChildExt;

use crate::testcase::TestCase;

/// A program that prints more than this is judged wrong instead of eating all memory.
const OUTPUT_LIMIT: u64 = 64 * 1024 * 1024;
const STDERR_LIMIT: u64 = 64 * 1024;

/// Ordered from best to worst, so the final verdict is the maximum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    Accepted,
    WrongAnswer,
    RuntimeError,
    TimeLimitExceeded,
}

impl Verdict {
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Accepted => "Accepted",
            Verdict::WrongAnswer => "Wrong Answer",
            Verdict::RuntimeError => "Runtime Error",
            Verdict::TimeLimitExceeded => "Time Limit Exceeded",
        }
    }
}

pub struct Outcome {
    pub name: String,
    pub verdict: Verdict,
    pub elapsed: Duration,
    /// Extra information such as the exit status of a crashed program.
    pub detail: Option<String>,
    pub input: PathBuf,
    pub actual: Vec<u8>,
    pub expected: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Two outputs match when they contain the same whitespace separated tokens.
pub fn outputs_match(expected: &[u8], actual: &[u8]) -> bool {
    fn tokens(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
        bytes
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty())
    }
    tokens(expected).eq(tokens(actual))
}

/// Reads at most `limit` bytes, then discards the rest so the writer never blocks.
/// Returns the bytes and whether anything was discarded.
fn read_capped(mut stream: impl Read, limit: u64) -> (Vec<u8>, bool) {
    let mut kept = Vec::new();
    let _ = (&mut stream).take(limit).read_to_end(&mut kept);
    let truncated = io::copy(&mut stream, &mut io::sink()).is_ok_and(|rest| rest > 0);
    (kept, truncated)
}

pub fn run_case(binary: &Path, case: &TestCase, time_limit: Duration) -> Result<Outcome> {
    let expected = std::fs::read(&case.expected)
        .with_context(|| format!("cannot read {}", case.expected.display()))?;
    let input =
        File::open(&case.input).with_context(|| format!("cannot open {}", case.input.display()))?;

    let mut child = Command::new(binary)
        .stdin(input)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("cannot run {}", binary.display()))?;
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let stdout_reader = thread::spawn(move || read_capped(stdout, OUTPUT_LIMIT));
    let stderr_reader = thread::spawn(move || read_capped(stderr, STDERR_LIMIT));

    let start = Instant::now();
    let status = child.wait_timeout(time_limit)?;
    let elapsed = start.elapsed();
    if status.is_none() {
        child.kill()?;
        child.wait()?;
    }

    let (actual, output_truncated) = stdout_reader.join().expect("reader thread panicked");
    let (stderr, _) = stderr_reader.join().expect("reader thread panicked");

    let (verdict, detail) = match status {
        None => (Verdict::TimeLimitExceeded, None),
        Some(status) if !status.success() => (Verdict::RuntimeError, Some(status.to_string())),
        Some(_) if output_truncated => (
            Verdict::WrongAnswer,
            Some("the program printed too much output".to_string()),
        ),
        Some(_) if outputs_match(&expected, &actual) => (Verdict::Accepted, None),
        Some(_) => (Verdict::WrongAnswer, None),
    };

    Ok(Outcome {
        name: case.name.clone(),
        verdict,
        elapsed,
        detail,
        input: case.input.clone(),
        actual,
        expected,
        stderr,
    })
}

/// Runs every test case with up to `jobs` of them in parallel.
/// The outcomes are returned in the same order as `cases`.
pub fn run_all(
    binary: &Path,
    cases: &[TestCase],
    time_limit: Duration,
    jobs: usize,
) -> Result<Vec<Outcome>> {
    let next = AtomicUsize::new(0);
    let workers = jobs.clamp(1, cases.len().max(1));

    let mut finished: Vec<(usize, Result<Outcome>)> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(case) = cases.get(index) else { break };
                        done.push((index, run_case(binary, case, time_limit)));
                    }
                    done
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("worker thread panicked"))
            .collect()
    });

    finished.sort_by_key(|(index, _)| *index);
    finished.into_iter().map(|(_, outcome)| outcome).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_is_ignored() {
        assert!(outputs_match(b"1 2\n3\n", b"1\n2   3"));
        assert!(outputs_match(b"", b"\n \n"));
        assert!(outputs_match(b"a\r\nb\r\n", b"a\nb\n"));
    }

    #[test]
    fn different_tokens_do_not_match() {
        assert!(!outputs_match(b"1 2", b"1 3"));
        assert!(!outputs_match(b"1 2", b"1"));
        assert!(!outputs_match(b"1", b"1 2"));
        assert!(!outputs_match(b"1", b""));
        assert!(!outputs_match(b"", b"1"));
        assert!(!outputs_match(b"12", b"1 2"));
    }

    #[test]
    fn verdicts_are_ordered_by_severity() {
        assert!(Verdict::TimeLimitExceeded > Verdict::RuntimeError);
        assert!(Verdict::RuntimeError > Verdict::WrongAnswer);
        assert!(Verdict::WrongAnswer > Verdict::Accepted);
    }

    #[test]
    fn capped_reader_discards_the_rest() {
        let (kept, truncated) = read_capped(&b"abcdef"[..], 4);
        assert_eq!(kept, b"abcd");
        assert!(truncated);
        let (kept, truncated) = read_capped(&b"abcd"[..], 4);
        assert_eq!(kept, b"abcd");
        assert!(!truncated);
    }
}
