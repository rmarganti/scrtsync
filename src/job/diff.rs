use crate::sources::Source;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::io::IsTerminal;

const CONTEXT_LINES: usize = 3;

pub struct DiffJob {
    from: Box<dyn Source>,
    to: Box<dyn Source>,
    to_uri: String,
}

impl DiffJob {
    pub fn new(from: Box<dyn Source>, to: Box<dyn Source>, to_uri: String) -> Self {
        Self { from, to, to_uri }
    }
}

impl super::Job for DiffJob {
    fn run(&self) -> Result<()> {
        let from_secrets = self
            .from
            .read_secrets()
            .context("unable to read secrets from source")?;

        let to_secrets = self
            .to
            .read_secrets()
            .context("unable to read secrets from target")?;

        let from_map = &from_secrets.content;
        let to_map = &to_secrets.content;

        let (diff_lines, added, changed, removed) = build_diff_lines(from_map, to_map);

        if added == 0 && changed == 0 && removed == 0 {
            eprintln!("Secrets are in sync.");
            return Ok(());
        }

        let hunks = build_hunks(&diff_lines);
        let printer = DiffPrinter::new();

        printer.print_header(&self.to_uri);
        for hunk in &hunks {
            printer.print_hunk_header(hunk);
            for line in &hunk.lines {
                printer.print_line(line);
            }
        }

        eprintln!("\n{added} added, {changed} changed, {removed} removed");

        Ok(())
    }
}

/// Format a key=value pair in dotenv style, matching Secrets::to_writer escaping.
fn format_entry(key: &str, value: &str) -> String {
    let escaped = serde_json::to_string(value).unwrap_or_else(|_| value.to_string());
    format!("{key}={escaped}")
}

#[derive(Clone, Debug)]
enum DiffLine {
    /// Unchanged line surrounding a change
    Context(String),

    /// Line that should be removed from the target
    Remove(String),

    /// Line that should be added to the target
    Add(String),
}

/// Build a list of DiffLine representing the differences between from_map and to_map.
fn build_diff_lines(
    from_map: &BTreeMap<String, String>,
    to_map: &BTreeMap<String, String>,
) -> (Vec<DiffLine>, usize, usize, usize) {
    let mut all_keys: Vec<&String> = from_map.keys().chain(to_map.keys()).collect();
    all_keys.sort();
    all_keys.dedup();

    let mut diff_lines = Vec::new();
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut changed = 0usize;

    for key in &all_keys {
        match (from_map.get(*key), to_map.get(*key)) {
            (Some(from_val), None) => {
                diff_lines.push(DiffLine::Add(format_entry(key, from_val)));
                added += 1;
            }
            (None, Some(to_val)) => {
                diff_lines.push(DiffLine::Remove(format_entry(key, to_val)));
                removed += 1;
            }
            (Some(from_val), Some(to_val)) if from_val != to_val => {
                diff_lines.push(DiffLine::Remove(format_entry(key, to_val)));
                diff_lines.push(DiffLine::Add(format_entry(key, from_val)));
                changed += 1;
            }
            (Some(from_val), Some(_)) => {
                diff_lines.push(DiffLine::Context(format_entry(key, from_val)));
            }
            _ => {}
        }
    }

    (diff_lines, added, changed, removed)
}

/// Group DiffLines into hunks with CONTEXT_LINES of unchanged lines around each change.
struct Hunk {
    old_start: usize,
    new_start: usize,
    lines: Vec<DiffLine>,
}

fn build_hunks(diff_lines: &[DiffLine]) -> Vec<Hunk> {
    // 1. Find indices of all non-context lines
    let change_indices: Vec<usize> = diff_lines
        .iter()
        .enumerate()
        .filter(|(_, line)| !matches!(line, DiffLine::Context(_)))
        .map(|(i, _)| i)
        .collect();

    if change_indices.is_empty() {
        return vec![];
    }

    // 2. Group into hunk ranges
    let mut hunk_ranges: Vec<(usize, usize)> = vec![];
    let mut range_start = change_indices[0].saturating_sub(CONTEXT_LINES);
    let mut range_end = (change_indices[0] + CONTEXT_LINES).min(diff_lines.len() - 1);

    for &idx in &change_indices[1..] {
        let ctx_start = idx.saturating_sub(CONTEXT_LINES);
        if ctx_start <= range_end + 1 {
            range_end = (idx + CONTEXT_LINES).min(diff_lines.len() - 1);
        } else {
            hunk_ranges.push((range_start, range_end));
            range_start = ctx_start;
            range_end = (idx + CONTEXT_LINES).min(diff_lines.len() - 1);
        }
    }
    hunk_ranges.push((range_start, range_end));

    // 3. Build Hunk structs with correct line numbers
    let mut hunks = vec![];
    let mut old_line = 1usize;
    let mut new_line = 1usize;
    let mut pos = 0usize;

    for (start, end) in &hunk_ranges {
        // Advance line counters to `start`
        for line in diff_lines.iter().take(*start).skip(pos) {
            match line {
                DiffLine::Context(_) => {
                    old_line += 1;
                    new_line += 1;
                }
                DiffLine::Remove(_) => {
                    old_line += 1;
                }
                DiffLine::Add(_) => {
                    new_line += 1;
                }
            }
        }

        let hunk_old_start = old_line;
        let hunk_new_start = new_line;
        let lines: Vec<DiffLine> = diff_lines[*start..=*end].to_vec();

        for line in &lines {
            match line {
                DiffLine::Context(_) => {
                    old_line += 1;
                    new_line += 1;
                }
                DiffLine::Remove(_) => {
                    old_line += 1;
                }
                DiffLine::Add(_) => {
                    new_line += 1;
                }
            }
        }

        hunks.push(Hunk {
            old_start: hunk_old_start,
            new_start: hunk_new_start,
            lines,
        });

        pos = end + 1;
    }

    hunks
}

