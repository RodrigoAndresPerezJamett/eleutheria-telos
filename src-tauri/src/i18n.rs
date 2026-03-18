use serde_json::Value;
use std::collections::HashMap;

pub struct I18n {
    strings: HashMap<String, String>,
}

impl I18n {
    /// Load the default English locale file. Falls back to empty map on error.
    pub fn load() -> Self {
        let path = "../ui/locales/en.json";
        let strings = match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(Value::Object(map)) => map
                    .into_iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                    .collect(),
                _ => {
                    tracing::warn!("i18n: {path} is not a flat JSON object");
                    HashMap::new()
                }
            },
            Err(err) => {
                tracing::warn!("i18n: failed to load {path}: {err}");
                HashMap::new()
            }
        };
        tracing::info!("i18n: {} strings loaded", strings.len());
        Self { strings }
    }

    /// Resolve a translation key. Returns the key itself if not found.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).map(|s| s.as_str()).unwrap_or(key)
    }
}
