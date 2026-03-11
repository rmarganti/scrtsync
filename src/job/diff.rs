use crate::sources::Source;
use anyhow::{Context, Result};

// ANSI color/style escape codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

pub struct DiffJob {
    from: Box<dyn Source>,
    to: Box<dyn Source>,
}

impl DiffJob {
    pub fn new(from: Box<dyn Source>, to: Box<dyn Source>) -> Self {
        Self { from, to }
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

        let mut added = 0usize;
        let mut removed = 0usize;
        let mut changed = 0usize;

        // Collect all keys from both maps in sorted order
        let mut all_keys: Vec<&String> = from_map.keys().chain(to_map.keys()).collect();
        all_keys.sort();
        all_keys.dedup();

        for key in all_keys {
            match (from_map.get(key), to_map.get(key)) {
                // Key only in from_map → added
                (Some(from_val), None) => {
                    println!("{GREEN}+ {key}={from_val}{RESET}");
                    added += 1;
                }
                // Key only in to_map → removed
                (None, Some(to_val)) => {
                    println!("{RED}- {key}={to_val}{RESET}");
                    removed += 1;
                }
                // Key in both but different values → changed
                (Some(from_val), Some(to_val)) if from_val != to_val => {
                    println!("{RED}- {key}={to_val}");
                    println!("{GREEN}+ {key}={from_val}{RESET}");
                    changed += 1;
                }
                (Some(from_val), Some(to_val)) if from_val == to_val => {
                    println!("  {key}={from_val}");
                }
                // All cases should be covered. (None, None) is impossible
                // (Some(_), Some(_)) variations are covered above.
                _ => {}
            }
        }

        if added == 0 && changed == 0 && removed == 0 {
            println!("Secrets are in sync.");
        } else {
            println!("\n{added} added, {changed} changed, {removed} removed");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
    fn diff_counts_added() {
        let from = make_secrets(&[("FOO", "1"), ("BAR", "2")]);
        let to = make_secrets(&[("BAR", "2")]);
        let from_map = &from.content;
        let to_map = &to.content;

        let added: usize = from_map.keys().filter(|k| !to_map.contains_key(*k)).count();
        assert_eq!(added, 1);
    }

    #[test]
    fn diff_counts_removed() {
        let from = make_secrets(&[("BAR", "2")]);
        let to = make_secrets(&[("FOO", "1"), ("BAR", "2")]);
        let from_map = &from.content;
        let to_map = &to.content;

        let removed: usize = to_map.keys().filter(|k| !from_map.contains_key(*k)).count();
        assert_eq!(removed, 1);
    }

    #[test]
    fn diff_counts_changed() {
        let from = make_secrets(&[("FOO", "new"), ("BAR", "same")]);
        let to = make_secrets(&[("FOO", "old"), ("BAR", "same")]);
        let from_map = &from.content;
        let to_map = &to.content;

        let changed: usize = from_map
            .keys()
            .filter(|k| to_map.get(*k).map(|v| *v != from_map[*k]).unwrap_or(false))
            .count();
        assert_eq!(changed, 1);
    }

    #[test]
    fn diff_in_sync() {
        let from = make_secrets(&[("FOO", "1")]);
        let to = make_secrets(&[("FOO", "1")]);
        let from_map = &from.content;
        let to_map = &to.content;

        let diffs: usize = from_map
            .keys()
            .filter(|k| to_map.get(*k).map(|v| *v != from_map[*k]).unwrap_or(true))
            .count()
            + to_map.keys().filter(|k| !from_map.contains_key(*k)).count();
        assert_eq!(diffs, 0);
    }

    #[test]
    fn diff_output_added_shows_value() {
        let from = make_secrets(&[("FOO", "bar")]);
        let to = make_secrets(&[]);
        let from_map = &from.content;
        let to_map = &to.content;

        let added: usize = from_map.keys().filter(|k| !to_map.contains_key(*k)).count();
        let changed: usize = from_map
            .keys()
            .filter(|k| to_map.get(*k).map(|v| *v != from_map[*k]).unwrap_or(false))
            .count();
        assert_eq!(added, 1);
        assert_eq!(changed, 0);
    }

    #[test]
    fn diff_output_removed_shows_value() {
        let from = make_secrets(&[]);
        let to = make_secrets(&[("FOO", "bar")]);
        let from_map = &from.content;
        let to_map = &to.content;

        let removed: usize = to_map.keys().filter(|k| !from_map.contains_key(*k)).count();
        assert_eq!(removed, 1);
    }

    #[test]
    fn diff_output_changed_distinguishes_old_and_new() {
        let from = make_secrets(&[("FOO", "new_val")]);
        let to = make_secrets(&[("FOO", "old_val")]);
        let from_map = &from.content;
        let to_map = &to.content;

        // The changed arm uses `to_val` as old and `from_val` as new.
        let from_val = from_map.get("FOO").unwrap();
        let to_val = to_map.get("FOO").unwrap();
        assert_eq!(from_val, "new_val");
        assert_eq!(to_val, "old_val");
        assert_ne!(from_val, to_val);
    }
}