/// Helper for printing diffs with optional color support.
struct DiffPrinter {
    use_color: bool,
}

impl DiffPrinter {
    fn new() -> Self {
        Self {
            use_color: std::io::stdout().is_terminal(),
        }
    }

    fn print_header(&self, uri: &str) {
        if self.use_color {
            println!("\x1b[1m--- a/{uri}\x1b[0m");
            println!("\x1b[1m+++ b/{uri}\x1b[0m");
        } else {
            println!("--- a/{uri}");
            println!("+++ b/{uri}");
        }
    }

    fn print_hunk_header(&self, hunk: &Hunk) {
        let old_count = hunk
            .lines
            .iter()
            .filter(|l| matches!(l, DiffLine::Context(_) | DiffLine::Remove(_)))
            .count();
        let new_count = hunk
            .lines
            .iter()
            .filter(|l| matches!(l, DiffLine::Context(_) | DiffLine::Add(_)))
            .count();
        let header = format!(
            "@@ -{},{} +{},{} @@",
            hunk.old_start, old_count, hunk.new_start, new_count
        );
        if self.use_color {
            println!("\x1b[36m{header}\x1b[0m");
        } else {
            println!("{header}");
        }
    }

    fn print_line(&self, line: &DiffLine) {
        match line {
            DiffLine::Context(text) => println!(" {text}"),
            DiffLine::Remove(text) => {
                if self.use_color {
                    println!("\x1b[31m-{text}\x1b[0m");
                } else {
                    println!("-{text}");
                }
            }
            DiffLine::Add(text) => {
                if self.use_color {
                    println!("\x1b[32m+{text}\x1b[0m");
                } else {
                    println!("+{text}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::Secrets;
    use std::collections::BTreeMap;

    fn make_secrets(pairs: &[(&str, &str)]) -> Secrets {
        let map: BTreeMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Secrets::from(map)
    }

    #[test]
    fn no_diff_when_in_sync() {
        let from = make_secrets(&[("FOO", "1")]);
        let to = make_secrets(&[("FOO", "1")]);
        let (diff_lines, _, _, _) = build_diff_lines(&from.content, &to.content);
        let hunks = build_hunks(&diff_lines);
        assert!(hunks.is_empty());
    }

    #[test]
    fn added_key_shows_plus_line() {
        let from = make_secrets(&[("BAR", "2"), ("FOO", "1")]);
        let to = make_secrets(&[("BAR", "2")]);
        let (diff_lines, added, changed, removed) = build_diff_lines(&from.content, &to.content);
        assert_eq!(added, 1);
        assert_eq!(changed, 0);
        assert_eq!(removed, 0);

        let hunks = build_hunks(&diff_lines);
        assert_eq!(hunks.len(), 1);
        assert!(hunks[0]
            .lines
            .iter()
            .any(|l| matches!(l, DiffLine::Add(s) if s.contains("FOO"))));
    }

    #[test]
    fn removed_key_shows_minus_line() {
        let from = make_secrets(&[("BAR", "2")]);
        let to = make_secrets(&[("BAR", "2"), ("FOO", "1")]);
        let (diff_lines, added, changed, removed) = build_diff_lines(&from.content, &to.content);
        assert_eq!(added, 0);
        assert_eq!(changed, 0);
        assert_eq!(removed, 1);

        let hunks = build_hunks(&diff_lines);
        assert!(hunks[0]
            .lines
            .iter()
            .any(|l| matches!(l, DiffLine::Remove(s) if s.contains("FOO"))));
    }

    #[test]
    fn changed_key_shows_minus_then_plus() {
        let from = make_secrets(&[("FOO", "new")]);
        let to = make_secrets(&[("FOO", "old")]);
        let (diff_lines, added, changed, removed) = build_diff_lines(&from.content, &to.content);
        assert_eq!(added, 0);
        assert_eq!(changed, 1);
        assert_eq!(removed, 0);

        let hunks = build_hunks(&diff_lines);
        let lines = &hunks[0].lines;
        let remove_idx = lines
            .iter()
            .position(|l| matches!(l, DiffLine::Remove(_)))
            .unwrap();
        let add_idx = lines
            .iter()
            .position(|l| matches!(l, DiffLine::Add(_)))
            .unwrap();
        assert!(
            remove_idx < add_idx,
            "Remove line should come before Add line"
        );
    }

    #[test]
    fn values_are_json_escaped() {
        let from = make_secrets(&[("FOO", "hello \"world\"\nnewline")]);
        let to = make_secrets(&[]);
        let (diff_lines, _, _, _) = build_diff_lines(&from.content, &to.content);
        let hunks = build_hunks(&diff_lines);
        let add_line = hunks[0]
            .lines
            .iter()
            .find(|l| matches!(l, DiffLine::Add(_)))
            .unwrap();
        if let DiffLine::Add(text) = add_line {
            assert!(
                text.contains(r#"\"world\""#),
                "Value should be JSON-escaped"
            );
            assert!(text.contains(r#"\n"#), "Newlines should be escaped");
        }
    }

    #[test]
    fn context_lines_are_limited() {
        let pairs_from: Vec<(&str, &str)> = vec![
            ("A01", "v"),
            ("A02", "v"),
            ("A03", "v"),
            ("A04", "v"),
            ("A05", "v"),
            ("A06", "changed"),
            ("A07", "v"),
            ("A08", "v"),
            ("A09", "v"),
            ("A10", "v"),
        ];
        let mut pairs_to = pairs_from.clone();
        pairs_to[5] = ("A06", "original");

        let from = make_secrets(&pairs_from);
        let to = make_secrets(&pairs_to);
        let (diff_lines, _, _, _) = build_diff_lines(&from.content, &to.content);
        let hunks = build_hunks(&diff_lines);

        assert_eq!(hunks.len(), 1);
        // Should have at most 3 context before + 2 change lines + 3 context after = 8
        assert!(hunks[0].lines.len() <= 8);
    }

    #[test]
    fn separate_hunks_for_distant_changes() {
        // Two changes separated by more than 6 unchanged keys should produce 2 hunks
        let pairs_from: Vec<(&str, &str)> = vec![
            ("A01", "changed"),
            ("A02", "v"),
            ("A03", "v"),
            ("A04", "v"),
            ("A05", "v"),
            ("A06", "v"),
            ("A07", "v"),
            ("A08", "v"),
            ("A09", "v"),
            ("A10", "changed"),
        ];
        let mut pairs_to = pairs_from.clone();
        pairs_to[0] = ("A01", "original");
        pairs_to[9] = ("A10", "original");

        let from = make_secrets(&pairs_from);
        let to = make_secrets(&pairs_to);
        let (diff_lines, _, _, _) = build_diff_lines(&from.content, &to.content);
        let hunks = build_hunks(&diff_lines);

        assert_eq!(hunks.len(), 2);
    }

    #[test]
    fn adjacent_changes_merge_into_one_hunk() {
        // Two changes separated by ≤6 unchanged keys should merge into 1 hunk
        let pairs_from: Vec<(&str, &str)> = vec![
            ("A01", "changed"),
            ("A02", "v"),
            ("A03", "v"),
            ("A04", "v"),
            ("A05", "v"),
            ("A06", "v"),
            ("A07", "v"),
            ("A08", "changed"),
        ];
        let mut pairs_to = pairs_from.clone();
        pairs_to[0] = ("A01", "original");
        pairs_to[7] = ("A08", "original");

        let from = make_secrets(&pairs_from);
        let to = make_secrets(&pairs_to);
        let (diff_lines, _, _, _) = build_diff_lines(&from.content, &to.content);
        let hunks = build_hunks(&diff_lines);

        assert_eq!(hunks.len(), 1);
    }

    #[test]
    fn hunk_header_line_numbers_are_correct() {
        // Single change at position 1 (0-indexed), with 1 context before
        let from = make_secrets(&[("A", "v"), ("B", "new"), ("C", "v")]);
        let to = make_secrets(&[("A", "v"), ("B", "old"), ("C", "v")]);
        let (diff_lines, _, _, _) = build_diff_lines(&from.content, &to.content);
        let hunks = build_hunks(&diff_lines);

        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_start, 1);
        assert_eq!(hunks[0].new_start, 1);

        let old_count = hunks[0]
            .lines
            .iter()
            .filter(|l| matches!(l, DiffLine::Context(_) | DiffLine::Remove(_)))
            .count();
        let new_count = hunks[0]
            .lines
            .iter()
            .filter(|l| matches!(l, DiffLine::Context(_) | DiffLine::Add(_)))
            .count();
        assert_eq!(old_count, 3); // A (ctx) + B old (remove) + C (ctx)
        assert_eq!(new_count, 3); // A (ctx) + B new (add) + C (ctx)
    }
}
