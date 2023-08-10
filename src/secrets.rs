use anyhow::{Context, Result};
use k8s_openapi::ByteString;
use std::collections::BTreeMap;
use std::str;

#[derive(Debug)]
pub struct Secrets {
    pub content: BTreeMap<String, String>,
}

/// Secrets is a container for secrets. It is a wrapper around a BTreeMap,
/// which means secrets are sorted alphabetically by key.
impl Secrets {
    pub fn new() -> Self {
        Self {
            content: BTreeMap::new(),
        }
    }

    /// Read a buffer of dotenv-style `KEY="VALUE"` lines into a Secrets struct.
    pub fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Self> {
        let mut secrets = Self::new();
        let mut buffer = String::new();

        reader
            .read_to_string(&mut buffer)
            .with_context(|| "Unable to read secrets")?;

        for line in buffer.lines() {
            // Ignore comments
            let mut parts = line.split('#');
            let line_body = parts.next().unwrap().trim();

            if line_body.is_empty() {
                continue;
            }

            let mut parts = line_body.splitn(2, '=');
            let key = parts.next().unwrap().trim();
            let mut value = parts.next().unwrap().trim().to_string();

            // Treat everything as a string.
            if !value.starts_with('"') {
                value = format!("\"{}\"", value);
            }

            // Until we find a reason not too, treat all values as JSON strings.
            // This allows us to handle escape characters for free.
            let value =
                serde_json::from_str(&value).with_context(|| "Unable to parse env value")?;

            secrets.content.insert(key.to_string(), value);
        }

        Ok(secrets)
    }

    /// Write secrets as dotenv-style `KEY="VALUE"` lines
    pub fn to_writer<T: std::io::Write>(&self, buf: &mut T) -> Result<()> {
        for (key, value) in &self.content {
            let line = format!(
                "{}={}\n",
                key,
                // JSON-encoding is a convenient way to escape characters.
                serde_json::to_string(value).with_context(|| "Unable to encode env value")?
            );

            buf.write(line.as_bytes())
                .with_context(|| "Unable to write secrets")?;
        }

        Ok(())
    }
}

impl From<BTreeMap<String, String>> for Secrets {
    fn from(map: BTreeMap<String, String>) -> Self {
        Self { content: map }
    }
}

impl From<&BTreeMap<String, String>> for Secrets {
    fn from(map: &BTreeMap<String, String>) -> Self {
        Self {
            content: map.clone(),
        }
    }
}

impl From<BTreeMap<String, ByteString>> for Secrets {
    fn from(map: BTreeMap<String, ByteString>) -> Self {
        Self {
            content: map
                .into_iter()
                .map(|(k, v)| (k, str::from_utf8(&v.0).unwrap().to_string()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Secrets;
    use std::collections::BTreeMap;

    #[test]
    fn from_btreemap_borrow() {
        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), "bar".to_string());
        map.insert("baz".to_string(), "qux".to_string());
        let secrets = Secrets::from(&map);

        assert_eq!(secrets.content, map);
    }

    #[test]
    fn from_btreemap_own() {
        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), "bar".to_string());
        map.insert("baz".to_string(), "qux".to_string());
        let secrets = Secrets::from(map.clone());

        assert_eq!(secrets.content, map);
    }

    #[test]
    fn from_reader() {
        let mut expected = BTreeMap::new();
        expected.insert("foo".to_string(), "bar".to_string());
        expected.insert("baz".to_string(), "qux".to_string());

        let input = "baz=\"qux\"\nfoo=\"bar\"\n";
        let mut buf = input.as_bytes();
        let result = Secrets::from_reader(&mut buf).unwrap();

        assert_eq!(expected, result.content);
    }

    #[test]
    fn to_writer() {
        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), "bar".to_string());
        map.insert("baz".to_string(), "qux".to_string());
        let secrets = Secrets::from(map);

        let mut buf: Vec<u8> = vec![];
        secrets.to_writer(&mut buf).unwrap();
        let secret_string = String::from_utf8(buf).unwrap();

        // Note: Keys are sorted alphabetically
        let expected = "baz=\"qux\"\nfoo=\"bar\"\n";

        assert_eq!(secret_string, expected);
    }

    #[test]
    fn handle_comments_and_white_space() {
        let mut expected = BTreeMap::new();
        expected.insert("foo".to_string(), "bar".to_string());
        expected.insert("baz".to_string(), "qux".to_string());

        let input = r#"
            # This is a comment
            baz="qux"

            foo="bar" # This is another comment
            "#;

        let mut buf = input.as_bytes();
        let result = Secrets::from_reader(&mut buf).unwrap();

        assert_eq!(expected, result.content);
    }
}
