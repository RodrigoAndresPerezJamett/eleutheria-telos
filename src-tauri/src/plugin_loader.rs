use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry: String,
    pub runtime: String,
    pub min_app_version: Option<String>,
    pub icon: Option<String>,
    pub routes: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub sidebar: Option<SidebarConfig>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SidebarConfig {
    pub show: bool,
    pub label: String,
    pub order: Option<u32>,
}

/// Scans `../plugins/*/manifest.json` and returns validated manifests.
/// Malformed or missing manifests are skipped with a warning.
pub fn scan_plugins() -> Vec<PluginManifest> {
    let plugins_dir = Path::new("../plugins");
    if !plugins_dir.exists() {
        return Vec::new();
    }

    let mut manifests = Vec::new();

    let entries = match std::fs::read_dir(plugins_dir) {
        Ok(e) => e,
        Err(err) => {
            tracing::warn!("Failed to read plugins directory: {err}");
            return manifests;
        }
    };

    for entry in entries.flatten() {
        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        match std::fs::read_to_string(&manifest_path) {
            Ok(content) => match serde_json::from_str::<PluginManifest>(&content) {
                Ok(manifest) => {
                    tracing::info!(
                        "Plugin loaded: {} v{} ({})",
                        manifest.name,
                        manifest.version,
                        manifest.runtime
                    );
                    manifests.push(manifest);
                }
                Err(err) => {
                    tracing::warn!("Invalid manifest at {}: {err}", manifest_path.display());
                }
            },
            Err(err) => {
                tracing::warn!("Cannot read {}: {err}", manifest_path.display());
            }
        }
    }

    manifests
}
