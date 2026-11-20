use std::fs;
use std::time::Duration;

use crate::judge::{Outcome, Verdict};
use crate::style::{Color, Style};

/// How much of a file `--output` prints before cutting it off.
const PREVIEW_BYTES: usize = 2000;

pub struct Summary {
    pub verdict: Verdict,
    pub slowest: Duration,
    pub passed: usize,
    pub total: usize,
}

pub fn summarize(outcomes: &[Outcome]) -> Summary {
    Summary {
        verdict: outcomes
            .iter()
            .map(|outcome| outcome.verdict)
            .max()
            .unwrap_or(Verdict::Accepted),
        slowest: outcomes
            .iter()
            .map(|outcome| outcome.elapsed)
            .max()
            .unwrap_or_default(),
        passed: outcomes
            .iter()
            .filter(|outcome| outcome.verdict == Verdict::Accepted)
            .count(),
        total: outcomes.len(),
    }
}

fn color_of(verdict: Verdict) -> Color {
    match verdict {
        Verdict::Accepted => Color::Green,
        Verdict::WrongAnswer => Color::Red,
        Verdict::RuntimeError => Color::Orange,
        Verdict::TimeLimitExceeded => Color::Yellow,
    }
}

fn format_seconds(duration: Duration) -> String {
    format!("{:.3}s", duration.as_secs_f64())
}

fn format_limit(limit: Duration) -> String {
    let text = format!("{:.3}", limit.as_secs_f64());
    format!("{}s", text.trim_end_matches('0').trim_end_matches('.'))
}

fn format_time(verdict: Verdict, elapsed: Duration, limit: Duration) -> String {
    if verdict == Verdict::TimeLimitExceeded {
        format!(">{}", format_limit(limit))
    } else {
        format_seconds(elapsed)
    }
}

pub fn print_table(outcomes: &[Outcome], limit: Duration, style: Style) {
    let rows: Vec<[String; 3]> = outcomes
        .iter()
        .map(|outcome| {
            [
                outcome.name.clone(),
                outcome.verdict.name().to_string(),
                format_time(outcome.verdict, outcome.elapsed, limit),
            ]
        })
        .collect();

    let header = ["Name", "Verdict", "Time"];
    let mut widths = header.map(|title| title.chars().count());
    for row in &rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    let rule = |left: &str, fill: &str, middle: &str, right: &str| {
        let segments: Vec<String> = widths.iter().map(|w| fill.repeat(w + 2)).collect();
        println!("{left}{}{right}", segments.join(middle));
    };
    let pad = |cell: &str, width: usize| {
        let padding = " ".repeat(width - cell.chars().count());
        format!("{cell}{padding}")
    };

    rule("┌", "─", "┬", "┐");
    let titles: Vec<String> = header
        .iter()
        .zip(&widths)
        .map(|(title, width)| style.bold(&pad(title, *width)))
        .collect();
    println!("│ {} │", titles.join(" │ "));
    rule("╞", "═", "╪", "╡");
    for (row, outcome) in rows.iter().zip(outcomes) {
        let cells = [
            pad(&row[0], widths[0]),
            style.paint(&pad(&row[1], widths[1]), color_of(outcome.verdict)),
            pad(&row[2], widths[2]),
        ];
        println!("│ {} │", cells.join(" │ "));
    }
    rule("└", "─", "┴", "┘");
}

/// Explains failures the table cannot, such as how a program crashed.
pub fn print_notes(outcomes: &[Outcome]) {
    for outcome in outcomes {
        if let Some(detail) = &outcome.detail {
            println!("{}: {}: {detail}", outcome.name, outcome.verdict.name());
        }
    }
}

pub fn print_summary(summary: &Summary, limit: Duration, style: Style) {
    let time = format_time(summary.verdict, summary.slowest, limit);
    let line = format!(
        " {} (Time: {time}, {}/{} passed) ",
        summary.verdict.name(),
        summary.passed,
        summary.total
    );
    println!("{}", style.badge(&line, color_of(summary.verdict)));
}

pub fn print_details(outcomes: &[Outcome], style: Style) {
    for outcome in outcomes {
        println!();
        println!("{}", style.bold(&format!("== {} ==", outcome.name)));
        let input = fs::read(&outcome.input).unwrap_or_default();
        print_section("Input", &input);
        print_section("Output", &outcome.actual);
        if outcome.verdict != Verdict::Accepted {
            print_section("Expected", &outcome.expected);
        }
        if !outcome.stderr.is_empty() {
            print_section("Stderr", &outcome.stderr);
        }
    }
}

fn print_section(title: &str, bytes: &[u8]) {
    println!("{title}:");
    println!("{}", preview(bytes));
}

fn preview(bytes: &[u8]) -> String {
    let shown = &bytes[..bytes.len().min(PREVIEW_BYTES)];
    let mut text = String::from_utf8_lossy(shown).trim_end().to_string();
    if bytes.len() > PREVIEW_BYTES {
        text.push_str(&format!(
            "\n... ({} more bytes)",
            bytes.len() - PREVIEW_BYTES
        ));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(verdict: Verdict, millis: u64) -> Outcome {
        Outcome {
            name: "x".into(),
            verdict,
            elapsed: Duration::from_millis(millis),
            detail: None,
            input: Default::default(),
            actual: vec![],
            expected: vec![],
            stderr: vec![],
        }
    }

    #[test]
    fn summary_takes_the_worst_verdict_and_slowest_time() {
        let outcomes = [
            outcome(Verdict::Accepted, 10),
            outcome(Verdict::RuntimeError, 30),
            outcome(Verdict::WrongAnswer, 20),
        ];
        let summary = summarize(&outcomes);
        assert_eq!(summary.verdict, Verdict::RuntimeError);
        assert_eq!(summary.slowest, Duration::from_millis(30));
        assert_eq!((summary.passed, summary.total), (1, 3));
    }

    #[test]
    fn limits_are_printed_without_trailing_zeros() {
        assert_eq!(format_limit(Duration::from_secs(1)), "1s");
        assert_eq!(format_limit(Duration::from_millis(1500)), "1.5s");
        assert_eq!(format_limit(Duration::from_millis(250)), "0.25s");
        assert_eq!(format_limit(Duration::from_secs(10)), "10s");
    }

    #[test]
    fn long_files_are_cut_off() {
        assert_eq!(preview(b"hi\n\n"), "hi");
        let long = vec![b'a'; PREVIEW_BYTES + 5];
        assert!(preview(&long).ends_with("... (5 more bytes)"));
    }
}
