// quick_actions.rs — Visual pipeline builder: CRUD routes + execution engine.
//
// Routes registered:
//   GET  /api/pipelines                           — list HTML
//   POST /api/pipelines                           — create pipeline
//   PUT  /api/pipelines/:id                       — update name/trigger/enabled
//   DELETE /api/pipelines/:id                     — delete pipeline
//   GET  /api/pipelines/:id/editor                — editor panel HTML
//   POST /api/pipelines/:id/run                   — manual run
//   POST /api/pipelines/:id/steps                 — add step
//   DELETE /api/pipelines/:id/steps/:step_id      — remove step
//   POST /api/pipelines/:id/steps/:step_id/move   — reorder (up / down)

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Form, Router,
};
use serde::Deserialize;

use crate::event_bus::Event;
use crate::server::AppState;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn scripts_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent")
        .join("scripts")
}

fn trigger_label(trigger_json: &str) -> &'static str {
    if trigger_json.contains("OcrCompleted") {
        "📷 After OCR"
    } else if trigger_json.contains("TranscriptionCompleted") {
        "🎙 After Voice"
    } else if trigger_json.contains("ClipboardChanged") {
        "📋 On Copy"
    } else {
        "⚡ Manual"
    }
}

fn tool_icon(tool: &str) -> &'static str {
    match tool {
        "translate" => "🌐",
        "copy_clipboard" => "📋",
        "save_note" => "📝",
        _ => "⚙️",
    }
}

fn tool_label(tool: &str) -> &'static str {
    match tool {
        "translate" => "Translate",
        "copy_clipboard" => "Copy to Clipboard",
        "save_note" => "Save as Note",
        _ => "Unknown step",
    }
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
        "auto" | "" => "Auto",
        _ => code,
    }
}

fn config_summary(tool: &str, config_json: &str) -> String {
    let config: serde_json::Value =
        serde_json::from_str(config_json).unwrap_or(serde_json::json!({}));
    match tool {
        "translate" => {
            let from = config["from_lang"].as_str().unwrap_or("auto");
            let to = config["to_lang"].as_str().unwrap_or("?");
            format!("{} → {}", lang_label(from), lang_label(to))
        }
        "save_note" => {
            let title = config["title"].as_str().unwrap_or("").trim().to_string();
            if title.is_empty() {
                "Title: first line".to_string()
            } else {
                format!("Title: {}", html_escape(&title))
            }
        }
        _ => String::new(),
    }
}

// ── DB row types ──────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow, Clone)]
struct PipelineRow {
    id: String,
    name: String,
    trigger: String,
    enabled: i64,
    #[allow(dead_code)]
    created_at: i64,
}

#[derive(sqlx::FromRow, Clone)]
struct StepRow {
    id: String,
    #[allow(dead_code)]
    pipeline_id: String,
    step_order: i64,
    tool: String,
    config: String,
}

// ── HTML renderers ────────────────────────────────────────────────────────────

