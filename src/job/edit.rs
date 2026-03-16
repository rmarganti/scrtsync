use crate::secrets::Secrets;
use crate::sources::Source;
use anyhow::{Context, Result};
use std::io::{BufRead, Write};

use super::diff::{build_diff_lines, build_hunks, DiffPrinter};

#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error("unable to create temporary file")]
    CreateTempFile(#[source] std::io::Error),

    #[error("unable to write temporary file")]
    WriteTempFile(#[source] std::io::Error),

    #[error("unable to read temporary file")]
    ReadTempFile(#[source] std::io::Error),

    #[error("editor exited with non-zero status")]
    EditorFailed,

    #[error("unable to launch editor '{0}'")]
    LaunchEditor(String, #[source] std::io::Error),
}

pub struct EditJob {
    source: Box<dyn Source>,
    uri: String,
}

impl EditJob {
    pub fn new(uri: &str) -> Result<Self> {
        let source = <dyn Source>::new(uri)?;
        Ok(Self {
            source,
            uri: uri.to_string(),
        })
    }
}

impl super::Job for EditJob {
    fn run(&self) -> Result<()> {
        let original = self
            .source
            .read_secrets()
            .context("unable to read secrets from source")?;

        let original_content = serialize_secrets(&original)?;

        let temp_file = tempfile::Builder::new()
            .prefix(".scrtsync-edit-")
            .suffix(".env")
            .tempfile()
            .map_err(EditError::CreateTempFile)?;

        std::fs::write(temp_file.path(), &original_content).map_err(EditError::WriteTempFile)?;

        let edited = loop {
            open_editor(temp_file.path())?;

            let edited_bytes = std::fs::read(temp_file.path()).map_err(EditError::ReadTempFile)?;

            if edited_bytes == original_content {
                eprintln!("No changes detected, aborting.");
                return Ok(());
            }

            if edited_bytes.is_empty() || edited_bytes.iter().all(|b| b.is_ascii_whitespace()) {
                eprintln!("Empty file, aborting.");
                return Ok(());
            }

            match Secrets::from_reader(&mut edited_bytes.as_slice()) {
                Ok(secrets) => break secrets,
                Err(e) => {
                    eprintln!("Parse error: {e}");
                    eprintln!("Re-opening editor...");
                    continue;
                }
            }
        };

        let (diff_lines, added, changed, removed) =
            build_diff_lines(&edited.content, &original.content);

        if added == 0 && changed == 0 && removed == 0 {
            eprintln!("No changes detected, aborting.");
            return Ok(());
        }

        let hunks = build_hunks(&diff_lines);
        let printer = DiffPrinter::new();

        printer.print_header(&self.uri);
        for hunk in &hunks {
            printer.print_hunk_header(hunk);
            for line in &hunk.lines {
                printer.print_line(line);
            }
        }

        eprintln!("\n{added} added, {changed} changed, {removed} removed");

        if !confirm("Apply changes?")? {
            eprintln!("Aborted.");
            return Ok(());
        }

        self.source
            .write_secrets(&edited)
            .context("unable to write secrets to source")?;

        eprintln!("Changes applied.");

        Ok(())
    }
}

/// Serialize secrets to bytes for comparison.
fn serialize_secrets(secrets: &Secrets) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    secrets
        .to_writer(&mut buf)
        .context("unable to serialize secrets")?;
    Ok(buf)
}

/// Resolve the user's preferred editor, falling back to `vi`.
fn resolve_editor() -> String {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string())
}

/// Open the given file in the user's editor and wait for it to exit.
fn open_editor(path: &std::path::Path) -> Result<()> {
    let editor = resolve_editor();
    let status = std::process::Command::new(&editor)
        .arg(path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| EditError::LaunchEditor(editor.clone(), e))?;

    if !status.success() {
        return Err(EditError::EditorFailed.into());
    }

    Ok(())
}

/// Prompt the user with `"<message> [y/N] "` on stderr, read from stdin.
/// Returns true only for `y` or `Y`.
fn confirm(message: &str) -> Result<bool> {
    eprint!("{message} [y/N] ");
    std::io::stderr().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn make_secrets(pairs: &[(&str, &str)]) -> Secrets {
        let map: BTreeMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Secrets::from(map)
    }

    #[test]
    fn identical_content_detected_as_no_change() {
        let secrets = make_secrets(&[("FOO", "bar"), ("BAZ", "qux")]);
        let content = serialize_secrets(&secrets).unwrap();

        let reparsed = Secrets::from_reader(&mut content.as_slice()).unwrap();
        let (_, added, changed, removed) = build_diff_lines(&reparsed.content, &secrets.content);

        assert_eq!(added, 0);
        assert_eq!(changed, 0);
        assert_eq!(removed, 0);
    }

    #[test]
    fn empty_content_is_detected() {
        let content = b"";
        assert!(content.is_empty());

        let whitespace_content = b"  \n  \n  ";
        assert!(whitespace_content.iter().all(|b| b.is_ascii_whitespace()));
    }

    #[test]
    fn modified_content_detected_as_changed() {
        let original = make_secrets(&[("FOO", "bar"), ("BAZ", "qux")]);
        let edited = make_secrets(&[("FOO", "changed"), ("BAZ", "qux")]);

        let (_, added, changed, removed) = build_diff_lines(&edited.content, &original.content);

        assert_eq!(added, 0);
        assert_eq!(changed, 1);
        assert_eq!(removed, 0);
    }

    #[test]
    fn added_key_detected() {
        let original = make_secrets(&[("FOO", "bar")]);
        let edited = make_secrets(&[("FOO", "bar"), ("NEW", "value")]);

        let (_, added, changed, removed) = build_diff_lines(&edited.content, &original.content);

        assert_eq!(added, 1);
        assert_eq!(changed, 0);
        assert_eq!(removed, 0);
    }

    #[test]
    fn removed_key_detected() {
        let original = make_secrets(&[("FOO", "bar"), ("BAZ", "qux")]);
        let edited = make_secrets(&[("FOO", "bar")]);

        let (_, added, changed, removed) = build_diff_lines(&edited.content, &original.content);

        assert_eq!(added, 0);
        assert_eq!(changed, 0);
        assert_eq!(removed, 1);
    }
}
