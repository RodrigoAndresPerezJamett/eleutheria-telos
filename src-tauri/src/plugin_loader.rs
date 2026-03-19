use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct SidebarConfig {
    pub show: bool,
    pub label: String,
    pub order: Option<u32>,
}

/// Runtime state for a running plugin: manifest + assigned port.
#[derive(Clone)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub port: u16,
}

/// Maps plugin ID → running plugin info. Guarded by a std Mutex since
/// all operations are brief (no lock held across await points).
pub type PluginRegistry = Arc<std::sync::Mutex<HashMap<String, PluginInfo>>>;

/// Spawns a subprocess for each manifest and returns the populated registry
/// plus the child process handles (kept alive for the duration of the app).
///
/// Each plugin receives these environment variables:
/// - `ELEUTHERIA_APP_PORT`  — Axum server port (for API callbacks)
/// - `ELEUTHERIA_TOKEN`     — session token (for API callbacks)
/// - `ELEUTHERIA_PLUGIN_ID` — this plugin's unique ID
/// - `ELEUTHERIA_PLUGIN_PORT` — port the plugin must listen on
pub fn start_plugins(
    manifests: Vec<PluginManifest>,
    app_port: u16,
    token: &str,
) -> (PluginRegistry, Vec<std::process::Child>) {
    let mut registry = HashMap::new();
    let mut children = Vec::new();

    for manifest in manifests {
        let plugin_port = crate::server::find_free_port_sync();
        let plugin_dir = Path::new("../plugins").join(&manifest.id);

        let mut cmd = match manifest.runtime.as_str() {
            "python" => {
                let mut c = std::process::Command::new("python3");
                c.arg(&manifest.entry);
                c
            }
            "node" => {
                let mut c = std::process::Command::new("node");
                c.arg(&manifest.entry);
                c
            }
            "binary" => std::process::Command::new(plugin_dir.join(&manifest.entry)),
            other => {
                log::warn!("Plugin '{}': unknown runtime '{other}'", manifest.id);
                continue;
            }
        };

        cmd.current_dir(&plugin_dir)
            .env("ELEUTHERIA_APP_PORT", app_port.to_string())
            .env("ELEUTHERIA_TOKEN", token)
            .env("ELEUTHERIA_PLUGIN_ID", &manifest.id)
            .env("ELEUTHERIA_PLUGIN_PORT", plugin_port.to_string());

        match cmd.spawn() {
            Ok(child) => {
                log::info!(
                    "Plugin '{}' v{} started on port {plugin_port}",
                    manifest.name,
                    manifest.version
                );
                registry.insert(
                    manifest.id.clone(),
                    PluginInfo {
                        manifest,
                        port: plugin_port,
                    },
                );
                children.push(child);
            }
            Err(e) => {
                log::warn!("Plugin '{}': failed to start — {e}", manifest.id);
            }
        }
    }

    (Arc::new(std::sync::Mutex::new(registry)), children)
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
