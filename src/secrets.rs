use k8s_openapi::ByteString;
use std::collections::BTreeMap;
use std::str;

#[derive(Debug, thiserror::Error)]
pub enum SecretsError {
    #[error("unable to decode env value")]
    DecodeEnv(#[from] dotenvy::Error),

    #[error("unable to encode env value")]
    EncodeValue(#[source] serde_json::Error),

    #[error("unable to decode UTF-8 value for key '{key}'")]
    InvalidUtf8 {
        key: String,
        #[source]
        source: str::Utf8Error,
    },

    #[error("unable to write env entry")]
    WriteEntry(#[source] std::io::Error),
}

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
    pub fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Self, SecretsError> {
        let mut secrets = Self::new();
        let iter = dotenvy::Iter::new(reader);

        for item in iter {
            let (key, value) = item?;
            secrets.content.insert(key, value);
        }

        Ok(secrets)
    }

    /// Write secrets as dotenv-style `KEY="VALUE"` lines.
    /// Dollar signs are escaped as `\$` to prevent dotenvy variable substitution.
    pub fn to_writer<T: std::io::Write>(&self, buf: &mut T) -> Result<(), SecretsError> {
        for (key, value) in &self.content {
            let encoded = serde_json::to_string(value)
                .map_err(SecretsError::EncodeValue)?
                // Escape `$` so that dotenvy does not perform variable substitution
                // when reading the value back. dotenvy recognises `\$` inside
                // double-quoted strings as a literal dollar sign.
                .replace('$', "\\$");

            let line = format!("{}={}\n", key, encoded);

            buf.write_all(line.as_bytes())
                .map_err(SecretsError::WriteEntry)?;
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

impl TryFrom<BTreeMap<String, ByteString>> for Secrets {
    type Error = SecretsError;

    fn try_from(map: BTreeMap<String, ByteString>) -> Result<Self, Self::Error> {
        let mut content = BTreeMap::new();

        for (key, value) in map {
            let string_value = str::from_utf8(&value.0)
                .map_err(|source| SecretsError::InvalidUtf8 {
                    key: key.clone(),
                    source,
                })?
                .to_string();
            content.insert(key, string_value);
        }

        Ok(Self { content })
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
        expected.insert("baz".to_string(), "qu#x".to_string());

        let input = "
            # single line comment
            baz=\"qu#x\" # inline comment
            foo=\"bar\"\n
        ";

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

    #[test]
    fn try_from_btreemap_bytestring_valid() {
        use k8s_openapi::ByteString;

        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), ByteString("bar".as_bytes().to_vec()));
        map.insert("baz".to_string(), ByteString("qux".as_bytes().to_vec()));

        let secrets = Secrets::try_from(map).unwrap();

        let mut expected = BTreeMap::new();
        expected.insert("foo".to_string(), "bar".to_string());
        expected.insert("baz".to_string(), "qux".to_string());

        assert_eq!(secrets.content, expected);
    }

    #[test]
    fn try_from_btreemap_bytestring_invalid_utf8() {
        use k8s_openapi::ByteString;

        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), ByteString("bar".as_bytes().to_vec()));
        // Invalid UTF-8 sequence
        map.insert("invalid".to_string(), ByteString(vec![0xff, 0xfe, 0xfd]));

        let result = Secrets::try_from(map);
        assert!(result.is_err());

        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("unable to decode UTF-8 value for key 'invalid'"));
    }

    #[test]
    fn dollar_sign_is_escaped_on_write() {
        let mut map = BTreeMap::new();
        map.insert("SECRET".to_string(), "p@$$word".to_string());
        let secrets = Secrets::from(map);

        let mut buf: Vec<u8> = vec![];
        secrets.to_writer(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // The dollar signs must be escaped in the written output
        assert_eq!(output, "SECRET=\"p@\\$\\$word\"\n");
    }

    #[test]
    fn dollar_sign_round_trips() {
        let mut map = BTreeMap::new();
        map.insert("A".to_string(), "p@$$word".to_string());
        map.insert("B".to_string(), "$HOME/bin".to_string());
        map.insert("C".to_string(), "${FOO}bar".to_string());
        map.insert("D".to_string(), "no dollar here".to_string());
        let secrets = Secrets::from(map.clone());

        // Write
        let mut buf: Vec<u8> = vec![];
        secrets.to_writer(&mut buf).unwrap();

        // Read back
        let result = Secrets::from_reader(&mut buf.as_slice()).unwrap();

        assert_eq!(result.content, map);
    }

    #[test]
    fn existing_escapes_still_round_trip_with_dollar() {
        // Combining $ with other characters that require escaping
        let mut map = BTreeMap::new();
        map.insert(
            "KEY".to_string(),
            "say \"$NAME\" and cost $5\nnewline".to_string(),
        );
        let secrets = Secrets::from(map.clone());

        let mut buf: Vec<u8> = vec![];
        secrets.to_writer(&mut buf).unwrap();
        let result = Secrets::from_reader(&mut buf.as_slice()).unwrap();

        assert_eq!(result.content, map);
    }
}