fn render_pipeline_list(pipelines: &[PipelineRow]) -> String {
    let items: String = pipelines
        .iter()
        .map(|p| {
            let name = html_escape(&p.name);
            let tlabel = trigger_label(&p.trigger);
            let enabled_dot = if p.enabled != 0 {
                r#"<span class="w-2 h-2 rounded-full bg-green-500 shrink-0" title="Enabled"></span>"#
            } else {
                r#"<span class="w-2 h-2 rounded-full bg-gray-600 shrink-0" title="Disabled"></span>"#
            };
            let is_manual = p.trigger.contains("Manual");
            let run_btn = if is_manual {
                format!(
                    r##"<button class="text-xs text-green-400 hover:text-green-300 border border-green-800 rounded px-2 py-1 transition-colors"
                             hx-post="/api/pipelines/{id}/run"
                             hx-target="#qa-run-result"
                             hx-swap="innerHTML">▶</button>"##,
                    id = p.id
                )
            } else {
                String::new()
            };
            format!(
                r##"<li class="flex items-center gap-2 p-3 bg-gray-800 hover:bg-gray-750 rounded-lg group">
  {enabled_dot}
  <div class="flex-1 min-w-0 cursor-pointer"
       hx-get="/api/pipelines/{id}/editor"
       hx-target="#qa-editor"
       hx-swap="innerHTML">
    <p class="text-sm font-medium text-gray-200 truncate">{name}</p>
    <p class="text-xs text-gray-500">{tlabel}</p>
  </div>
  {run_btn}
  <button class="text-xs text-red-400 hover:text-red-300 opacity-0 group-hover:opacity-100 transition-opacity px-1"
          hx-delete="/api/pipelines/{id}"
          hx-target="#pipeline-list"
          hx-swap="outerHTML"
          hx-confirm="Delete pipeline «{name}»?">✕</button>
</li>"##,
                id = p.id,
                name = name,
                tlabel = tlabel,
                enabled_dot = enabled_dot,
                run_btn = run_btn,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let empty = if pipelines.is_empty() {
        r#"<p class="text-xs text-gray-600 px-1 py-2">No pipelines yet. Create one above.</p>"#
    } else {
        ""
    };

    format!(
        r#"<ul id="pipeline-list" class="space-y-1.5">{items}{empty}</ul>"#,
        items = items,
        empty = empty
    )
}

fn render_steps(pipeline_id: &str, steps: &[StepRow]) -> String {
    if steps.is_empty() {
        return format!(
            r#"<div id="steps-{pid}" class="text-xs text-gray-600 italic py-2">No steps yet. Add one below.</div>"#,
            pid = pipeline_id
        );
    }

    let mut parts: Vec<String> = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let icon = tool_icon(&step.tool);
        let label = tool_label(&step.tool);
        let summary = config_summary(&step.tool, &step.config);
        let summary_html = if summary.is_empty() {
            String::new()
        } else {
            format!(
                r#"<p class="text-xs text-gray-500 mt-0.5">{}</p>"#,
                summary
            )
        };

        let up_disabled = if i == 0 { "opacity-30 cursor-not-allowed" } else { "hover:text-gray-200" };
        let down_disabled = if i == steps.len() - 1 { "opacity-30 cursor-not-allowed" } else { "hover:text-gray-200" };

        parts.push(format!(
            r##"<div class="flex items-start gap-3 bg-gray-800 rounded-lg p-3">
  <span class="text-xl shrink-0 mt-0.5">{icon}</span>
  <div class="flex-1 min-w-0">
    <p class="text-sm font-medium text-gray-200">{label}</p>
    {summary_html}
  </div>
  <div class="flex flex-col gap-0.5 shrink-0">
    <button class="text-gray-500 text-xs leading-none {up_disabled}"
            hx-post="/api/pipelines/{pid}/steps/{sid}/move"
            hx-vals='{{"direction":"up"}}'
            hx-target="#steps-{pid}"
            hx-swap="outerHTML">▲</button>
    <button class="text-gray-500 text-xs leading-none {down_disabled}"
            hx-post="/api/pipelines/{pid}/steps/{sid}/move"
            hx-vals='{{"direction":"down"}}'
            hx-target="#steps-{pid}"
            hx-swap="outerHTML">▼</button>
  </div>
  <button class="text-gray-600 hover:text-red-400 text-sm shrink-0 transition-colors"
          hx-delete="/api/pipelines/{pid}/steps/{sid}"
          hx-target="#steps-{pid}"
          hx-swap="outerHTML">✕</button>
</div>"##,
            icon = icon,
            label = label,
            summary_html = summary_html,
            pid = pipeline_id,
            sid = step.id,
            up_disabled = up_disabled,
            down_disabled = down_disabled,
        ));

        if i < steps.len() - 1 {
            parts.push(r#"<div class="text-center text-gray-700 text-xs select-none">↓</div>"#.to_string());
        }
    }

    format!(
        r#"<div id="steps-{pid}" class="space-y-1">{content}</div>"#,
        pid = pipeline_id,
        content = parts.join("\n")
    )
}

fn render_editor(pipeline: &PipelineRow, steps: &[StepRow]) -> String {
    let name = html_escape(&pipeline.name);
    let steps_html = render_steps(&pipeline.id, steps);
    let enabled_checked = if pipeline.enabled != 0 { "checked" } else { "" };
    let trigger_opts = [
        (r#"{"type":"Manual"}"#, "⚡ Manual"),
        (r#"{"type":"OcrCompleted"}"#, "📷 After OCR"),
        (r#"{"type":"TranscriptionCompleted"}"#, "🎙 After Voice"),
        (r#"{"type":"ClipboardChanged"}"#, "📋 On Clipboard Change"),
    ]
    .iter()
    .map(|(val, label)| {
        let selected = if pipeline.trigger == *val { "selected" } else { "" };
        format!(r#"<option value="{}" {}>{}</option>"#, html_escape(val), selected, label)
    })
    .collect::<Vec<_>>()
    .join("");

    let lang_opts = [
        ("auto", "Auto"),
        ("en", "English"),
        ("es", "Spanish"),
        ("fr", "French"),
        ("de", "German"),
        ("pt", "Portuguese"),
        ("it", "Italian"),
    ]
    .iter()
    .map(|(code, label)| format!(r#"<option value="{code}">{label}</option>"#))
    .collect::<Vec<_>>()
    .join("");

    let is_manual = pipeline.trigger.contains("Manual");
    let run_section = if is_manual {
        format!(
            r##"<div class="border-t border-gray-700 pt-4"
                  x-data="{{showRun: false, runText: ''}}">
  <button class="text-xs text-green-400 hover:text-green-300 border border-green-800 rounded px-3 py-1.5 transition-colors"
          @click="showRun = !showRun">▶ Run Now</button>
  <div x-show="showRun" class="mt-2 flex gap-2">
    <input x-model="runText"
           type="text"
           placeholder="Initial text (optional)"
           class="flex-1 bg-gray-700 text-white text-xs rounded px-2 py-1.5 focus:outline-none focus:ring-1 focus:ring-green-600" />
    <button class="text-xs bg-green-700 hover:bg-green-600 text-white rounded px-3 py-1.5 transition-colors"
            :hx-vals="JSON.stringify({{initial_text: runText}})"
            hx-post="/api/pipelines/{id}/run"
            hx-target="#qa-run-result"
            hx-swap="innerHTML">Go</button>
  </div>
  <div id="qa-run-result" class="mt-3 text-xs"></div>
</div>"##,
            id = pipeline.id
        )
    } else {
        String::new()
    };

    format!(
        r##"<!-- Editor for pipeline {id} -->
<div class="flex flex-col gap-4">

  <!-- Header + metadata form -->
  <form class="flex flex-col gap-3"
        hx-put="/api/pipelines/{id}"
        hx-target="#pipeline-list"
        hx-swap="outerHTML">
    <div class="flex items-center gap-3">
      <input name="name" value="{name}"
             class="flex-1 bg-gray-700 text-white text-sm rounded px-3 py-1.5 focus:outline-none focus:ring-1 focus:ring-blue-600"
             required />
      <label class="flex items-center gap-1.5 text-xs text-gray-400 cursor-pointer">
        <input type="checkbox" name="enabled" value="1" {enabled_checked}
               class="accent-blue-500" />
        Enabled
      </label>
      <button type="submit"
              class="text-xs bg-blue-700 hover:bg-blue-600 text-white rounded px-3 py-1.5 transition-colors">
        Save
      </button>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-xs text-gray-500">Trigger:</span>
      <select name="trigger"
              class="bg-gray-700 text-white text-xs rounded px-2 py-1.5 focus:outline-none">
        {trigger_opts}
      </select>
    </div>
  </form>

  <!-- Steps -->
  <div>
    <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">Steps</p>
    {steps_html}
  </div>

  <!-- Add step -->
  <div x-data="{{
    tool: 'translate',
    from_lang: 'auto',
    to_lang: 'en',
    note_title: ''
  }}" class="border border-gray-700 rounded-lg p-3 flex flex-col gap-3">
    <p class="text-xs font-semibold text-gray-400">Add step</p>

    <select x-model="tool"
            class="bg-gray-700 text-white text-xs rounded px-2 py-1.5 w-full focus:outline-none">
      <option value="translate">🌐 Translate</option>
      <option value="copy_clipboard">📋 Copy to Clipboard</option>
      <option value="save_note">📝 Save as Note</option>
    </select>

    <div x-show="tool === 'translate'" class="flex gap-2">
      <div class="flex-1">
        <label class="text-xs text-gray-500 block mb-1">From</label>
        <select x-model="from_lang" class="w-full bg-gray-700 text-white text-xs rounded px-2 py-1.5 focus:outline-none">
          {lang_opts}
        </select>
      </div>
      <div class="flex-1">
        <label class="text-xs text-gray-500 block mb-1">To</label>
        <select x-model="to_lang" class="w-full bg-gray-700 text-white text-xs rounded px-2 py-1.5 focus:outline-none">
          {lang_opts}
        </select>
      </div>
    </div>

    <div x-show="tool === 'save_note'">
      <label class="text-xs text-gray-500 block mb-1">Note title (blank = first line of text)</label>
      <input x-model="note_title" type="text" placeholder="Optional title"
             class="w-full bg-gray-700 text-white text-xs rounded px-2 py-1.5 focus:outline-none" />
    </div>

    <button class="text-xs bg-gray-700 hover:bg-gray-600 text-white rounded px-3 py-1.5 transition-colors self-start"
            :hx-vals="JSON.stringify({{
              tool: tool,
              config: JSON.stringify(
                tool === 'translate' ? {{from_lang: from_lang, to_lang: to_lang}} :
                tool === 'save_note' ? {{title: note_title}} :
                {{}}
              )
            }})"
            hx-post="/api/pipelines/{id}/steps"
            hx-target="#steps-{id}"
            hx-swap="outerHTML">
      + Add Step
    </button>
  </div>

  {run_section}
</div>"##,
        id = pipeline.id,
        name = name,
        enabled_checked = enabled_checked,
        trigger_opts = trigger_opts,
        lang_opts = lang_opts,
        steps_html = steps_html,
        run_section = run_section,
    )
}

// ── Request structs ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateParams {
    name: String,
    trigger: String,
}

#[derive(Deserialize)]
struct UpdateParams {
    name: String,
    trigger: String,
    enabled: Option<String>,
}

#[derive(Deserialize)]
struct AddStepParams {
    tool: String,
    config: Option<String>,
}

#[derive(Deserialize)]
struct MoveParams {
    direction: String,
}

#[derive(Deserialize)]
struct RunParams {
    initial_text: Option<String>,
}

// ── CRUD handlers ─────────────────────────────────────────────────────────────

async fn list_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pipelines: Vec<PipelineRow> =
        sqlx::query_as("SELECT id, name, trigger, enabled, created_at FROM pipelines ORDER BY created_at ASC")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
    Html(render_pipeline_list(&pipelines))
}

async fn create_handler(
    State(state): State<Arc<AppState>>,
    Form(params): Form<CreateParams>,
) -> impl IntoResponse {
    let name = params.name.trim().to_string();
    if name.is_empty() {
        return Html(r#"<p class="text-red-400 text-xs">Name required.</p>"#.to_string());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_secs();
    let _ = sqlx::query(
        "INSERT INTO pipelines (id, name, trigger, enabled, created_at) VALUES (?, ?, ?, 1, ?)",
    )
    .bind(&id)
    .bind(&name)
    .bind(&params.trigger)
    .bind(now)
    .execute(&state.db)
    .await;

    let pipelines: Vec<PipelineRow> =
        sqlx::query_as("SELECT id, name, trigger, enabled, created_at FROM pipelines ORDER BY created_at ASC")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
    Html(render_pipeline_list(&pipelines))
}

async fn update_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Form(params): Form<UpdateParams>,
) -> impl IntoResponse {
    let name = params.name.trim().to_string();
    let enabled: i64 = if params.enabled.as_deref() == Some("1") { 1 } else { 0 };
    let _ = sqlx::query(
        "UPDATE pipelines SET name = ?, trigger = ?, enabled = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&params.trigger)
    .bind(enabled)
    .bind(&id)
    .execute(&state.db)
    .await;

    let pipelines: Vec<PipelineRow> =
        sqlx::query_as("SELECT id, name, trigger, enabled, created_at FROM pipelines ORDER BY created_at ASC")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
    Html(render_pipeline_list(&pipelines))
}

async fn delete_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM pipelines WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    let pipelines: Vec<PipelineRow> =
        sqlx::query_as("SELECT id, name, trigger, enabled, created_at FROM pipelines ORDER BY created_at ASC")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
    Html(render_pipeline_list(&pipelines))
}

async fn editor_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pipeline: Option<PipelineRow> = sqlx::query_as(
        "SELECT id, name, trigger, enabled, created_at FROM pipelines WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some(pipeline) = pipeline else {
        return Html(r#"<p class="text-red-400 text-sm">Pipeline not found.</p>"#.to_string());
    };

    let steps: Vec<StepRow> = sqlx::query_as(
        "SELECT id, pipeline_id, step_order, tool, config FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Html(render_editor(&pipeline, &steps))
}

async fn add_step_handler(
    State(state): State<Arc<AppState>>,
    Path(pipeline_id): Path<String>,
    Form(params): Form<AddStepParams>,
) -> impl IntoResponse {
    let max_order: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(step_order), 0) FROM pipeline_steps WHERE pipeline_id = ?",
    )
    .bind(&pipeline_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let step_id = uuid::Uuid::new_v4().to_string();
    let config = params.config.unwrap_or_else(|| "{}".to_string());
    let _ = sqlx::query(
        "INSERT INTO pipeline_steps (id, pipeline_id, step_order, tool, config) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&step_id)
    .bind(&pipeline_id)
    .bind(max_order + 1)
    .bind(&params.tool)
    .bind(&config)
    .execute(&state.db)
    .await;

    let steps: Vec<StepRow> = sqlx::query_as(
        "SELECT id, pipeline_id, step_order, tool, config FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC",
    )
    .bind(&pipeline_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Html(render_steps(&pipeline_id, &steps))
}

async fn delete_step_handler(
    State(state): State<Arc<AppState>>,
    Path((pipeline_id, step_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM pipeline_steps WHERE id = ? AND pipeline_id = ?")
        .bind(&step_id)
        .bind(&pipeline_id)
        .execute(&state.db)
        .await;

    // Re-normalize step_order so there are no gaps.
    let remaining: Vec<StepRow> = sqlx::query_as(
        "SELECT id, pipeline_id, step_order, tool, config FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC",
    )
    .bind(&pipeline_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (i, step) in remaining.iter().enumerate() {
        let _ = sqlx::query("UPDATE pipeline_steps SET step_order = ? WHERE id = ?")
            .bind((i + 1) as i64)
            .bind(&step.id)
            .execute(&state.db)
            .await;
    }

    let steps: Vec<StepRow> = sqlx::query_as(
        "SELECT id, pipeline_id, step_order, tool, config FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC",
    )
    .bind(&pipeline_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Html(render_steps(&pipeline_id, &steps))
}

async fn move_step_handler(
    State(state): State<Arc<AppState>>,
    Path((pipeline_id, step_id)): Path<(String, String)>,
    Form(params): Form<MoveParams>,
) -> impl IntoResponse {
    let steps: Vec<StepRow> = sqlx::query_as(
        "SELECT id, pipeline_id, step_order, tool, config FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC",
    )
    .bind(&pipeline_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if let Some(pos) = steps.iter().position(|s| s.id == step_id) {
        let swap_pos = if params.direction == "up" {
            if pos == 0 { None } else { Some(pos - 1) }
        } else {
            if pos + 1 >= steps.len() { None } else { Some(pos + 1) }
        };

        if let Some(other_pos) = swap_pos {
            let a = &steps[pos];
            let b = &steps[other_pos];
            // Swap step_order values.
            let _ = sqlx::query("UPDATE pipeline_steps SET step_order = ? WHERE id = ?")
                .bind(b.step_order)
                .bind(&a.id)
                .execute(&state.db)
                .await;
            let _ = sqlx::query("UPDATE pipeline_steps SET step_order = ? WHERE id = ?")
                .bind(a.step_order)
                .bind(&b.id)
                .execute(&state.db)
                .await;
        }
    }

    let steps: Vec<StepRow> = sqlx::query_as(
        "SELECT id, pipeline_id, step_order, tool, config FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC",
    )
    .bind(&pipeline_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Html(render_steps(&pipeline_id, &steps))
}

// ── Manual run handler ────────────────────────────────────────────────────────

async fn run_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Form(params): Form<RunParams>,
) -> impl IntoResponse {
    let initial = StepValue {
        text: params.initial_text.filter(|s| !s.trim().is_empty()),
    };
    match run_pipeline_steps(&id, initial, &state).await {
        Ok(final_val) => {
            let output = final_val
                .text
                .map(|t| {
                    format!(
                        r#"<p class="text-green-400">✓ Done: <span class="font-mono">{}</span></p>"#,
                        html_escape(&t.chars().take(120).collect::<String>())
                    )
                })
                .unwrap_or_else(|| {
                    r#"<p class="text-green-400">✓ Pipeline completed.</p>"#.to_string()
                });
            Html(output)
        }
        Err(e) => Html(format!(
            r#"<p class="text-red-400">✗ {}</p>"#,
            html_escape(&e)
        )),
    }
}

// ── Execution engine ──────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct StepValue {
    pub text: Option<String>,
}

async fn run_pipeline_steps(
    pipeline_id: &str,
    initial: StepValue,
    state: &Arc<AppState>,
) -> Result<StepValue, String> {
    let steps: Vec<StepRow> = sqlx::query_as(
        "SELECT id, pipeline_id, step_order, tool, config FROM pipeline_steps \
         WHERE pipeline_id = ? ORDER BY step_order ASC",
    )
    .bind(pipeline_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let mut value = initial;
    for step in &steps {
        value = execute_step(step, value, state).await?;
    }
    Ok(value)
}

async fn execute_step(
    step: &StepRow,
    input: StepValue,
    state: &Arc<AppState>,
) -> Result<StepValue, String> {
    match step.tool.as_str() {
        "translate" => {
            let config: serde_json::Value =
                serde_json::from_str(&step.config).unwrap_or(serde_json::json!({}));
            let from = config["from_lang"]
                .as_str()
                .unwrap_or("auto")
                .to_string();
            let to = config["to_lang"].as_str().unwrap_or("en").to_string();
            let text = input
                .text
                .ok_or_else(|| "translate: no text input".to_string())?;
            let translated = run_translate_raw(&text, &from, &to).await?;
            Ok(StepValue {
                text: Some(translated),
            })
        }
        "copy_clipboard" => {
            let text = input
                .text
                .clone()
                .ok_or_else(|| "copy_clipboard: no text input".to_string())?;
            // Suppress clipboard monitor from re-recording this copy.
            let hash = {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                text.hash(&mut h);
                h.finish()
            };
            let _ = state.clipboard_suppress_tx.send(hash);
            tokio::task::spawn_blocking(move || {
                arboard::Clipboard::new()
                    .and_then(|mut c| c.set_text(text))
                    .map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| e.to_string())??;
            Ok(input)
        }
        "save_note" => {
            let config: serde_json::Value =
                serde_json::from_str(&step.config).unwrap_or(serde_json::json!({}));
            let title_cfg = config["title"].as_str().unwrap_or("").trim().to_string();
            let text = input
                .text
                .clone()
                .ok_or_else(|| "save_note: no text input".to_string())?;
            let title = if title_cfg.is_empty() {
                text.lines()
                    .next()
                    .unwrap_or("Quick Action Note")
                    .chars()
                    .take(80)
                    .collect()
            } else {
                title_cfg
            };
            let id = uuid::Uuid::new_v4().to_string();
            let now = now_secs();
            sqlx::query(
                "INSERT INTO notes (id, title, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&title)
            .bind(&text)
            .bind(now)
            .bind(now)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
            state.event_bus.publish(Event::NoteCreated {
                id,
                title,
            });
            Ok(input)
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

async fn run_translate_raw(
    text: &str,
    from_lang: &str,
    to_lang: &str,
) -> Result<String, String> {
    let script = scripts_dir().join("translate.py");
    let output = tokio::process::Command::new("python3")
        .arg(&script)
        .arg(text)
        .arg(from_lang)
        .arg(to_lang)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Subscribes to the Event Bus and runs pipelines whose trigger matches incoming events.
/// Spawned as a background task in lib.rs.
pub async fn start_pipeline_engine(state: Arc<AppState>) {
    let mut rx = state.event_bus.subscribe();
    loop {
        let event = match rx.recv().await {
            Ok(e) => e,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                log::warn!("Quick Actions engine: lagged by {n} events, some triggers may have been missed");
                continue;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        };

        let (trigger_type, initial) = match &event {
            Event::OcrCompleted { text, .. } => (
                "OcrCompleted",
                StepValue {
                    text: Some(text.clone()),
                },
            ),
            Event::TranscriptionCompleted { text, .. } => (
                "TranscriptionCompleted",
                StepValue {
                    text: Some(text.clone()),
                },
            ),
            Event::ClipboardChanged { content, .. } => (
                "ClipboardChanged",
                StepValue {
                    text: Some(content.clone()),
                },
            ),
            _ => continue,
        };

        // Load matching enabled pipelines.
        let pattern = format!("%{}%", trigger_type);
        let pipelines: Vec<PipelineRow> = sqlx::query_as(
            "SELECT id, name, trigger, enabled, created_at FROM pipelines \
             WHERE enabled = 1 AND trigger LIKE ?",
        )
        .bind(&pattern)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for pipeline in pipelines {
            let state = state.clone();
            let val = initial.clone();
            tokio::spawn(async move {
                if let Err(e) = run_pipeline_steps(&pipeline.id, val, &state).await {
                    log::warn!(
                        "Quick Actions: pipeline '{}' failed — {e}",
                        pipeline.name
                    );
                } else {
                    log::info!("Quick Actions: pipeline '{}' completed.", pipeline.name);
                }
            });
        }
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/pipelines", get(list_handler).post(create_handler))
        .route(
            "/api/pipelines/:id",
            put(update_handler).delete(delete_handler),
        )
        .route("/api/pipelines/:id/editor", get(editor_handler))
        .route("/api/pipelines/:id/run", post(run_handler))
        .route("/api/pipelines/:id/steps", post(add_step_handler))
        .route(
            "/api/pipelines/:id/steps/:step_id",
            delete(delete_step_handler),
        )
        .route(
            "/api/pipelines/:id/steps/:step_id/move",
            post(move_step_handler),
        )
}
