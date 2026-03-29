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
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Form, Json, Router,
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
        "After OCR"
    } else if trigger_json.contains("TranscriptionCompleted") {
        "After Voice"
    } else if trigger_json.contains("ClipboardChanged") {
        "On Copy"
    } else {
        "Manual"
    }
}

fn tool_icon(tool: &str) -> &'static str {
    match tool {
        "translate" => r#"<i data-lucide="languages" style="width:14px;height:14px;"></i>"#,
        "copy_clipboard" => r#"<i data-lucide="clipboard" style="width:14px;height:14px;"></i>"#,
        "save_note" => r#"<i data-lucide="notebook-pen" style="width:14px;height:14px;"></i>"#,
        _ => r#"<i data-lucide="settings-2" style="width:14px;height:14px;"></i>"#,
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
    folder_id: Option<String>,
}

#[derive(sqlx::FromRow)]
struct FolderRow {
    id: String,
    name: String,
    #[allow(dead_code)]
    sort_order: i64,
}

// ── YAML structs (export / import) ────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct PipelineYaml {
    name: String,
    trigger: String,
    #[serde(default)]
    nodes: Vec<NodeYaml>,
    #[serde(default)]
    edges: Vec<EdgeYaml>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeYaml {
    id: String,
    #[serde(rename = "type")]
    node_type: String,
    #[serde(default)]
    config: serde_json::Value,
    #[serde(default)]
    pos_x: f64,
    #[serde(default)]
    pos_y: f64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EdgeYaml {
    id: String,
    source: String,
    target: String,
    #[serde(default = "default_edge_label")]
    label: String,
}

fn default_edge_label() -> String {
    "default".to_string()
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

/// Render a single pipeline `<li>` row, with a folder `<select>` for moving.
fn render_pipeline_item(p: &PipelineRow, folders: &[FolderRow]) -> String {
    let name = html_escape(&p.name);
    let name_js = name.replace('\'', "\\'");
    let tlabel = trigger_label(&p.trigger);
    let dot_color = if p.enabled != 0 { "var(--success)" } else { "var(--border)" };

    let run_btn = if p.trigger.contains("Manual") {
        format!(
            r##"<button class="btn btn-ghost btn-sm" style="padding:3px;opacity:0;transition:opacity 150ms;"
                     onmouseenter="this.style.opacity=1" onmouseleave="this.style.opacity=0"
                     hx-post="/api/pipelines/{id}/run" hx-target="#qa-run-result" hx-swap="innerHTML"
                     title="Run pipeline">
              <i data-lucide="play" style="width:11px;height:11px;"></i>
            </button>"##,
            id = p.id
        )
    } else {
        String::new()
    };

    // Folder <select> — shows on hover; on change, PUTs folder_id
    let none_sel = if p.folder_id.is_none() { " selected" } else { "" };
    let mut folder_opts = format!("<option value=''{none_sel}>No folder</option>", none_sel = none_sel);
    for f in folders {
        let sel = if p.folder_id.as_deref() == Some(f.id.as_str()) { " selected" } else { "" };
        folder_opts.push_str(&format!(
            "<option value='{fid}'{sel}>{fname}</option>",
            fid = f.id,
            fname = html_escape(&f.name),
            sel = sel
        ));
    }

    format!(
        r##"<li style="display:flex;align-items:center;gap:6px;padding:8px 10px;border-radius:var(--radius-md);margin-bottom:3px;"
     onmouseenter="this.querySelectorAll('.qa-hover').forEach(e=>e.style.opacity='1')"
     onmouseleave="this.querySelectorAll('.qa-hover').forEach(e=>e.style.opacity='0')">
  <span style="width:7px;height:7px;border-radius:50%;background:{dot};flex-shrink:0;"></span>
  <div style="flex:1;min-width:0;cursor:pointer;" onclick="qaLoadPipeline('{id}','{name_js}','{tlabel}')">
    <p style="font-size:13px;font-weight:500;color:var(--text-primary);margin:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{name}</p>
    <p style="font-size:11px;color:var(--text-muted);margin:1px 0 0;">{tlabel}</p>
  </div>
  {run_btn}
  <select name="folder_id" class="qa-hover"
          style="font-size:10px;padding:2px 3px;border-radius:4px;border:1px solid var(--border);background:var(--bg-base);color:var(--text-muted);opacity:0;transition:opacity 150ms;max-width:72px;"
          title="Move to folder"
          hx-put="/api/pipelines/{id}/folder"
          hx-target="#pipeline-list"
          hx-swap="outerHTML"
          hx-trigger="change">
    {folder_opts}
  </select>
  <button class="qa-hover btn btn-ghost btn-sm"
          style="padding:3px;opacity:0;transition:opacity 150ms;color:var(--destructive);"
          hx-delete="/api/pipelines/{id}" hx-target="#pipeline-list" hx-swap="outerHTML"
          hx-confirm="Delete pipeline «{name}»?" title="Delete">✕</button>
</li>"##,
        dot = dot_color,
        id = p.id,
        name = name,
        name_js = name_js,
        tlabel = tlabel,
        run_btn = run_btn,
        folder_opts = folder_opts
    )
}

fn render_pipeline_list(pipelines: &[PipelineRow], folders: &[FolderRow]) -> String {
    // Group pipelines by folder
    let mut by_folder: std::collections::HashMap<String, Vec<&PipelineRow>> = std::collections::HashMap::new();
    let mut uncategorised: Vec<&PipelineRow> = Vec::new();
    for p in pipelines {
        match &p.folder_id {
            Some(fid) => by_folder.entry(fid.clone()).or_default().push(p),
            None => uncategorised.push(p),
        }
    }

    let mut html = String::from(r#"<ul id="pipeline-list" style="list-style:none;margin:0;padding:0;">"#);

    // Render folders (collapsible via <details>/<summary>)
    for f in folders {
        let fps = by_folder.get(&f.id).map(|v| v.as_slice()).unwrap_or(&[]);
        let items: String = fps.iter().map(|p| render_pipeline_item(p, folders)).collect();
        html.push_str(&format!(
            r##"<li style="margin-bottom:2px;">
  <details open style="margin:0;">
    <summary style="display:flex;align-items:center;gap:5px;padding:6px 10px;cursor:pointer;list-style:none;border-radius:var(--radius-md);"
             onmouseenter="this.style.background='var(--bg-elevated)'" onmouseleave="this.style.background='transparent'">
      <i data-lucide="folder" style="width:12px;height:12px;color:var(--text-muted);flex-shrink:0;"></i>
      <span style="flex:1;font-size:12px;font-weight:600;color:var(--text-secondary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{fname}</span>
      <span style="font-size:10px;color:var(--text-muted);margin-right:4px;">{count}</span>
      <button class="btn btn-ghost btn-sm" style="padding:2px 4px;font-size:10px;color:var(--text-muted);"
              hx-delete="/api/pipeline-folders/{fid}"
              hx-target="#pipeline-list" hx-swap="outerHTML"
              hx-confirm="Delete folder «{fname_esc}»? Pipelines will become uncategorised."
              title="Delete folder" onclick="event.stopPropagation()">✕</button>
    </summary>
    <ul style="list-style:none;margin:0;padding:0 0 0 10px;">{items}</ul>
  </details>
</li>"##,
            fid = f.id,
            fname = html_escape(&f.name),
            fname_esc = html_escape(&f.name),
            count = fps.len(),
            items = items
        ));
    }

    // Uncategorised pipelines
    for p in &uncategorised {
        html.push_str(&render_pipeline_item(p, folders));
    }

    if folders.is_empty() && uncategorised.is_empty() {
        html.push_str(r#"<li style="font-size:12px;color:var(--text-muted);padding:8px 4px;">No pipelines yet — pick a template or create one above.</li>"#);
    }

    html.push_str("</ul>");
    html
}

async fn load_and_render_list(state: &Arc<AppState>) -> String {
    let folders: Vec<FolderRow> = sqlx::query_as(
        "SELECT id, name, sort_order FROM pipeline_folders ORDER BY sort_order ASC, name ASC"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let pipelines: Vec<PipelineRow> = sqlx::query_as(
        "SELECT id, name, trigger, enabled, created_at, folder_id FROM pipelines ORDER BY created_at ASC"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    render_pipeline_list(&pipelines, &folders)
}

fn render_steps(pipeline_id: &str, steps: &[StepRow]) -> String {
    if steps.is_empty() {
        return format!(
            r#"<div id="steps-{pid}" style="font-size:12px;color:var(--text-muted);font-style:italic;padding:8px 0;">No steps yet. Add one below.</div>"#,
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
                r#"<p style="font-size:11px;color:var(--text-muted);margin:2px 0 0;">{}</p>"#,
                summary
            )
        };

        let up_disabled = if i == 0 { "disabled" } else { "" };
        let down_disabled = if i == steps.len() - 1 { "disabled" } else { "" };

        parts.push(format!(
            r##"<div class="card" style="display:flex;align-items:flex-start;gap:10px;padding:10px 12px;">
  <span style="display:flex;align-items:center;flex-shrink:0;margin-top:2px;color:var(--text-muted);">{icon}</span>
  <div style="flex:1;min-width:0;">
    <p style="font-size:13px;font-weight:500;color:var(--text-primary);margin:0;">{label}</p>
    {summary_html}
  </div>
  <div style="display:flex;flex-direction:column;gap:2px;flex-shrink:0;">
    <button class="btn btn-ghost btn-sm {up_disabled}"
            hx-post="/api/pipelines/{pid}/steps/{sid}/move"
            hx-vals='{{"direction":"up"}}'
            hx-target="#steps-{pid}"
            hx-swap="outerHTML">▲</button>
    <button class="btn btn-ghost btn-sm {down_disabled}"
            hx-post="/api/pipelines/{pid}/steps/{sid}/move"
            hx-vals='{{"direction":"down"}}'
            hx-target="#steps-{pid}"
            hx-swap="outerHTML">▼</button>
  </div>
  <button class="btn btn-danger btn-sm"
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
            parts.push(r#"<div style="text-align:center;color:var(--border);font-size:12px;user-select:none;">↓</div>"#.to_string());
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
        (r#"{"type":"Manual"}"#, "Manual"),
        (r#"{"type":"OcrCompleted"}"#, "After OCR"),
        (r#"{"type":"TranscriptionCompleted"}"#, "After Voice"),
        (r#"{"type":"ClipboardChanged"}"#, "On Clipboard Change"),
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
            r##"<div style="border-top:1px solid var(--border);padding-top:16px;"
                  x-data="{{showRun: false, runText: ''}}">
  <button class="btn btn-secondary btn-sm"
          @click="showRun = !showRun">
    <i data-lucide="play" style="width:12px;height:12px;"></i> Run Now
  </button>
  <div x-show="showRun" style="margin-top:8px;display:flex;gap:8px;">
    <input x-model="runText"
           type="text"
           placeholder="Initial text (optional)"
           class="input flex-1"
           style="font-size:12px;padding:5px 8px;" />
    <button class="btn btn-primary btn-sm"
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
             class="input flex-1"
             style="width:auto;font-size:13px;padding:6px 10px;"
             pattern="[^/\\&lt;&gt;&quot;]+"
             title="Cannot contain / \ &lt; &gt; or &quot;"
             required />
      <label style="display:flex;align-items:center;gap:6px;font-size:12px;color:var(--text-muted);cursor:pointer;white-space:nowrap;">
        <input type="checkbox" name="enabled" value="1" {enabled_checked} />
        Enabled
      </label>
      <button type="submit" class="btn btn-primary btn-sm">Save</button>
    </div>
    <div style="display:flex;align-items:center;gap:8px;">
      <span style="font-size:12px;color:var(--text-muted);white-space:nowrap;">Trigger:</span>
      <select name="trigger" class="input" style="width:auto;font-size:12px;padding:5px 8px;">
        {trigger_opts}
      </select>
    </div>
  </form>

  <!-- Steps -->
  <div>
    <p style="font-size:11px;font-weight:600;letter-spacing:0.08em;text-transform:uppercase;color:var(--text-muted);margin-bottom:8px;">Steps</p>
    {steps_html}
  </div>

  <!-- Add step -->
  <div x-data="{{
    tool: 'translate',
    from_lang: 'auto',
    to_lang: 'en',
    note_title: ''
  }}" class="card flex flex-col gap-3">
    <p style="font-size:11px;font-weight:600;letter-spacing:0.08em;text-transform:uppercase;color:var(--text-muted);margin:0;">Add step</p>

    <select x-model="tool" class="input" style="font-size:12px;padding:5px 8px;">
      <option value="translate">Translate</option>
      <option value="copy_clipboard">Copy to Clipboard</option>
      <option value="save_note">Save as Note</option>
    </select>

    <div x-show="tool === 'translate'" class="flex gap-2">
      <div class="flex-1">
        <label style="font-size:11px;color:var(--text-muted);display:block;margin-bottom:4px;">From</label>
        <select x-model="from_lang" class="input" style="font-size:12px;padding:5px 8px;">
          {lang_opts}
        </select>
      </div>
      <div class="flex-1">
        <label style="font-size:11px;color:var(--text-muted);display:block;margin-bottom:4px;">To</label>
        <select x-model="to_lang" class="input" style="font-size:12px;padding:5px 8px;">
          {lang_opts}
        </select>
      </div>
    </div>

    <div x-show="tool === 'save_note'">
      <label style="font-size:11px;color:var(--text-muted);display:block;margin-bottom:4px;">Note title (blank = first line of text)</label>
      <input x-model="note_title" type="text" placeholder="Optional title" class="input" style="font-size:12px;padding:5px 8px;" />
    </div>

    <button class="btn btn-secondary btn-sm" style="align-self:flex-start;"
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

// ── Pipeline templates ────────────────────────────────────────────────────────

struct Template {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    trigger: &'static str,
    steps: &'static [(&'static str, &'static str)], // (tool, config_json)
}

const TEMPLATES: &[Template] = &[
    Template {
        id: "ocr-copy",
        name: "OCR → Copy",
        description: "After OCR, copy the extracted text to clipboard automatically.",
        trigger: r#"{"type":"OcrCompleted"}"#,
        steps: &[("copy_clipboard", "{}")],
    },
    Template {
        id: "ocr-translate-copy",
        name: "OCR → Translate → Copy",
        description: "After OCR, translate the text to English and copy it.",
        trigger: r#"{"type":"OcrCompleted"}"#,
        steps: &[
            ("translate", r#"{"from_lang":"auto","to_lang":"en"}"#),
            ("copy_clipboard", "{}"),
        ],
    },
    Template {
        id: "voice-note",
        name: "Voice → Save as Note",
        description: "After transcription, save the transcript as a note.",
        trigger: r#"{"type":"TranscriptionCompleted"}"#,
        steps: &[("save_note", "{}")],
    },
    Template {
        id: "voice-translate-copy",
        name: "Voice → Translate → Copy",
        description: "After transcription, translate to English and copy.",
        trigger: r#"{"type":"TranscriptionCompleted"}"#,
        steps: &[
            ("translate", r#"{"from_lang":"auto","to_lang":"en"}"#),
            ("copy_clipboard", "{}"),
        ],
    },
    Template {
        id: "clipboard-translate-copy",
        name: "Clipboard → Translate → Copy",
        description: "On clipboard change, translate to English and re-copy.",
        trigger: r#"{"type":"ClipboardChanged"}"#,
        steps: &[
            ("translate", r#"{"from_lang":"auto","to_lang":"en"}"#),
            ("copy_clipboard", "{}"),
        ],
    },
];

fn render_templates() -> String {
    let cards: String = TEMPLATES
        .iter()
        .map(|t| {
            let flow: String = std::iter::once(trigger_label(t.trigger))
                .chain(t.steps.iter().map(|(tool, _)| tool_label(tool)))
                .collect::<Vec<_>>()
                .join(" → ");

            format!(
                r##"<div class="card" style="display:flex;flex-direction:column;gap:8px;padding:14px;">
  <p style="font-size:13px;font-weight:500;color:var(--text-primary);margin:0;">{name}</p>
  <p style="font-size:12px;color:var(--text-muted);margin:0;line-height:1.4;">{desc}</p>
  <p style="font-size:11px;color:var(--text-muted);margin:0;font-family:monospace;">{flow}</p>
  <button class="btn btn-secondary btn-sm" style="align-self:flex-start;margin-top:2px;"
          hx-post="/api/pipeline-templates/use"
          hx-vals='{{"template_id":"{id}"}}'
          hx-target="#pipeline-list"
          hx-swap="outerHTML">
    Use this
  </button>
</div>"##,
                name = html_escape(t.name),
                desc = html_escape(t.description),
                flow = html_escape(&flow),
                id = t.id,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<div id="qa-editor">
  <div style="margin-bottom:16px;">
    <p style="font-size:11px;font-weight:600;letter-spacing:0.08em;text-transform:uppercase;color:var(--text-muted);margin:0 0 4px;">Templates</p>
    <p style="font-size:12px;color:var(--text-muted);margin:0;">Start from a pre-built pipeline, or create one from scratch on the left.</p>
  </div>
  <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(240px,1fr));gap:10px;">
    {cards}
  </div>
</div>"#,
        cards = cards
    )
}

async fn templates_handler() -> impl IntoResponse {
    Html(render_templates())
}

#[derive(Deserialize)]
struct UseTemplateParams {
    template_id: String,
}

async fn use_template_handler(
    State(state): State<Arc<AppState>>,
    Form(params): Form<UseTemplateParams>,
) -> impl IntoResponse {
    use axum::http::{HeaderMap, HeaderName, HeaderValue};

    let Some(tmpl) = TEMPLATES.iter().find(|t| t.id == params.template_id) else {
        return (
            HeaderMap::new(),
            Html(format!(
                r#"<ul id="pipeline-list"><li style="font-size:12px;color:var(--destructive);padding:8px;">Unknown template: {}</li></ul>"#,
                html_escape(&params.template_id)
            )),
        );
    };

    let pipeline_id = uuid::Uuid::new_v4().to_string();
    let now = now_secs();
    let _ = sqlx::query(
        "INSERT INTO pipelines (id, name, trigger, enabled, created_at) VALUES (?, ?, ?, 1, ?)",
    )
    .bind(&pipeline_id)
    .bind(tmpl.name)
    .bind(tmpl.trigger)
    .bind(now)
    .execute(&state.db)
    .await;

    for (order, (tool, config)) in tmpl.steps.iter().enumerate() {
        let step_id = uuid::Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO pipeline_steps (id, pipeline_id, step_order, tool, config) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&step_id)
        .bind(&pipeline_id)
        .bind((order + 1) as i64)
        .bind(*tool)
        .bind(*config)
        .execute(&state.db)
        .await;
    }

    // Updated list (main swap target)
    let list_html = load_and_render_list(&state).await;

    // Kept for legacy compatibility; graph-based pipelines no longer use steps
    let _steps: Vec<StepRow> = sqlx::query_as(
        "SELECT id, pipeline_id, step_order, tool, config FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC",
    )
    .bind(&pipeline_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Emit HX-Trigger to open the canvas for the new pipeline
    let tlabel = trigger_label(tmpl.trigger);
    let name_js = tmpl.name.replace('\'', "\\'");
    let hx_trigger = format!(
        r#"{{"qa:load-pipeline":{{"id":"{pid}","name":"{name}","triggerLabel":"{tl}"}}}}"#,
        pid  = pipeline_id,
        name = name_js,
        tl   = tlabel,
    );

    let mut headers = HeaderMap::new();
    if let Ok(val) = HeaderValue::from_str(&hx_trigger) {
        headers.insert(HeaderName::from_static("hx-trigger"), val);
    }
    (headers, Html(list_html))
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
struct CreateFolderParams {
    name: String,
}

#[derive(Deserialize)]
struct MoveFolderParams {
    folder_id: Option<String>,
}

#[derive(Deserialize)]
struct ImportParams {
    yaml: String,
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

// ── Name validation ───────────────────────────────────────────────────────────

/// Trims whitespace and rejects names containing filesystem-unsafe or HTML-unsafe
/// characters: `/  \  <  >  "  \0`. Returns the trimmed name or an HTML error fragment.
fn validate_name(raw: &str) -> Result<String, Html<String>> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(Html(
            r#"<p style="font-size:12px;color:var(--destructive);">Name cannot be empty.</p>"#
                .to_string(),
        ));
    }
    if name.chars().any(|c| matches!(c, '/' | '\\' | '<' | '>' | '"' | '\0')) {
        return Err(Html(
            r#"<p style="font-size:12px;color:var(--destructive);">Name cannot contain / \ &lt; &gt; " or null bytes.</p>"#
                .to_string(),
        ));
    }
    Ok(name.to_string())
}

// ── CRUD handlers ─────────────────────────────────────────────────────────────

async fn list_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Html(load_and_render_list(&state).await)
}

async fn create_handler(
    State(state): State<Arc<AppState>>,
    Form(params): Form<CreateParams>,
) -> impl IntoResponse {
    let name = match validate_name(&params.name) {
        Ok(n) => n,
        Err(e) => return e,
    };
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
    Html(load_and_render_list(&state).await)
}

async fn update_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Form(params): Form<UpdateParams>,
) -> impl IntoResponse {
    let name = match validate_name(&params.name) {
        Ok(n) => n,
        Err(e) => return e,
    };
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
    Html(load_and_render_list(&state).await)
}

async fn delete_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM pipelines WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;
    Html(load_and_render_list(&state).await)
}

// ── Folder handlers ───────────────────────────────────────────────────────────

async fn create_folder_handler(
    State(state): State<Arc<AppState>>,
    Form(params): Form<CreateFolderParams>,
) -> impl IntoResponse {
    let name = match validate_name(&params.name) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let id = uuid::Uuid::new_v4().to_string();
    let _ = sqlx::query(
        "INSERT INTO pipeline_folders (id, name, sort_order) VALUES (?, ?, (SELECT COALESCE(MAX(sort_order),0)+1 FROM pipeline_folders))",
    )
    .bind(&id)
    .bind(&name)
    .execute(&state.db)
    .await;
    Html(load_and_render_list(&state).await)
}

async fn delete_folder_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Detach pipelines from folder first
    let _ = sqlx::query("UPDATE pipelines SET folder_id = NULL WHERE folder_id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;
    let _ = sqlx::query("DELETE FROM pipeline_folders WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;
    Html(load_and_render_list(&state).await)
}

async fn move_to_folder_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Form(params): Form<MoveFolderParams>,
) -> impl IntoResponse {
    let folder_id: Option<String> = params.folder_id.filter(|s| !s.is_empty());
    let _ = sqlx::query("UPDATE pipelines SET folder_id = ? WHERE id = ?")
        .bind(folder_id)
        .bind(&id)
        .execute(&state.db)
        .await;
    Html(load_and_render_list(&state).await)
}

// ── Export / Import handlers ──────────────────────────────────────────────────

async fn export_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    let pipeline: Option<PipelineRow> = sqlx::query_as(
        "SELECT id, name, trigger, enabled, created_at, folder_id FROM pipelines WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some(pipeline) = pipeline else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Pipeline not found"))
            .unwrap();
    };

    let nodes: Vec<NodeRow> = sqlx::query_as(
        "SELECT id, pipeline_id, node_type, config, pos_x, pos_y FROM pipeline_nodes WHERE pipeline_id = ? ORDER BY pos_x ASC",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let edges: Vec<EdgeRow> = sqlx::query_as(
        "SELECT id, pipeline_id, source_id, target_id, edge_label FROM pipeline_edges WHERE pipeline_id = ?",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Extract trigger type string for the top-level field
    let trigger_type = serde_json::from_str::<serde_json::Value>(&pipeline.trigger)
        .ok()
        .and_then(|v| v.get("type").and_then(|t| t.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| "Manual".to_string());

    let yaml_nodes: Vec<NodeYaml> = nodes
        .iter()
        .map(|n| NodeYaml {
            id: n.id.clone(),
            node_type: n.node_type.clone(),
            config: serde_json::from_str(&n.config).unwrap_or(serde_json::Value::Object(Default::default())),
            pos_x: n.pos_x,
            pos_y: n.pos_y,
        })
        .collect();

    let yaml_edges: Vec<EdgeYaml> = edges
        .iter()
        .map(|e| EdgeYaml {
            id: e.id.clone(),
            source: e.source_id.clone(),
            target: e.target_id.clone(),
            label: e.edge_label.clone(),
        })
        .collect();

    let pipeline_yaml = PipelineYaml {
        name: pipeline.name.clone(),
        trigger: trigger_type,
        nodes: yaml_nodes,
        edges: yaml_edges,
    };

    let yaml_str = match serde_yaml::to_string(&pipeline_yaml) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Serialization error: {}", e)))
                .unwrap()
        }
    };

    let safe_name: String = pipeline
        .name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let filename = format!("{}.yaml", safe_name);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-yaml; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(yaml_str))
        .unwrap()
}

async fn import_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ImportParams>,
) -> impl IntoResponse {
    let parsed: PipelineYaml = match serde_yaml::from_str(&body.yaml) {
        Ok(p) => p,
        Err(e) => {
            return Json(serde_json::json!({ "error": format!("Invalid YAML: {}", e) }));
        }
    };

    let pipeline_id = uuid::Uuid::new_v4().to_string();
    let trigger_json = serde_json::json!({ "type": parsed.trigger }).to_string();
    let now = now_secs();

    let _ = sqlx::query(
        "INSERT INTO pipelines (id, name, trigger, enabled, created_at) VALUES (?, ?, ?, 1, ?)",
    )
    .bind(&pipeline_id)
    .bind(&parsed.name)
    .bind(&trigger_json)
    .bind(now)
    .execute(&state.db)
    .await;

    // Insert nodes — generate new UUIDs but store YAML id as a slug map for edge remapping
    let mut id_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for node in &parsed.nodes {
        let new_id = uuid::Uuid::new_v4().to_string();
        id_map.insert(node.id.clone(), new_id.clone());
        let config_str = serde_json::to_string(&node.config).unwrap_or_else(|_| "{}".to_string());
        let _ = sqlx::query(
            "INSERT INTO pipeline_nodes (id, pipeline_id, node_type, config, pos_x, pos_y) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&new_id)
        .bind(&pipeline_id)
        .bind(&node.node_type)
        .bind(&config_str)
        .bind(node.pos_x)
        .bind(node.pos_y)
        .execute(&state.db)
        .await;
    }

    // Insert edges — remap source/target through id_map
    for edge in &parsed.edges {
        let Some(src) = id_map.get(&edge.source) else { continue; };
        let Some(tgt) = id_map.get(&edge.target) else { continue; };
        let edge_id = uuid::Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO pipeline_edges (id, pipeline_id, source_id, target_id, edge_label) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&edge_id)
        .bind(&pipeline_id)
        .bind(src)
        .bind(tgt)
        .bind(&edge.label)
        .execute(&state.db)
        .await;
    }

    Json(serde_json::json!({ "id": pipeline_id, "name": parsed.name }))
}

async fn editor_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pipeline: Option<PipelineRow> = sqlx::query_as(
        "SELECT id, name, trigger, enabled, created_at, folder_id FROM pipelines WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some(pipeline) = pipeline else {
        return Html(r#"<p style="font-size:13px;color:var(--destructive);">Pipeline not found.</p>"#.to_string());
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
    match run_pipeline_graph(&id, initial, &state).await {
        Ok(final_val) => {
            let output = final_val
                .text
                .map(|t| {
                    format!(
                        r#"<p style="font-size:13px;color:var(--success);">✓ Done: <span style="font-family:monospace;">{}</span></p>"#,
                        html_escape(&t.chars().take(120).collect::<String>())
                    )
                })
                .unwrap_or_else(|| {
                    r#"<p style="font-size:13px;color:var(--success);">✓ Pipeline completed.</p>"#.to_string()
                });
            Html(output)
        }
        Err(e) => Html(format!(
            r#"<p style="font-size:13px;color:var(--destructive);">✗ {}</p>"#,
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
            // Suppress clipboard monitor from re-recording this copy (D-051: send text directly).
            let _ = state.clipboard_suppress_tx.send(text.clone());
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

// ── Graph execution engine (H4) ──────────────────────────────────────────────
//
// Traversal is iterative (loop, not recursion) except for `for_each_file` which
// fans out into sub-traversals via a boxed recursive call to `run_graph_from`.
//
// Cycle / timeout protection:
//   60 s elapsed  → log warning (once per run)
//   120 s elapsed → return error to caller
//
// Backward compatibility:
//   Pipelines that have no graph nodes fall back to the old `run_pipeline_steps`.

const GRAPH_WARN_SECS: u64 = 60;
const GRAPH_KILL_SECS: u64 = 120;
const GRAPH_MAX_FILES: usize = 10_000;

struct GraphCtx {
    nodes: std::collections::HashMap<String, NodeRow>,
    /// source_id → outgoing edges
    adj: std::collections::HashMap<String, Vec<EdgeRow>>,
    started_at: std::time::Instant,
    warned: std::sync::atomic::AtomicBool,
}

impl GraphCtx {
    fn build(nodes: Vec<NodeRow>, edges: Vec<EdgeRow>) -> Self {
        let node_map = nodes.into_iter().map(|n| (n.id.clone(), n)).collect();
        let mut adj: std::collections::HashMap<String, Vec<EdgeRow>> =
            std::collections::HashMap::new();
        for e in edges {
            adj.entry(e.source_id.clone()).or_default().push(e);
        }
        Self {
            nodes: node_map,
            adj,
            started_at: std::time::Instant::now(),
            warned: std::sync::atomic::AtomicBool::new(false),
        }
    }

    fn guard(&self, name: &str) -> Result<(), String> {
        let s = self.started_at.elapsed().as_secs();
        if s >= GRAPH_KILL_SECS {
            return Err(format!(
                "Loop timed out after {s}s — break the back-edge cycle or raise the timeout in Settings"
            ));
        }
        if s >= GRAPH_WARN_SECS {
            use std::sync::atomic::Ordering;
            if !self.warned.swap(true, Ordering::Relaxed) {
                log::warn!(
                    "Quick Actions: pipeline '{name}' has been running for {s}s \
                     — possible infinite loop via back-edge"
                );
            }
        }
        Ok(())
    }
}

async fn run_pipeline_graph(
    pipeline_id: &str,
    initial: StepValue,
    state: &std::sync::Arc<AppState>,
) -> Result<StepValue, String> {
    let nodes: Vec<NodeRow> = sqlx::query_as(
        "SELECT id, pipeline_id, node_type, config, pos_x, pos_y \
         FROM pipeline_nodes WHERE pipeline_id = ?",
    )
    .bind(pipeline_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if nodes.is_empty() {
        // No graph yet — fall back to old linear steps runner for compatibility.
        return run_pipeline_steps(pipeline_id, initial, state).await;
    }

    let edges: Vec<EdgeRow> = sqlx::query_as(
        "SELECT id, pipeline_id, source_id, target_id, edge_label \
         FROM pipeline_edges WHERE pipeline_id = ?",
    )
    .bind(pipeline_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let trigger_id = nodes
        .iter()
        .find(|n| n.node_type == "trigger")
        .ok_or("No trigger node — add a Trigger to this pipeline")?
        .id
        .clone();

    let ctx = std::sync::Arc::new(GraphCtx::build(nodes, edges));
    run_graph_from(
        trigger_id,
        initial,
        ctx,
        pipeline_id.to_string(),
        state.clone(),
    )
    .await
}

/// Iterative graph traversal.
/// Returned as a boxed future so `execute_action` can call it recursively
/// for `for_each_file` fan-out without triggering Rust's infinite-type error.
fn run_graph_from(
    start_id: String,
    initial: StepValue,
    ctx: std::sync::Arc<GraphCtx>,
    pipeline_name: String,
    state: std::sync::Arc<AppState>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<StepValue, String>> + Send>> {
    Box::pin(async move {
        let mut current_id = start_id;
        let mut value = initial;
        loop {
            ctx.guard(&pipeline_name)?;

            let node = match ctx.nodes.get(&current_id) {
                Some(n) => n.clone(),
                None => break, // dangling edge — stop gracefully
            };

            match node.node_type.as_str() {
                "trigger" => { /* passthrough — initial value continues */ }
                "end" => break,
                "action" => {
                    let (new_val, signal) =
                        execute_action(&node, value, ctx.clone(), &pipeline_name, state.clone())
                            .await?;
                    value = new_val;
                    if signal == "__done__" {
                        // for_each_file already traversed the rest of the graph; stop here.
                        return Ok(value);
                    }
                }
                "condition" => {
                    let branch = evaluate_condition(&node, &value)?;
                    let edges = ctx.adj.get(&current_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let next_id = edges
                        .iter()
                        .find(|e| e.edge_label == branch)
                        .or_else(|| edges.first())
                        .map(|e| e.target_id.clone());
                    match next_id {
                        Some(nid) => { current_id = nid; continue; }
                        None => break,
                    }
                }
                _ => { /* unknown node type — skip */ }
            }

            // Advance along default outgoing edge.
            let edges = ctx.adj.get(&current_id).map(|v| v.as_slice()).unwrap_or(&[]);
            let next_id = edges
                .iter()
                .find(|e| e.edge_label == "default" || e.edge_label.is_empty())
                .or_else(|| edges.first())
                .map(|e| e.target_id.clone());
            match next_id {
                Some(nid) => current_id = nid,
                None => break,
            }
        }
        Ok(value)
    })
}

/// Execute a single action node.
/// Returns `(new_value, signal)` where signal == "__done__" means the caller
/// should stop normal traversal (used by for_each_file which consumes the rest
/// of the graph internally).
async fn execute_action(
    node: &NodeRow,
    input: StepValue,
    ctx: std::sync::Arc<GraphCtx>,
    pipeline_name: &str,
    state: std::sync::Arc<AppState>,
) -> Result<(StepValue, String), String> {
    let config: serde_json::Value =
        serde_json::from_str(&node.config).unwrap_or_else(|_| serde_json::json!({}));
    let tool = config["tool"].as_str().unwrap_or("").to_string();
    let params = &config["params"];

    let result: StepValue = match tool.as_str() {
        // ── Text tools ─────────────────────────────────────────────────────
        "translate" => {
            let from = params["from_lang"].as_str().unwrap_or("auto").to_string();
            let to = params["to_lang"].as_str().unwrap_or("en").to_string();
            let text = input
                .text
                .clone()
                .ok_or_else(|| "translate: no text input".to_string())?;
            let translated = run_translate_raw(&text, &from, &to).await?;
            StepValue { text: Some(translated) }
        }

        "copy_clipboard" => {
            let text = input
                .text
                .clone()
                .ok_or_else(|| "copy_clipboard: no text input".to_string())?;
            // Suppress clipboard monitor from re-recording this copy (D-051: send text directly).
            let _ = state.clipboard_suppress_tx.send(text.clone());
            tokio::task::spawn_blocking(move || {
                arboard::Clipboard::new()
                    .and_then(|mut c| c.set_text(text))
                    .map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| e.to_string())??;
            input
        }

        "save_note" => {
            let prefix = params["title_prefix"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("Quick Action Note")
                .to_string();
            let text = input
                .text
                .clone()
                .ok_or_else(|| "save_note: no text input".to_string())?;
            let first_line: String = text.lines().next().unwrap_or("").chars().take(60).collect();
            let title = if first_line.is_empty() {
                prefix
            } else {
                format!("{prefix}: {first_line}")
            };
            let id = uuid::Uuid::new_v4().to_string();
            let now = now_secs();
            sqlx::query(
                "INSERT INTO notes (id, title, content, created_at, updated_at) VALUES (?,?,?,?,?)",
            )
            .bind(&id)
            .bind(&title)
            .bind(&text)
            .bind(now)
            .bind(now)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
            state.event_bus.publish(Event::NoteCreated { id, title });
            input
        }

        // ── File tools ─────────────────────────────────────────────────────
        "read_file" => {
            let path = params["file_path"]
                .as_str()
                .filter(|s| !s.is_empty())
                .or(input.text.as_deref())
                .ok_or_else(|| "read_file: no file path".to_string())?
                .to_string();
            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(|e| format!("read_file '{path}': {e}"))?;
            StepValue { text: Some(content) }
        }

        "write_file" => {
            let path = params["file_path"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "write_file: no file path".to_string())?
                .to_string();
            let content = input.text.clone().unwrap_or_default();
            match params["if_exists"].as_str().unwrap_or("overwrite") {
                "append" => {
                    let existing = tokio::fs::read_to_string(&path).await.unwrap_or_default();
                    let joined = if existing.is_empty() {
                        content
                    } else {
                        format!("{existing}\n{content}")
                    };
                    tokio::fs::write(&path, joined)
                        .await
                        .map_err(|e| format!("write_file '{path}': {e}"))?;
                }
                "skip" => {
                    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
                        tokio::fs::write(&path, &content)
                            .await
                            .map_err(|e| format!("write_file '{path}': {e}"))?;
                    }
                }
                _ => {
                    tokio::fs::write(&path, &content)
                        .await
                        .map_err(|e| format!("write_file '{path}': {e}"))?;
                }
            }
            input
        }

        "append_file" => {
            let path = params["file_path"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "append_file: no file path".to_string())?
                .to_string();
            let content = input.text.clone().unwrap_or_default();
            let sep = params["separator"]
                .as_str()
                .unwrap_or(r"\n")
                .replace(r"\n", "\n")
                .replace(r"\t", "\t");
            let existing = tokio::fs::read_to_string(&path).await.unwrap_or_default();
            let joined = if existing.is_empty() {
                content
            } else {
                format!("{existing}{sep}{content}")
            };
            tokio::fs::write(&path, joined)
                .await
                .map_err(|e| format!("append_file '{path}': {e}"))?;
            input
        }

        // ── Media tools ────────────────────────────────────────────────────
        "ocr_file" => {
            let path = params["file_path"]
                .as_str()
                .filter(|s| !s.is_empty())
                .or(input.text.as_deref())
                .ok_or_else(|| "ocr_file: no file path".to_string())?
                .to_string();
            let lang = params["lang"].as_str().unwrap_or("eng").to_string();
            let out = tokio::process::Command::new("tesseract")
                .arg(&path)
                .arg("stdout")
                .arg("-l")
                .arg(&lang)
                .output()
                .await
                .map_err(|e| format!("ocr_file: tesseract: {e}"))?;
            if out.status.success() {
                StepValue {
                    text: Some(String::from_utf8_lossy(&out.stdout).trim().to_string()),
                }
            } else {
                return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
            }
        }

        // ── Loop / fan-out ─────────────────────────────────────────────────
        "for_each_file" => {
            let folder = params["folder_path"].as_str().unwrap_or("").to_string();
            let pattern = params["pattern"].as_str().unwrap_or("*").to_string();
            let recursive = params["recursive"].as_bool().unwrap_or(false);

            let files = list_files(&folder, &pattern, recursive)
                .await
                .map_err(|e| format!("for_each_file: {e}"))?;

            // First outgoing edge from this node leads to the body of the loop.
            let next_id = ctx
                .adj
                .get(&node.id)
                .and_then(|edges| {
                    edges
                        .iter()
                        .find(|e| e.edge_label == "default" || e.edge_label.is_empty())
                        .or_else(|| edges.first())
                })
                .map(|e| e.target_id.clone());

            let mut results: Vec<String> = Vec::new();
            if let Some(next_node_id) = next_id {
                for file_path in files.into_iter().take(GRAPH_MAX_FILES) {
                    ctx.guard(pipeline_name)?;
                    let fval = StepValue { text: Some(file_path.clone()) };
                    match run_graph_from(
                        next_node_id.clone(),
                        fval,
                        ctx.clone(),
                        pipeline_name.to_string(),
                        state.clone(),
                    )
                    .await
                    {
                        Ok(r) => {
                            if let Some(t) = r.text {
                                results.push(t);
                            }
                        }
                        Err(e) => log::warn!("for_each_file '{file_path}': {e}"),
                    }
                }
            }
            // Return __done__ — the sub-traversals already consumed the graph.
            return Ok((
                StepValue { text: Some(results.join("\n---\n")) },
                "__done__".to_string(),
            ));
        }

        other => return Err(format!("unknown tool: {other}")),
    };

    Ok((result, "default".to_string()))
}

fn evaluate_condition(node: &NodeRow, value: &StepValue) -> Result<String, String> {
    let config: serde_json::Value =
        serde_json::from_str(&node.config).unwrap_or_else(|_| serde_json::json!({}));
    let cond = config["condition"].as_str().unwrap_or("always_true");
    let cval = config["value"].as_str().unwrap_or("");
    let text = value.text.as_deref().unwrap_or("");

    let passes = match cond {
        "always_true" => true,
        "not_empty" => !text.trim().is_empty(),
        "contains" => text.contains(cval),
        "length_gt" => text.len() > cval.parse::<usize>().unwrap_or(0),
        // matches_regex: regex crate not in deps — fall back to substring match
        "matches_regex" => text.contains(cval),
        _ => true,
    };

    Ok(if passes { "true".to_string() } else { "false".to_string() })
}

/// List files in `folder` whose names match `pattern` (`*`, `*.ext`, or exact name).
async fn list_files(
    folder: &str,
    pattern: &str,
    recursive: bool,
) -> Result<Vec<String>, String> {
    let mut results: Vec<String> = Vec::new();
    let mut dirs = vec![std::path::PathBuf::from(folder)];
    while let Some(dir) = dirs.pop() {
        let mut rd = tokio::fs::read_dir(&dir)
            .await
            .map_err(|e| format!("cannot read '{}': {e}", dir.display()))?;
        while let Some(entry) = rd.next_entry().await.map_err(|e| e.to_string())? {
            let ft = entry.file_type().await.map_err(|e| e.to_string())?;
            if ft.is_dir() {
                if recursive {
                    dirs.push(entry.path());
                }
            } else if let Some(name) = entry.file_name().to_str().map(str::to_owned) {
                if file_matches_pattern(&name, pattern) {
                    results.push(entry.path().to_string_lossy().into_owned());
                }
            }
        }
    }
    results.sort();
    Ok(results)
}

fn file_matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern.is_empty() {
        return true;
    }
    if let Some(ext) = pattern.strip_prefix("*.") {
        return name.ends_with(&format!(".{ext}"));
    }
    name == pattern
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
                if let Err(e) = run_pipeline_graph(&pipeline.id, val, &state).await {
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

// ── Graph DB row types ────────────────────────────────────────────────────────

#[derive(sqlx::FromRow, serde::Serialize, Clone)]
struct NodeRow {
    id: String,
    pipeline_id: String,
    node_type: String,
    config: String,
    pos_x: f64,
    pos_y: f64,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct EdgeRow {
    id: String,
    pipeline_id: String,
    source_id: String,
    target_id: String,
    edge_label: String,
}

// ── Graph request structs ─────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct AddNodeParams {
    id: Option<String>, // caller may supply original ID (used by undo to restore deleted node)
    node_type: String,
    config: Option<String>,
    pos_x: Option<f64>,
    pos_y: Option<f64>,
}

#[derive(serde::Deserialize)]
struct UpdateNodeParams {
    config: Option<String>,
    pos_x: Option<f64>,
    pos_y: Option<f64>,
}

#[derive(serde::Deserialize)]
struct AddEdgeParams {
    id: Option<String>, // caller may supply original ID (used by undo to restore deleted edge)
    source_id: String,
    target_id: String,
    edge_label: Option<String>,
}

// ── Graph handlers ────────────────────────────────────────────────────────────

async fn graph_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let nodes: Vec<NodeRow> = sqlx::query_as(
        "SELECT id, pipeline_id, node_type, config, pos_x, pos_y \
         FROM pipeline_nodes WHERE pipeline_id = ? ORDER BY pos_x ASC",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let edges: Vec<EdgeRow> = sqlx::query_as(
        "SELECT id, pipeline_id, source_id, target_id, edge_label \
         FROM pipeline_edges WHERE pipeline_id = ?",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    axum::Json(serde_json::json!({ "nodes": nodes, "edges": edges }))
}

async fn add_node_handler(
    State(state): State<Arc<AppState>>,
    Path(pipeline_id): Path<String>,
    axum::Json(params): axum::Json<AddNodeParams>,
) -> impl IntoResponse {
    let id = params.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let config = params.config.unwrap_or_else(|| "{}".to_string());
    let pos_x = params.pos_x.unwrap_or(100.0);
    let pos_y = params.pos_y.unwrap_or(200.0);
    let _ = sqlx::query(
        "INSERT INTO pipeline_nodes (id, pipeline_id, node_type, config, pos_x, pos_y) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&pipeline_id)
    .bind(&params.node_type)
    .bind(&config)
    .bind(pos_x)
    .bind(pos_y)
    .execute(&state.db)
    .await;

    axum::Json(serde_json::json!({ "id": id }))
}

async fn update_node_handler(
    State(state): State<Arc<AppState>>,
    Path((pipeline_id, node_id)): Path<(String, String)>,
    axum::Json(params): axum::Json<UpdateNodeParams>,
) -> impl IntoResponse {
    if let Some(config) = &params.config {
        let _ = sqlx::query(
            "UPDATE pipeline_nodes SET config = ? WHERE id = ? AND pipeline_id = ?",
        )
        .bind(config)
        .bind(&node_id)
        .bind(&pipeline_id)
        .execute(&state.db)
        .await;
    }
    if let (Some(x), Some(y)) = (params.pos_x, params.pos_y) {
        let _ = sqlx::query(
            "UPDATE pipeline_nodes SET pos_x = ?, pos_y = ? WHERE id = ? AND pipeline_id = ?",
        )
        .bind(x)
        .bind(y)
        .bind(&node_id)
        .bind(&pipeline_id)
        .execute(&state.db)
        .await;
    }
    axum::Json(serde_json::json!({ "ok": true }))
}

async fn delete_node_handler(
    State(state): State<Arc<AppState>>,
    Path((pipeline_id, node_id)): Path<(String, String)>,
) -> impl IntoResponse {
    // Edges referencing this node are deleted via ON DELETE CASCADE on source_id/target_id.
    let _ = sqlx::query(
        "DELETE FROM pipeline_nodes WHERE id = ? AND pipeline_id = ?",
    )
    .bind(&node_id)
    .bind(&pipeline_id)
    .execute(&state.db)
    .await;
    axum::Json(serde_json::json!({ "ok": true }))
}

async fn add_edge_handler(
    State(state): State<Arc<AppState>>,
    Path(pipeline_id): Path<String>,
    axum::Json(params): axum::Json<AddEdgeParams>,
) -> impl IntoResponse {
    let id = params.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let label = params.edge_label.unwrap_or_else(|| "default".to_string());
    let _ = sqlx::query(
        "INSERT INTO pipeline_edges (id, pipeline_id, source_id, target_id, edge_label) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&pipeline_id)
    .bind(&params.source_id)
    .bind(&params.target_id)
    .bind(&label)
    .execute(&state.db)
    .await;
    axum::Json(serde_json::json!({ "id": id }))
}

async fn delete_edge_handler(
    State(state): State<Arc<AppState>>,
    Path((pipeline_id, edge_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let _ = sqlx::query(
        "DELETE FROM pipeline_edges WHERE id = ? AND pipeline_id = ?",
    )
    .bind(&edge_id)
    .bind(&pipeline_id)
    .execute(&state.db)
    .await;
    axum::Json(serde_json::json!({ "ok": true }))
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/pipeline-templates", get(templates_handler))
        .route("/api/pipeline-templates/use", post(use_template_handler))
        // Folders
        .route("/api/pipeline-folders", post(create_folder_handler))
        .route("/api/pipeline-folders/:id", delete(delete_folder_handler))
        // Import must be registered before /:id to avoid capture
        .route("/api/pipelines/import", post(import_handler))
        // Pipeline CRUD
        .route("/api/pipelines", get(list_handler).post(create_handler))
        .route(
            "/api/pipelines/:id",
            put(update_handler).delete(delete_handler),
        )
        .route("/api/pipelines/:id/editor", get(editor_handler))
        .route("/api/pipelines/:id/run", post(run_handler))
        .route("/api/pipelines/:id/folder", put(move_to_folder_handler))
        .route("/api/pipelines/:id/export", get(export_handler))
        .route("/api/pipelines/:id/steps", post(add_step_handler))
        .route(
            "/api/pipelines/:id/steps/:step_id",
            delete(delete_step_handler),
        )
        .route(
            "/api/pipelines/:id/steps/:step_id/move",
            post(move_step_handler),
        )
        // Graph API (H1+)
        .route("/api/pipelines/:id/graph", get(graph_handler))
        .route("/api/pipelines/:id/nodes", post(add_node_handler))
        .route(
            "/api/pipelines/:id/nodes/:node_id",
            put(update_node_handler).delete(delete_node_handler),
        )
        .route("/api/pipelines/:id/edges", post(add_edge_handler))
        .route(
            "/api/pipelines/:id/edges/:edge_id",
            delete(delete_edge_handler),
        )
}
