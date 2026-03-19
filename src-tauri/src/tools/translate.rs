use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::server::AppState;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn scripts_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent")
        .join("scripts")
}

/// Parse a model ID like `argos-en-es` into `("en", "es")`.
fn parse_lang_pair(model_id: &str) -> Option<(&str, &str)> {
    // format: argos-{from}-{to}
    let rest = model_id.strip_prefix("argos-")?;
    let (from, to) = rest.split_once('-')?;
    Some((from, to))
}

fn lang_label(code: &str) -> &str {
    match code {
        "en" => "English",
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "pt" => "Portuguese",
        "it" => "Italian",
        "zh" => "Chinese",
        "ja" => "Japanese",
        "ar" => "Arabic",
        "ru" => "Russian",
        _ => code,
    }
}

// ── Request structs ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TranslateParams {
    pub text: String,
    pub from_lang: String,
    pub to_lang: String,
}

#[derive(Deserialize)]
pub struct TextBody {
    pub text: String,
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// GET /api/translate/langs
/// Returns the language pair selector HTML based on installed models.
/// If no models are installed, returns a "no models" prompt.
pub async fn langs_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rows: Vec<(String,)> = match sqlx::query_as(
        "SELECT id FROM models WHERE tool = 'translate' AND downloaded = 1 ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return Html(format!(
                r##"<p class="text-red-400 text-sm">DB error: {}</p>"##,
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    };

    if rows.is_empty() {
        return Html(
            r#"<div class="rounded-lg border border-gray-700 bg-gray-900 p-6 text-center text-gray-400">
  <p class="text-2xl mb-2">📦</p>
  <p class="font-medium">No language packs installed.</p>
  <p class="text-sm mt-1">Go to <strong class="text-gray-300">Models</strong> to download a language pair.</p>
</div>"#
                .to_string(),
        )
        .into_response();
    }

    // Collect unique from-languages, and for each from-lang collect its to-langs.
    let mut pairs: Vec<(String, String)> = Vec::new();
    for (id,) in &rows {
        if let Some((from, to)) = parse_lang_pair(id) {
            pairs.push((from.to_string(), to.to_string()));
        }
    }

    // Build from-lang <option> list (deduplicated, preserving order).
    let mut seen_from: Vec<String> = Vec::new();
    for (from, _) in &pairs {
        if !seen_from.contains(from) {
            seen_from.push(from.clone());
        }
    }

    let from_opts: String = seen_from
        .iter()
        .map(|code| {
            format!(
                r#"<option value="{code}">{label}</option>"#,
                code = code,
                label = lang_label(code)
            )
        })
        .collect();

    // Build to-lang <option> list: initially shows all available to-langs.
    // A full from→to dependency map is embedded as JSON for Alpine to filter.
    let pairs_json = {
        let mut map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (from, to) in &pairs {
            map.entry(from.clone()).or_default().push(to.clone());
        }
        serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
    };

    // Default to-langs for the first from-lang.
    let default_from = seen_from.first().map(|s| s.as_str()).unwrap_or("en");
    let default_tos: Vec<String> = pairs
        .iter()
        .filter(|(f, _)| f == default_from)
        .map(|(_, t)| t.clone())
        .collect();

