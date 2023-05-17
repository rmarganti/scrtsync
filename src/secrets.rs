use std::collections::BTreeMap;

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
    pub fn from_reader<T: std::io::Read>(reader: &mut T) -> crate::Result<Self> {
        let mut secrets = Self::new();

        let mut buffer = String::new();
        reader.read_to_string(&mut buffer)?;

        for line in buffer.lines() {
            let mut parts = line.split('=');
            let key = parts.next().unwrap();
            let value = parts.next().unwrap().trim_matches('"');

            secrets.content.insert(key.to_string(), value.to_string());
        }

        Ok(secrets)
    }

    /// Write secrets as dotenv-style `KEY="VALUE"` lines
    pub fn to_writer<T: std::io::Write>(&self, buf: &mut T) -> crate::Result<()> {
        for (key, value) in &self.content {
            buf.write(format!("{}=\"{}\"\n", key, value).as_bytes())?;
        }

        Ok(())
    }
}

impl From<BTreeMap<String, String>> for Secrets {
    // Create a Secrets from a BTreeMap
    fn from(map: BTreeMap<String, String>) -> Self {
        Self { content: map }
    }
}

impl From<&BTreeMap<String, String>> for Secrets {
    // Create a Secrets from a BTreeMap
    fn from(map: &BTreeMap<String, String>) -> Self {
        Self {
            content: map.clone(),
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
    fn to_env() {
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
}
