// plugins.rs — Plugin proxy, sidebar API, and plugin list handlers.
//
// Routes registered:
//   GET  /api/plugins                — JSON list of running plugins
//   GET  /api/plugins/sidebar        — HTMX sidebar items (desktop)
//   GET  /api/plugins/sidebar?layout=tablet — HTMX sidebar items (icon-only)
//   *    /plugins/:plugin_id         — reverse proxy to plugin HTTP server
//   *    /plugins/:plugin_id/*path   — reverse proxy (subpaths)

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{any, get},
    Router,
};
use axum::{extract::Path, http::Request};
use serde::Deserialize;
use serde_json::json;

use crate::server::AppState;

// ── Plugin list ───────────────────────────────────────────────────────────────

pub async fn plugins_list_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let registry = state.plugin_registry.lock().unwrap();
    let plugins: Vec<_> = registry
        .values()
        .map(|p| {
            json!({
                "id": p.manifest.id,
                "name": p.manifest.name,
                "version": p.manifest.version,
                "description": p.manifest.description,
                "runtime": p.manifest.runtime,
                "icon": p.manifest.icon,
                "port": p.port,
            })
        })
        .collect();
    Json(json!({ "plugins": plugins }))
}

// ── Plugin sidebar ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SidebarQuery {
    layout: Option<String>,
}

pub async fn plugins_sidebar_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SidebarQuery>,
) -> impl IntoResponse {
    let tablet = params.layout.as_deref() == Some("tablet");

    let registry = state.plugin_registry.lock().unwrap();
    let mut items: Vec<_> = registry
        .values()
        .filter(|p| p.manifest.sidebar.as_ref().map(|s| s.show).unwrap_or(false))
        .collect::<Vec<_>>();

    // Sort by sidebar.order (ascending), then alphabetically by name.
    items.sort_by_key(|p| {
        (
            p.manifest
                .sidebar
                .as_ref()
                .and_then(|s| s.order)
                .unwrap_or(u32::MAX),
            p.manifest.name.clone(),
        )
    });

    let html = if items.is_empty() {
        String::new()
    } else if tablet {
        items
            .iter()
            .map(|p| {
                let icon = p.manifest.icon.as_deref().unwrap_or("🔌");
                let label = p
                    .manifest
                    .sidebar
                    .as_ref()
                    .map(|s| s.label.as_str())
                    .unwrap_or(&p.manifest.name);
                let id = &p.manifest.id;
                format!(
                    r##"<li>
  <button class="nav-item p-2 rounded-lg hover:bg-gray-800 transition-colors"
          title="{label}"
          hx-get="/plugins/{id}"
          hx-target="#tool-panel"
          hx-swap="innerHTML">{icon}</button>
</li>"##
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        items
            .iter()
            .map(|p| {
                let icon = p.manifest.icon.as_deref().unwrap_or("🔌");
                let label = p
                    .manifest
                    .sidebar
                    .as_ref()
                    .map(|s| s.label.as_str())
                    .unwrap_or(&p.manifest.name);
                let id = &p.manifest.id;
                format!(
                    r##"<li>
  <button class="nav-item w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm hover:bg-gray-800 transition-colors"
          hx-get="/plugins/{id}"
          hx-target="#tool-panel"
          hx-swap="innerHTML">
    <span class="text-lg">{icon}</span>
    <span>{label}</span>
  </button>
</li>"##
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    Html(html)
}

// ── Plugin proxy ──────────────────────────────────────────────────────────────

pub async fn plugin_proxy_handler(
    State(state): State<Arc<AppState>>,
    Path(plugin_id): Path<String>,
    req: Request<Body>,
) -> impl IntoResponse {
    // 1. Look up the plugin in the registry.
    let plugin_info = {
        let registry = state.plugin_registry.lock().unwrap();
        registry.get(&plugin_id).cloned()
    };

    let Some(plugin) = plugin_info else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "plugin not found" })),
        )
            .into_response();
    };

    // 2. Permission check — the plugin must declare this route in its manifest.
    if let Some(routes) = &plugin.manifest.routes {
        let prefix = format!("/plugins/{}", plugin_id);
        let allowed = routes
            .iter()
            .any(|r| r == &prefix || r.starts_with(&format!("{}/", prefix)));
        if !allowed {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "route not declared in plugin manifest" })),
            )
                .into_response();
        }
    }

    // 3. Build the target URL (strip the /plugins/{id} prefix, forward the rest).
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or(req.uri().path());

    let prefix = format!("/plugins/{}", plugin_id);
    let subpath = path_and_query
        .strip_prefix(&prefix)
        .map(|s| if s.is_empty() { "/" } else { s })
        .unwrap_or("/");

    let target_url = format!("http://127.0.0.1:{}{}", plugin.port, subpath);

    // 4. Build reqwest request — forward method, headers, body.
    let method: reqwest::Method = req
        .method()
        .as_str()
        .parse()
        .unwrap_or(reqwest::Method::GET);

    let headers = req.headers().clone();

    let body_bytes = match to_bytes(req.into_body(), 16 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("failed to read request body: {e}") })),
            )
                .into_response()
        }
    };

    let client = reqwest::Client::new();
    let mut rb = client.request(method, &target_url);

    for (name, value) in &headers {
        // Drop hop-by-hop headers; add our own identity headers below.
        if matches!(
            name.as_str(),
            "host" | "authorization" | "connection" | "keep-alive"
        ) {
            continue;
        }
        if let Ok(v) = value.to_str() {
            rb = rb.header(name.as_str(), v);
        }
    }

    // Inject identity so the plugin knows who called it.
    rb = rb
        .header("x-session-token", &state.session_token)
        .header("x-plugin-id", &plugin_id);

    if !body_bytes.is_empty() {
        rb = rb.body(body_bytes.to_vec());
    }

    // 5. Forward the plugin response back to the caller.
    match rb.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            let mut builder = Response::builder().status(status);
            for (name, value) in resp.headers() {
                if !matches!(
                    name.as_str(),
                    "transfer-encoding" | "connection" | "keep-alive"
                ) {
                    builder = builder.header(name, value);
                }
            }

            let body = resp.bytes().await.unwrap_or_default();
            builder
                .body(Body::from(body))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("plugin unreachable: {e}") })),
        )
            .into_response(),
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/plugins", get(plugins_list_handler))
        .route("/api/plugins/sidebar", get(plugins_sidebar_handler))
        // Both routes needed: one for the plugin root, one for subpaths.
        .route("/plugins/:plugin_id", any(plugin_proxy_handler))
        .route("/plugins/:plugin_id/*path", any(plugin_proxy_handler))
}