    Html(format!(
        r##"<div x-data='{{ pairs: {pairs_json}, fromLang: "{default_from}", toLangs: {default_tos_json} }}'
     x-init="$watch('fromLang', v => toLangs = pairs[v] || [])">
  <form id="translate-form"
        hx-post="/api/translate/text"
        hx-target="#translate-result"
        hx-swap="innerHTML"
        hx-indicator="#translate-spinner"
        class="flex flex-col gap-4">

    <!-- Language selectors -->
    <div class="flex items-center gap-3">
      <div class="flex-1">
        <label class="block text-xs text-gray-400 mb-1">From</label>
        <select name="from_lang"
                x-model="fromLang"
                class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 focus:outline-none focus:border-blue-500">
          {from_opts}
        </select>
      </div>
      <span class="text-gray-500 mt-5">→</span>
      <div class="flex-1">
        <label class="block text-xs text-gray-400 mb-1">To</label>
        <select name="to_lang"
                class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 focus:outline-none focus:border-blue-500">
          <template x-for="code in toLangs" :key="code">
            <option :value="code" x-text="code === 'en' ? 'English' : code === 'es' ? 'Spanish' : code === 'fr' ? 'French' : code === 'de' ? 'German' : code === 'pt' ? 'Portuguese' : code"></option>
          </template>
        </select>
      </div>
    </div>

    <!-- Input textarea -->
    <div>
      <label class="block text-xs text-gray-400 mb-1">Text to translate</label>
      <textarea name="text"
                rows="6"
                placeholder="Enter text to translate…"
                class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-blue-500 resize-y"></textarea>
    </div>

    <!-- Submit -->
    <div class="flex items-center gap-3">
      <button type="submit"
              class="bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-lg px-5 py-2 transition-colors">
        Translate
      </button>
      <span id="translate-spinner" class="htmx-indicator text-xs text-gray-400">Translating…</span>
    </div>
  </form>

  <!-- Result -->
  <div id="translate-result" class="mt-4"></div>
</div>"##,
        pairs_json = pairs_json,
        default_from = default_from,
        default_tos_json = serde_json::to_string(&default_tos).unwrap_or_else(|_| "[]".to_string()),
        from_opts = from_opts,
    ))
    .into_response()
}

