use std::collections::BTreeMap;

pub struct Secrets {
    pub content: BTreeMap<String, String>,
}

impl Secrets {
    pub fn new() -> Self {
        Self {
            content: BTreeMap::new(),
        }
    }

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

    pub fn to_env<T: std::io::Write>(&self, buf: &mut T) -> crate::Result<()> {
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
    fn from_btreemap() {
        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), "bar".to_string());
        map.insert("baz".to_string(), "qux".to_string());

        let secrets = Secrets::from(&map);

        assert_eq!(secrets.content, map);
    }
}