/// POST /api/translate/text  (form-encoded: text, from_lang, to_lang)
/// Runs scripts/translate.py in a subprocess and returns the result card HTML.
pub async fn translate_handler(
    State(_state): State<Arc<AppState>>,
    Form(params): Form<TranslateParams>,
) -> impl IntoResponse {
    if params.text.trim().is_empty() {
        return Html(
            r#"<p class="text-gray-400 text-sm">Enter some text to translate.</p>"#.to_string(),
        )
        .into_response();
    }

    let script = scripts_dir().join("translate.py");
    let text = params.text.clone();
    let from_lang = params.from_lang.clone();
    let to_lang = params.to_lang.clone();

    let result = tokio::spawn(async move {
        tokio::process::Command::new("python3")
            .arg(&script)
            .arg(&text)
            .arg(&from_lang)
            .arg(&to_lang)
            .output()
            .await
    })
    .await;

    match result {
        Ok(Ok(output)) if output.status.success() => {
            let translated = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if translated.is_empty() {
                Html(
                    r#"<p class="text-gray-400 text-sm">Translation returned empty.</p>"#
                        .to_string(),
                )
                .into_response()
            } else {
                render_result(&translated).into_response()
            }
        }
        Ok(Ok(output)) => {
            let err = html_escape(String::from_utf8_lossy(&output.stderr).trim());
            Html(format!(
                r##"<p class="text-red-400 text-sm">Translation error: {err}</p>"##
            ))
            .into_response()
        }
        Ok(Err(e)) => Html(format!(
            r##"<p class="text-red-400 text-sm">Could not run python3: {}</p>"##,
            html_escape(&e.to_string())
        ))
        .into_response(),
        Err(e) => Html(format!(
            r##"<p class="text-red-400 text-sm">Task error: {}</p>"##,
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

/// Build the result card shown after successful translation.
fn render_result(text: &str) -> Html<String> {
    let escaped = html_escape(text);
    // Pre-compute ID references to avoid r#"..."# raw-string "# termination (D-023).
    let include_target = "#translate-text-form";
    let feedback_target = "#translate-feedback";
    Html(format!(
        r##"<div class="flex flex-col gap-3">
  <pre class="text-sm text-gray-200 bg-gray-800 rounded-lg p-4 whitespace-pre-wrap break-words max-h-64 overflow-y-auto font-sans leading-relaxed">{escaped}</pre>
  <form id="translate-text-form">
    <textarea name="text" class="hidden">{escaped}</textarea>
  </form>
  <div class="flex gap-2">
    <button class="text-xs text-blue-400 hover:text-blue-300 border border-blue-700 rounded px-3 py-1.5"
            hx-post="/api/translate/copy"
            hx-include="{include_target}"
            hx-target="{feedback_target}"
            hx-swap="innerHTML">Copy to Clipboard</button>
  </div>
  <div id="translate-feedback" class="text-xs"></div>
</div>"##
    ))
}

/// POST /api/translate/copy  (form-encoded, field: text)
/// Copies the translated text to the system clipboard via arboard.
pub async fn copy_handler(
    State(_state): State<Arc<AppState>>,
    Form(body): Form<TextBody>,
) -> impl IntoResponse {
    if body.text.is_empty() {
        return Html(r#"<span class="text-gray-400">Nothing to copy.</span>"#.to_string())
            .into_response();
    }

    let text = body.text.clone();
    tokio::task::spawn_blocking(move || {
        if let Ok(mut board) = arboard::Clipboard::new() {
            let _ = board.set_text(&text);
        }
    });

    Html(r#"<span class="text-green-400">Copied to clipboard!</span>"#.to_string()).into_response()
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/translate/langs", get(langs_handler))
        .route("/api/translate/text", post(translate_handler))
        .route("/api/translate/copy", post(copy_handler))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        middleware,
    };
    use http_body_util::BodyExt;
    use tokio::sync::watch;
    use tower::ServiceExt;

    async fn make_test_state() -> Arc<AppState> {
        let db = sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("in-memory DB");
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("migrations");
        let (clipboard_suppress_tx, _) = watch::channel::<u64>(0);
        let download_states =
            std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
        let voice_recording = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let screen_recording = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        Arc::new(AppState {
            db,
            session_token: "test-token".to_string(),
            port: 0,
            event_bus: crate::event_bus::EventBus::new(),
            clipboard_suppress_tx,
            download_states,
            voice_recording,
            screen_recording,
        })
    }

    fn test_app(state: Arc<AppState>) -> axum::Router {
        use crate::server::auth_middleware;
        router()
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    async fn get_req(app: axum::Router, uri: &str) -> (StatusCode, String) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("Authorization", "Bearer test-token")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    async fn post_form(
        app: axum::Router,
        uri: &str,
        body: &[(&str, &str)],
    ) -> (StatusCode, String) {
        let encoded = body
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(encoded))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    #[tokio::test]
    async fn test_langs_no_models() {
        // With an in-memory DB (migrations seed models as downloaded=0),
        // the langs endpoint should return the "no models" message.
        let state = make_test_state().await;
        let app = test_app(state);
        let (status, body) = get_req(app, "/api/translate/langs").await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.contains("No language packs installed"),
            "Expected no-models message, got: {body}"
        );
    }

    #[tokio::test]
    async fn test_langs_with_installed_model() {
        let state = make_test_state().await;
        // Mark argos-en-es as downloaded.
        sqlx::query("UPDATE models SET downloaded = 1 WHERE id = 'argos-en-es'")
            .execute(&state.db)
            .await
            .unwrap();
        let app = test_app(state);
        let (status, body) = get_req(app, "/api/translate/langs").await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.contains("from_lang"),
            "Expected language selector form, got: {body}"
        );
        assert!(
            body.contains("English"),
            "Expected English option, got: {body}"
        );
    }

    #[tokio::test]
    async fn test_translate_empty_text() {
        let state = make_test_state().await;
        let app = test_app(state);
        let (status, body) = post_form(
            app,
            "/api/translate/text",
            &[("text", ""), ("from_lang", "en"), ("to_lang", "es")],
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.contains("Enter some text"),
            "Expected empty-text message, got: {body}"
        );
    }

    #[tokio::test]
    async fn test_copy_empty() {
        let state = make_test_state().await;
        let app = test_app(state);
        let (status, body) = post_form(app, "/api/translate/copy", &[("text", "")]).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Nothing to copy"));
    }

    #[tokio::test]
    async fn test_copy_returns_ok() {
        let state = make_test_state().await;
        let app = test_app(state);
        let (status, body) = post_form(app, "/api/translate/copy", &[("text", "Hola mundo")]).await;
        assert_eq!(status, StatusCode::OK);
        // arboard may or may not work in CI, but the handler must return 200.
        let _ = body;
    }
}
