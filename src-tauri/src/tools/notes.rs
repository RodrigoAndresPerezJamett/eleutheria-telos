use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post, put},
    Router,
};
use sqlx::SqlitePool;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::event_bus::Event;
use crate::server::AppState;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// Max bytes stored in data-* attribute and shown in the card preview.
// Full content is fetched on-demand via GET /api/notes/:id when truncated.
const CONTENT_ATTR_MAX: usize = 2048;
const CONTENT_PREVIEW_MAX: usize = 300;

fn truncate_bytes(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn render_note_card(id: &str, title: &str, content: &str, pinned: i64, updated_at: i64) -> String {
    let clean_title = strip_inline_tags(title);
    let display_title = if clean_title.is_empty() { "Untitled".to_string() } else { clean_title };
    let pin_icon = if pinned == 1 { "📌 " } else { "" };
    let pin_btn_label = if pinned == 1 { "Unpin" } else { "Pin" };
    let ts = format_timestamp(updated_at);
    let truncated = content.len() > CONTENT_ATTR_MAX;
    let attr_content = truncate_bytes(content, CONTENT_ATTR_MAX);
    let preview_content = truncate_bytes(content, CONTENT_PREVIEW_MAX);
    let truncated_attr = if truncated { r#" data-note-truncated="true""# } else { "" };
    // padding-bottom:75% = 4:3 ratio (see clipboard.rs for full explanation).
    format!(
        r##"<div id="note-{id}"
     data-note-id="{id}"
     data-note-title="{escaped_title}"
     data-note-content="{escaped_content}"{truncated_attr}
     draggable="true"
     ondragstart="notesDragStart(event,'{id}')"
     ondragend="this.style.opacity='1'"
     style="background:var(--bg-elevated);border-radius:var(--radius-md);padding:14px 14px 12px;cursor:pointer;display:flex;flex-direction:column;overflow:hidden;outline:1px solid transparent;outline-offset:-1px;position:relative;user-select:none;height:225px;"
     onmouseenter="this.style.outlineColor='var(--accent)';this.querySelectorAll('.note-action').forEach(e=>e.style.display='inline-flex')"
     onmouseleave="this.style.outlineColor='transparent';this.querySelectorAll('.note-action').forEach(e=>e.style.display='none')"
     oncontextmenu="notesContextMenu(event,this)"
     onclick="notesOpenEditor(this)">
  <div style="display:flex;align-items:flex-start;gap:4px;margin-bottom:6px;flex-shrink:0;">
    <h3 style="flex:1;font-size:13px;font-weight:600;color:var(--text-primary);margin:0;overflow:hidden;max-height:2.6em;line-height:1.3;">{pin_icon}{escaped_title}</h3>
    <button class="note-action btn btn-ghost btn-sm"
            style="display:none;flex-shrink:0;padding:2px 5px;font-size:11px;"
            hx-post="/api/notes/{id}/pin"
            hx-target="#note-{id}"
            hx-swap="outerHTML"
            onclick="event.stopPropagation()"
            title="{pin_btn_label}">{pin_btn_label}</button>
  </div>
  <p style="font-size:12px;color:var(--text-muted);flex:1;margin:0 0 10px;overflow:hidden;line-height:1.5;">{escaped_preview}</p>
  <div style="display:flex;align-items:center;justify-content:space-between;flex-shrink:0;position:relative;z-index:1;">
    <span style="font-size:11px;color:var(--text-muted);">{ts}</span>
    <button class="note-action btn btn-ghost btn-sm"
            style="display:none;padding:2px 5px;font-size:11px;color:var(--destructive);"
            hx-delete="/api/notes/{id}"
            hx-target="#note-{id}"
            hx-swap="outerHTML"
            hx-confirm="Move to Trash?"
            onclick="event.stopPropagation()">✕</button>
  </div>
  <div style="position:absolute;bottom:0;left:0;right:0;height:38%;pointer-events:none;background:linear-gradient(to top, rgba(0,0,0,0.09) 0%, transparent 100%);border-radius:0 0 var(--radius-md) var(--radius-md);"></div>
</div>"##,
        id = id,
        pin_icon = pin_icon,
        pin_btn_label = pin_btn_label,
        escaped_title = html_escape(&display_title),
        escaped_content = html_escape(attr_content),
        truncated_attr = truncated_attr,
        escaped_preview = html_escape(preview_content),
        ts = ts,
    )
}

const PAGE: i64 = 24;

// ── Date bucketing ────────────────────────────────────────────────────────────

fn date_bucket(unix_ts: i64) -> &'static str {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let diff = now - unix_ts;
    if diff < 86_400 { "Today" }
    else if diff < 2 * 86_400 { "Yesterday" }
    else if diff < 7 * 86_400 { "This Week" }
    else if diff < 30 * 86_400 { "This Month" }
    else { "Older" }
}

fn bucket_separator(label: &str) -> String {
    format!(
        r#"<div style="grid-column:1/-1;padding:12px 2px 4px;font-size:10px;font-weight:700;color:var(--text-muted);letter-spacing:0.08em;text-transform:uppercase;border-bottom:1px solid var(--border);margin-bottom:4px;">{label}</div>"#
    )
}

fn render_note_list(entries: &[(String, String, String, i64, i64)], has_more: bool, next_offset: i64, show_buckets: bool) -> String {
    if entries.is_empty() {
        return r#"<div style="grid-column:1/-1;padding:48px 16px;text-align:center;">
  <p style="font-size:15px;font-weight:500;color:var(--text-primary);margin:0 0 8px;">Got something worth keeping?</p>
  <p style="font-size:13px;color:var(--text-muted);margin:0;line-height:1.6;">Write your first note — it stays local, searchable, and yours forever.</p>
</div>"#.to_string();
    }
    let mut html = String::new();
    let mut current_bucket = "";
    for (id, title, content, pinned, updated_at) in entries {
        if show_buckets {
            let b = date_bucket(*updated_at);
            if b != current_bucket {
                current_bucket = b;
                html.push_str(&bucket_separator(b));
            }
        }
        html.push_str(&render_note_card(id, title, content, *pinned, *updated_at));
    }
    if has_more {
        html.push_str(&format!(
            r#"<div id="notes-sentinel" style="grid-column:1/-1;padding:12px;text-align:center;color:var(--text-muted);font-size:12px;"
     hx-get="/api/notes?offset={next_offset}&limit={PAGE}"
     hx-trigger="intersect once"
     hx-swap="outerHTML">Loading more…</div>"#
        ));
    }
    html
}

// ── Trash card ────────────────────────────────────────────────────────────────

fn render_trash_note_card(id: &str, title: &str, content: &str, deleted_at: i64) -> String {
    let clean_title = strip_inline_tags(title);
    let display_title = if clean_title.is_empty() { "Untitled".to_string() } else { clean_title };
    let ts = format_timestamp(deleted_at);
    let preview = html_escape(truncate_bytes(content, CONTENT_PREVIEW_MAX));
    format!(
        r##"<div id="note-{id}"
     style="background:var(--bg-elevated);border-radius:var(--radius-md);padding:14px 14px 12px;display:flex;flex-direction:column;overflow:hidden;outline:1px solid var(--border);outline-offset:-1px;position:relative;opacity:0.65;transition:opacity 0.15s;"
     onmouseenter="this.style.opacity='1'"
     onmouseleave="this.style.opacity='0.65'">
  <h3 style="font-size:13px;font-weight:600;color:var(--text-primary);margin:0 0 6px;overflow:hidden;max-height:2.6em;line-height:1.3;">{escaped_title}</h3>
  <p style="font-size:12px;color:var(--text-muted);flex:1;margin:0 0 10px;overflow:hidden;line-height:1.5;">{preview}</p>
  <div style="display:flex;align-items:center;justify-content:space-between;flex-shrink:0;">
    <span style="font-size:11px;color:var(--text-muted);">Deleted {ts}</span>
    <div style="display:flex;gap:6px;">
      <button class="btn btn-ghost btn-sm"
              style="font-size:11px;padding:2px 8px;color:var(--success);"
              hx-post="/api/notes/{id}/restore"
              hx-target="#note-{id}"
              hx-swap="outerHTML"
              hx-on::after-request="htmx.trigger(document.body,'noteUpdated')"
              onclick="event.stopPropagation()">Restore</button>
      <button class="btn btn-ghost btn-sm"
              style="font-size:11px;padding:2px 8px;color:var(--destructive);"
              hx-delete="/api/notes/{id}/purge"
              hx-target="#note-{id}"
              hx-swap="outerHTML"
              hx-confirm="Permanently delete? This cannot be undone."
              onclick="event.stopPropagation()">Delete forever</button>
    </div>
  </div>
</div>"##,
        id = id,
        escaped_title = html_escape(&display_title),
        preview = preview,
        ts = ts,
    )
}

// ── Note references ───────────────────────────────────────────────────────────

/// Extracts [[Note Title]] references from content. Returns unique title strings.
fn extract_note_refs(content: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let bytes = content.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i + 1 < n {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            i += 2;
            let start = i;
            while i + 1 < n && !(bytes[i] == b']' && bytes[i + 1] == b']') {
                i += 1;
            }
            if i + 1 < n {
                let title = content[start..i].trim();
                if !title.is_empty() && !title.contains('\n') {
                    refs.push(title.to_string());
                }
                i += 2;
            }
        } else {
            i += 1;
        }
    }
    refs.sort();
    refs.dedup();
    refs
}

/// Rebuilds note_links for a given note from its content [[refs]].
async fn sync_note_links(db: &SqlitePool, from_id: &str, content: &str) {
    let refs = extract_note_refs(content);
    sqlx::query("DELETE FROM note_links WHERE from_id = ?")
        .bind(from_id)
        .execute(db)
        .await
        .ok();
    for title in &refs {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM notes WHERE LOWER(title) = LOWER(?) AND deleted_at IS NULL LIMIT 1",
        )
        .bind(title)
        .fetch_optional(db)
        .await
        .unwrap_or(None);
        if let Some((to_id,)) = row {
            if to_id != from_id {
                sqlx::query("INSERT OR IGNORE INTO note_links (from_id, to_id) VALUES (?, ?)")
                    .bind(from_id)
                    .bind(&to_id)
                    .execute(db)
                    .await
                    .ok();
            }
        }
    }
}

fn render_editor(id: &str, title: &str, content: &str) -> String {
    let escaped_content = html_escape(content);
    format!(
        r#"<div x-data="notesEditor('{id}')" style="display:flex;flex-direction:column;height:100%;">
  <!-- Title row: title | saving/saved | preview -->
  <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;border-bottom:1px solid var(--border);padding-bottom:10px;flex-shrink:0;">
    <input type="text"
           x-model="title"
           @input="saved = false"
           @input.debounce.800ms="save()"
           placeholder="Note title…"
           style="flex:1;background:transparent;font-size:16px;font-weight:600;color:var(--text-primary);outline:none;border:none;font-family:inherit;"/>
    <span x-show="saving" style="font-size:11px;color:var(--text-muted);flex-shrink:0;">Saving…</span>
    <span x-show="saved && !saving" x-transition style="font-size:11px;color:var(--success);flex-shrink:0;">Saved ✓</span>
    <button @click="preview = !preview"
            class="btn btn-secondary btn-sm"
            style="flex-shrink:0;">
      Preview
    </button>
  </div>
  <!-- Tag pills row — shows parsed inline tags, or a hint if none -->
  <div style="display:flex;align-items:center;gap:5px;flex-wrap:wrap;min-height:20px;margin-bottom:8px;flex-shrink:0;">
    <template x-if="parsedTags().length === 0">
      <span style="font-size:11px;color:var(--text-muted);opacity:0.55;font-style:italic;">Type #tag or #folder/tag in your note to organize it</span>
    </template>
    <template x-for="tag in parsedTags()" :key="tag">
      <span style="font-size:10px;padding:2px 7px;border-radius:10px;border:1px solid var(--accent);color:var(--accent);opacity:0.75;line-height:1.4;" x-text="'#' + tag"></span>
    </template>
  </div>
  <div x-show="!preview" style="flex:1;min-height:0;">
    <textarea x-model="content"
              @input="saved = false"
              @input.debounce.800ms="save()"
              placeholder="Start writing…"
              style="width:100%;height:100%;background:transparent;font-size:13px;color:var(--text-primary);resize:none;outline:none;border:none;line-height:1.6;font-family:monospace;box-sizing:border-box;">{escaped_content}</textarea>
  </div>
  <div x-show="preview" style="flex:1;overflow-y:auto;" class="prose prose-invert prose-sm max-w-none"
       x-html="renderMarkdown()"></div>
  <!-- Backlinks panel — always visible below editor/preview -->
  <div id="note-links-{id}"
       style="flex-shrink:0;border-top:1px solid var(--border);padding:8px 0 0;margin-top:4px;"
       hx-get="/api/notes/{id}/links"
       hx-trigger="load, noteUpdated from:body"
       hx-swap="innerHTML">
    <p style="font-size:11px;color:var(--text-muted);margin:0;padding:4px 0;">Loading links…</p>
  </div>
</div>
<script>
function notesEditor(noteId) {{
  return {{
    noteId: noteId,
    title: {title_json},
    content: {content_json},
    saving: false,
    saved: false,
    preview: false,
    init() {{
      this._onClear = () => {{
        this.content = '';
        this.save();
      }};
      document.addEventListener('notes:clear-content', this._onClear);
    }},
    destroy() {{
      document.removeEventListener('notes:clear-content', this._onClear);
    }},
    async save() {{
      this.saving = true;
      try {{
        await fetch('http://127.0.0.1:' + window.__API_PORT__ + '/api/notes/' + this.noteId, {{
          method: 'PUT',
          headers: {{
            'Content-Type': 'application/json',
            'Authorization': 'Bearer ' + window.__SESSION_TOKEN__,
          }},
          body: JSON.stringify({{ title: this.title, content: this.content }}),
        }});
        this.saved = true;
        htmx.trigger(document.body, 'noteUpdated');
      }} finally {{
        this.saving = false;
      }}
    }},
    parsedTags() {{
      const combined = (this.title || '') + ' ' + (this.content || '');
      const m = [...combined.matchAll(/(?:^|\s)#([a-zA-Z][a-zA-Z0-9_/]*)/g)];
      return [...new Set(m.map(x => x[1].toLowerCase()))];
    }},
    renderMarkdown() {{
      let text = (this.content || '').replace(/\[\[([^\]\n]+)\]\]/g, function(_, t) {{
        const safe = t.replace(/'/g, "\\'");
        return '<span style="color:var(--accent);cursor:pointer;text-decoration:underline;" onclick="document.dispatchEvent(new CustomEvent(\'notes:find-by-title\',{{detail:\'' + safe + '\'}}))">[[' + t + ']]</span>';
      }});
      if (typeof marked !== 'undefined') return marked.parse(text);
      return '<pre>' + text + '</pre>';
    }},
  }};
}}
</script>"#,
        id = id,
        escaped_content = escaped_content,
        title_json = serde_json::to_string(title).unwrap_or_else(|_| "\"\"".to_string()),
        content_json = serde_json::to_string(content).unwrap_or_else(|_| "\"\"".to_string()),
    )
}

fn format_timestamp(ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(ts);
    let diff = now - ts;
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

// ── Tag extraction ────────────────────────────────────────────────────────────

/// Removes inline `#tag` tokens from a string for clean display.
/// Uses the same word-boundary rule as `extract_tags` (preceded by whitespace or start).
fn strip_inline_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut prev_ws = true; // treat start-of-string as whitespace boundary

    while let Some(ch) = chars.next() {
        if ch == '#' && prev_ws {
            if let Some(&next) = chars.peek() {
                if next.is_ascii_alphabetic() {
                    // Skip the whole tag token (alphanumeric + _ + /)
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' || c == '/' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // Remove the trailing space that preceded the tag (if any)
                    let trimmed = result.trim_end_matches(' ');
                    let trim_len = trimmed.len();
                    result.truncate(trim_len);
                    prev_ws = true;
                    continue;
                }
            }
        }
        prev_ws = ch.is_whitespace();
        result.push(ch);
    }
    result.trim().to_string()
}

/// Scans note content for Bear-style inline tags: `#tagname` or `#parent/child`.
/// Tag must start with an ASCII letter. Returns unique lowercase tags.
/// No regex crate — hand-parsed byte scan.
fn extract_tags(content: &str) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'#' {
            // Require word boundary: `#` must be at start or preceded by whitespace.
            // This prevents `##MarkdownHeading` from being picked up as a tag.
            if i > 0 && !bytes[i - 1].is_ascii_whitespace() {
                i += 1;
                continue;
            }
            let start = i + 1;
            if start < len && bytes[start].is_ascii_alphabetic() {
                let mut end = start;
                while end < len
                    && (bytes[end].is_ascii_alphanumeric()
                        || bytes[end] == b'_'
                        || bytes[end] == b'/')
                {
                    end += 1;
                }
                // Strip trailing slashes
                while end > start && bytes[end - 1] == b'/' {
                    end -= 1;
                }
                if end > start {
                    let tag = content[start..end].to_lowercase();
                    if !tags.contains(&tag) {
                        tags.push(tag);
                    }
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }
    tags
}

/// Rebuilds the note_tags join table for one note and updates the JSON blob.
/// Scans both title and content so tags in the title field are indexed too.
async fn sync_note_tags(db: &SqlitePool, note_id: &str, title: &str, content: &str) {
    let combined = format!("{title} {content}");
    let tags = extract_tags(&combined);
    let _ = sqlx::query("DELETE FROM note_tags WHERE note_id = ?")
        .bind(note_id)
        .execute(db)
        .await;
    for tag in &tags {
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO note_tags (note_id, tag) VALUES (?, ?)",
        )
        .bind(note_id)
        .bind(tag)
        .execute(db)
        .await;
    }
    let json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
    let _ = sqlx::query("UPDATE notes SET tags = ? WHERE id = ?")
        .bind(&json)
        .bind(note_id)
        .execute(db)
        .await;
}

/// Renders the tag sidebar HTML from flat (tag, count) rows.
/// Hierarchy is derived at render time by splitting on '/'.
fn render_tag_tree(rows: &[(String, i64)]) -> String {
    if rows.is_empty() {
        return r#"<p style="padding:12px 10px;font-size:11px;color:var(--text-muted);margin:0;line-height:1.6;">No tags yet.<br>Type #tag in a note.</p>"#.to_string();
    }

    // Separate roots and children
    let mut roots: Vec<(&str, i64)> = Vec::new();
    let mut children: std::collections::HashMap<&str, Vec<(&str, i64)>> =
        std::collections::HashMap::new();

    for (tag, count) in rows {
        match tag.find('/') {
            None => roots.push((tag.as_str(), *count)),
            Some(slash) => {
                let parent = &tag[..slash];
                children.entry(parent).or_default().push((tag.as_str(), *count));
            }
        }
    }

    // Add implicit parents (child exists but parent has no own row)
    let child_parents: Vec<&str> = children.keys().copied().collect();
    for parent in child_parents {
        if !roots.iter().any(|(r, _)| *r == parent) {
            roots.push((parent, 0));
        }
    }
    roots.sort_by_key(|(r, _)| *r);

    let chevron_down = r#"<svg xmlns="http://www.w3.org/2000/svg" width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>"#;

    // "All Notes" reset — small uppercase section label
    let mut html = String::from(
        r##"<div style="padding:6px 10px 4px;">
  <button data-tag=""
          onclick="notesSetActiveTag('')"
          style="width:100%;text-align:left;padding:5px 8px;font-size:11px;font-weight:600;letter-spacing:0.05em;text-transform:uppercase;color:var(--text-muted);background:transparent;border:none;cursor:pointer;border-radius:5px;display:flex;align-items:center;gap:5px;"
          onmouseenter="this.style.color='var(--text-primary)';this.style.background='var(--bg-hover)'"
          onmouseleave="this.style.color='var(--text-muted)';this.style.background='transparent'"
          ondragover="event.preventDefault();this.style.background='var(--bg-hover)'"
          ondragleave="this.style.background='transparent'"
          ondrop="this.style.background='transparent';notesDrop(event,'')"
          hx-get="/api/notes" hx-target="#notes-grid" hx-swap="innerHTML">
    All Notes
  </button>
</div>
<div style="height:1px;background:var(--border);margin:2px 10px 6px;"></div>
"##,
    );

    for (root, root_count) in &roots {
        let kids = children.get(root).cloned().unwrap_or_default();
        let has_kids = !kids.is_empty();
        let count_badge = if *root_count > 0 {
            format!(r#"<span style="margin-left:auto;font-size:9px;padding:1px 5px;border-radius:8px;background:var(--bg-base);color:var(--text-muted);font-weight:500;">{root_count}</span>"#)
        } else {
            String::new()
        };

        if has_kids {
            html.push_str(&format!(
                r##"<div x-data="{{ open: true }}" style="margin:0 6px 4px;">
  <div data-tag-card="{root}"
       style="display:flex;align-items:center;background:var(--bg-elevated);border-radius:6px;border-left:2px solid var(--accent);"
       ondragover="event.preventDefault();this.style.background='var(--bg-hover)'"
       ondragleave="this.style.background='var(--bg-elevated)'"
       ondrop="this.style.background='var(--bg-elevated)';notesDrop(event,'{root}')"
       oncontextmenu="notesTagContextMenu(event,'{root}')">
    <div @click.stop="open=!open"
         style="flex-shrink:0;cursor:pointer;padding:6px 10px 6px 8px;color:var(--text-muted);display:flex;align-items:center;align-self:stretch;transition:color 0.1s;user-select:none;"
         :style="open ? '' : 'transform:rotate(-90deg)'"
         onmouseenter="this.style.color='var(--text-primary)'"
         onmouseleave="this.style.color='var(--text-muted)'">
      {chevron}
    </div>
    <button data-tag="{root}"
            onclick="notesSetActiveTag(this.dataset.tag)"
            hx-get="/api/notes?tag={root}" hx-target="#notes-grid" hx-swap="innerHTML"
            style="flex:1;min-width:0;text-align:left;background:transparent;border:none;cursor:pointer;padding:6px 8px 6px 2px;display:flex;align-items:center;gap:5px;"
            onmouseenter="this.style.background='var(--bg-hover)';this.style.borderRadius='0 4px 4px 0'"
            onmouseleave="this.style.background='transparent';this.style.borderRadius=''">
      <span style="color:var(--accent);font-size:11px;font-weight:700;flex-shrink:0;">#</span>
      <span style="font-size:12px;font-weight:600;color:var(--text-primary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{root}</span>
      {count_badge}
    </button>
  </div>
  <div x-show="open" style="margin-top:2px;margin-left:10px;border-left:1px solid var(--border);padding-left:4px;">
"##,
                chevron = chevron_down,
                root = root,
                count_badge = count_badge,
            ));
            for (child_tag, child_count) in &kids {
                let child_label = child_tag.find('/').map(|s| &child_tag[s + 1..]).unwrap_or(child_tag);
                html.push_str(&format!(
                    r##"    <button data-tag="{child_tag}"
            onclick="notesSetActiveTag(this.dataset.tag)"
            style="width:100%;text-align:left;padding:3px 6px;font-size:11px;color:var(--text-muted);background:transparent;border:none;cursor:pointer;display:flex;align-items:center;gap:4px;border-radius:4px;"
            onmouseenter="this.style.background='var(--bg-hover)';this.style.color='var(--text-primary)'"
            onmouseleave="this.style.background='transparent';this.style.color='var(--text-muted)'"
            ondragover="event.preventDefault();this.style.background='var(--bg-hover)'"
            ondragleave="this.style.background='transparent'"
            ondrop="this.style.background='transparent';notesDrop(event,this.dataset.tag)"
            oncontextmenu="notesTagContextMenu(event,'{child_tag}')"
            hx-get="/api/notes?tag={child_tag_enc}" hx-target="#notes-grid" hx-swap="innerHTML">
      <span style="color:var(--accent);opacity:0.5;font-size:10px;flex-shrink:0;">›</span>
      <span style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{child_label}</span>
      <span style="font-size:9px;color:var(--text-muted);opacity:0.7;flex-shrink:0;">{child_count}</span>
    </button>
"##,
                    child_tag = child_tag,
                    child_tag_enc = child_tag.replace('/', "%2F"),
                    child_label = child_label,
                    child_count = child_count,
                ));
            }
            html.push_str("  </div>\n</div>\n");
        } else {
            html.push_str(&format!(
                r##"<div style="margin:0 6px 4px;">
  <button data-tag="{root}"
          onclick="notesSetActiveTag(this.dataset.tag)"
          hx-get="/api/notes?tag={root}" hx-target="#notes-grid" hx-swap="innerHTML"
          style="width:100%;text-align:left;background:var(--bg-elevated);border:none;cursor:pointer;padding:6px 9px;border-radius:6px;display:flex;align-items:center;gap:5px;border-left:2px solid var(--accent);"
          onmouseenter="this.style.background='var(--bg-hover)'"
          onmouseleave="this.style.background='var(--bg-elevated)'"
          ondragover="event.preventDefault();this.style.background='var(--bg-hover)'"
          ondragleave="this.style.background='var(--bg-elevated)'"
          ondrop="this.style.background='var(--bg-elevated)';notesDrop(event,this.dataset.tag)"
          oncontextmenu="notesTagContextMenu(event,'{root}')">
    <span style="color:var(--accent);font-size:11px;font-weight:700;flex-shrink:0;">#</span>
    <span style="font-size:12px;font-weight:600;color:var(--text-primary);flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{root}</span>
    {count_badge}
  </button>
</div>
"##,
                root = root,
                count_badge = count_badge,
            ));
        }
    }
    html
}

// ── Query/body params ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    PAGE
}

#[derive(Deserialize)]
pub struct CreateBody {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tags: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateBody {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub pinned: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    // Detect #tag prefix in free-text search — route to tag filter instead of FTS5
    let tag_from_q: Option<String> = if params.q.starts_with('#') {
        let t = params.q[1..].trim().to_lowercase();
        if !t.is_empty() { Some(t) } else { None }
    } else {
        None
    };
    // Effective tag: explicit ?tag= param wins, else derived from #prefix in q
    let effective_tag = if !params.tag.is_empty() {
        Some(params.tag.as_str())
    } else {
        tag_from_q.as_deref()
    };

    // Fetch one extra row to detect whether a next page exists
    let fetch_limit = params.limit + 1;
    let browsing = params.q.is_empty() && effective_tag.is_none();
    let mut rows: Vec<(String, String, String, i64, i64)> = if let Some(tag) = effective_tag {
        // Tag filter: exact match OR any tag whose last segment matches (e.g. #rod → work/rod).
        // GROUP BY deduplicates notes matching multiple tags; MIN(t.tag) orders by tree path.
        let like_pattern = format!("%/{}", tag);
        sqlx::query_as(
            "SELECT n.id, n.title, n.content, n.pinned, n.updated_at
             FROM notes n
             JOIN note_tags t ON t.note_id = n.id
             WHERE (t.tag = ? OR t.tag LIKE ?) AND n.deleted_at IS NULL
             GROUP BY n.id
             ORDER BY n.pinned DESC, MIN(t.tag), n.updated_at DESC
             LIMIT 200",
        )
        .bind(tag)
        .bind(&like_pattern)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else if !params.q.is_empty() {
        // FTS5 full-text search
        sqlx::query_as(
            "SELECT n.id, n.title, n.content, n.pinned, n.updated_at
             FROM notes n
             JOIN notes_fts f ON f.rowid = n.rowid
             WHERE notes_fts MATCH ? AND n.deleted_at IS NULL
             ORDER BY n.pinned DESC, rank
             LIMIT 200 OFFSET 0",
        )
        .bind(&params.q)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as(
            "SELECT id, title, content, pinned, updated_at FROM notes
             WHERE deleted_at IS NULL
             ORDER BY pinned DESC, updated_at DESC LIMIT ? OFFSET ?",
        )
        .bind(fetch_limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    };

    let has_more = rows.len() > params.limit as usize && browsing;
    rows.truncate(params.limit as usize);
    let next_offset = params.offset + params.limit;
    let show_buckets = params.offset == 0 || effective_tag.is_some() || !params.q.is_empty();

    Html(render_note_list(&rows, has_more, next_offset, show_buckets))
}

pub async fn create_handler(
    State(state): State<Arc<AppState>>,
    Form(body): Form<CreateBody>,
) -> Response {
    let id = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let tags = body.tags.unwrap_or_else(|| "[]".to_string());

    let result = sqlx::query(
        "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&body.title)
    .bind(&body.content)
    .bind(&body.content) // Phase 1: content_fts = content (Markdown stripping in Phase 2)
    .bind(&tags)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            sync_note_tags(&state.db, &id, &body.title, &body.content).await;
            sync_note_links(&state.db, &id, &body.content).await;
            state.event_bus.publish(Event::NoteCreated {
                id: id.clone(),
                title: body.title.clone(),
            });
            let card = render_note_card(&id, &body.title, &body.content, 0, now);
            (StatusCode::CREATED, [("X-Note-Id", id)], Html(card)).into_response()
        }
        Err(e) => {
            log::error!("Note create failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "insert failed" })),
            )
                .into_response()
        }
    }
}

pub async fn get_handler(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT title, content FROM notes WHERE id = ? AND deleted_at IS NULL")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

    match row {
        Some((title, content)) => Html(render_editor(&id, &title, &content)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" }))).into_response(),
    }
}

pub async fn get_content_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let row: Option<(String,)> = sqlx::query_as("SELECT content FROM notes WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);
    match row {
        Some((content,)) => Json(serde_json::json!({ "content": content })).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "not found" })),
        )
            .into_response(),
    }
}

pub async fn update_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateBody>,
) -> impl IntoResponse {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Track whether title or content changed — needed to decide if tag sync is required
    let title_changed = body.title.is_some();
    let content_changed = body.content.is_some();

    // Build SET clause dynamically — raw query (no macros), with comment per CLAUDE.md.
    // This is the one place dynamic SQL is used; fields are bound parameters, not interpolated.
    let mut set_parts = vec!["updated_at = ?"];
    let mut title_val: Option<String> = None;
    let mut content_val: Option<String> = None;
    let mut tags_val: Option<String> = None;
    let mut pinned_val: Option<i64> = None;

    if body.title.is_some() {
        set_parts.push("title = ?");
        title_val = body.title;
    }
    if body.content.is_some() {
        // content_fts mirrors content (Phase 1; Markdown stripping deferred to Phase 2)
        set_parts.push("content = ?");
        set_parts.push("content_fts = ?");
        content_val = body.content.clone();
    }
    if body.tags.is_some() {
        set_parts.push("tags = ?");
        tags_val = body.tags;
    }
    if body.pinned.is_some() {
        set_parts.push("pinned = ?");
        pinned_val = body.pinned;
    }

    let sql = format!("UPDATE notes SET {} WHERE id = ?", set_parts.join(", "));

    let mut q = sqlx::query(&sql).bind(now);
    if let Some(v) = title_val {
        q = q.bind(v);
    }
    if let Some(v) = content_val {
        q = q.bind(v.clone()).bind(v); // content + content_fts
    }
    if let Some(v) = tags_val {
        q = q.bind(v);
    }
    if let Some(v) = pinned_val {
        q = q.bind(v);
    }
    q = q.bind(&id);

    match q.execute(&state.db).await {
        Ok(_) => {
            if title_changed || content_changed {
                // Fetch the post-update values so tags are always in sync with actual DB state
                if let Some((t, c)) = sqlx::query_as::<_, (String, String)>(
                    "SELECT title, content FROM notes WHERE id = ?",
                )
                .bind(&id)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None)
                {
                    sync_note_tags(&state.db, &id, &t, &c).await;
                    sync_note_links(&state.db, &id, &c).await;
                }
            }
            state.event_bus.publish(Event::NoteUpdated { id });
            Json(json!({ "ok": true }))
        }
        Err(e) => {
            log::error!("Note update failed: {e}");
            Json(json!({ "ok": false, "error": e.to_string() }))
        }
    }
}

pub async fn delete_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Soft delete: move to trash, preserve data for 30 days
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    sqlx::query("UPDATE notes SET deleted_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();
    // Return empty — HTMX outerHTML swap removes the card
    Html(String::new())
}

pub async fn restore_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    sqlx::query("UPDATE notes SET deleted_at = NULL WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();
    // Return empty to remove card from trash view; HX-Trigger refreshes sidebar
    (
        [("HX-Trigger", "noteUpdated")],
        Html(String::new()),
    )
}

pub async fn purge_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM notes WHERE id = ? AND deleted_at IS NOT NULL")
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();
    Html(String::new())
}

pub async fn trash_list_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Auto-purge items older than 30 days before rendering
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let cutoff = now - 30 * 86_400;
    sqlx::query("DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?")
        .bind(cutoff)
        .execute(&state.db)
        .await
        .ok();

    let rows: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT id, title, content, deleted_at FROM notes
         WHERE deleted_at IS NOT NULL
         ORDER BY deleted_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<div style="grid-column:1/-1;padding:48px 16px;text-align:center;">
  <p style="font-size:15px;font-weight:500;color:var(--text-primary);margin:0 0 8px;">Trash is empty</p>
  <p style="font-size:13px;color:var(--text-muted);margin:0;line-height:1.6;">Deleted notes are kept here for 30 days,<br>then automatically removed forever.</p>
</div>"#.to_string());
    }

    let mut html = String::from(
        r#"<div style="grid-column:1/-1;padding:6px 0 12px;display:flex;align-items:center;justify-content:space-between;">
  <p style="font-size:11px;color:var(--text-muted);margin:0;">Items are automatically deleted after 30 days.</p>
</div>"#,
    );
    for (id, title, content, deleted_at) in &rows {
        html.push_str(&render_trash_note_card(id, title, content, *deleted_at));
    }
    Html(html)
}

pub async fn pin_toggle_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    // Toggle pinned and return updated card HTML
    sqlx::query("UPDATE notes SET pinned = CASE WHEN pinned = 1 THEN 0 ELSE 1 END WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();

    let row: Option<(String, String, String, i64, i64)> =
        sqlx::query_as("SELECT id, title, content, pinned, updated_at FROM notes WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

    match row {
        Some((id, title, content, pinned, updated_at)) => {
            Html(render_note_card(&id, &title, &content, pinned, updated_at)).into_response()
        }
        None => (StatusCode::NOT_FOUND, Html(String::new())).into_response(),
    }
}

/// Replaces or removes `#from_tag` (and `#from_tag/child`) tokens in `text`.
/// Returns `(new_text, was_found)`. Uses the same word-boundary rule as `extract_tags`.
fn replace_tag_in_text(text: &str, from_tag: &str, to_tag: &str) -> (String, bool) {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut prev_ws = true;
    let mut found = false;

    while let Some(ch) = chars.next() {
        if ch == '#' && prev_ws {
            if let Some(&next) = chars.peek() {
                if next.is_ascii_alphabetic() {
                    let mut tag_buf = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' || c == '/' {
                            tag_buf.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    let tag_lower = tag_buf.to_lowercase();
                    let child_suffix = if tag_lower == from_tag {
                        Some(String::new())
                    } else if from_tag.is_empty() {
                        None // empty from_tag means "add" mode, not replace
                    } else if tag_lower.starts_with(&format!("{}/", from_tag)) {
                        Some(tag_lower[from_tag.len()..].to_string()) // includes leading '/'
                    } else {
                        None
                    };

                    if let Some(suffix) = child_suffix {
                        found = true;
                        let trimmed_len = result.trim_end_matches(' ').len();
                        result.truncate(trimmed_len);
                        if !to_tag.is_empty() {
                            if !result.is_empty() { result.push(' '); }
                            result.push('#');
                            result.push_str(to_tag);
                            result.push_str(&suffix); // e.g. "/child" or ""
                        }
                        prev_ws = true;
                        continue;
                    } else {
                        result.push('#');
                        result.push_str(&tag_buf);
                        prev_ws = false;
                        continue;
                    }
                }
            }
        }
        prev_ws = ch.is_whitespace();
        result.push(ch);
    }
    (result.trim().to_string(), found)
}

#[derive(Deserialize)]
pub struct RetagBody {
    pub from_tag: String,
    pub to_tag: String,
}

pub async fn retag_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<RetagBody>,
) -> impl IntoResponse {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT title, content FROM notes WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

    let (title, content) = match row {
        Some(r) => r,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" }))).into_response(),
    };

    let from = body.from_tag.to_lowercase();
    let to = body.to_tag.to_lowercase();

    let (mut new_title, found_title) = replace_tag_in_text(&title, &from, &to);
    let (new_content, found_content) = replace_tag_in_text(&content, &from, &to);

    // If the tag wasn't found anywhere and we have a destination, append to title
    if !found_title && !found_content && !to.is_empty() {
        if new_title.is_empty() {
            new_title = format!("#{to}");
        } else {
            new_title = format!("{new_title} #{to}");
        }
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    sqlx::query(
        "UPDATE notes SET title = ?, content = ?, content_fts = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&new_title)
    .bind(&new_content)
    .bind(&new_content)
    .bind(now)
    .bind(&id)
    .execute(&state.db)
    .await
    .ok();

    sync_note_tags(&state.db, &id, &new_title, &new_content).await;
    state.event_bus.publish(Event::NoteUpdated { id });
    Json(json!({ "ok": true })).into_response()
}

#[derive(Deserialize)]
pub struct DeleteTagQuery {
    pub name: String,
}

pub async fn delete_tag_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DeleteTagQuery>,
) -> impl IntoResponse {
    let tag = params.name.to_lowercase();

    // Fetch all non-deleted notes that reference this tag or any child tag
    let notes: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT DISTINCT n.id, n.title, n.content FROM notes n \
         JOIN note_tags t ON t.note_id = n.id \
         WHERE (t.tag = ? OR t.tag LIKE ?) AND n.deleted_at IS NULL",
    )
    .bind(&tag)
    .bind(format!("{}/%", tag))
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    for (id, title, content) in notes {
        let (new_title, _) = replace_tag_in_text(&title, &tag, "");
        let (new_content, _) = replace_tag_in_text(&content, &tag, "");
        sqlx::query(
            "UPDATE notes SET title = ?, content = ?, content_fts = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&new_title)
        .bind(&new_content)
        .bind(&new_content)
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();
        sync_note_tags(&state.db, &id, &new_title, &new_content).await;
    }

    StatusCode::NO_CONTENT
}

pub async fn tags_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT t.tag, COUNT(*) AS count FROM note_tags t
         JOIN notes n ON n.id = t.note_id
         WHERE n.deleted_at IS NULL
         GROUP BY t.tag ORDER BY t.tag",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    Html(render_tag_tree(&rows))
}

pub async fn links_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Backlinks: notes that reference this note
    let backlinks: Vec<(String, String)> = sqlx::query_as(
        "SELECT n.id, n.title FROM note_links nl
         JOIN notes n ON n.id = nl.from_id
         WHERE nl.to_id = ? AND n.deleted_at IS NULL
         ORDER BY n.updated_at DESC",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Outgoing refs: notes this note references
    let outgoing: Vec<(String, String)> = sqlx::query_as(
        "SELECT n.id, n.title FROM note_links nl
         JOIN notes n ON n.id = nl.to_id
         WHERE nl.from_id = ? AND n.deleted_at IS NULL
         ORDER BY n.title ASC",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if backlinks.is_empty() && outgoing.is_empty() {
        return Html(String::new());
    }

    let mut html = String::new();
    if !backlinks.is_empty() {
        html.push_str(r#"<p style="font-size:10px;font-weight:700;color:var(--text-muted);letter-spacing:0.06em;text-transform:uppercase;margin:0 0 4px;">Backlinks</p>"#);
        for (link_id, link_title) in &backlinks {
            let title = if link_title.is_empty() { "Untitled" } else { link_title.as_str() };
            html.push_str(&format!(
                r#"<button style="display:block;width:100%;text-align:left;font-size:11px;color:var(--accent);background:transparent;border:none;cursor:pointer;padding:2px 0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;"
                        onclick="document.dispatchEvent(new CustomEvent('notes:open-editor',{{detail:{{id:'{link_id}'}}}}))"
                        onmouseenter="this.style.textDecoration='underline'"
                        onmouseleave="this.style.textDecoration='none'">↩ {title}</button>"#,
                link_id = link_id,
                title = html_escape(title),
            ));
        }
    }
    if !outgoing.is_empty() {
        if !backlinks.is_empty() { html.push_str(r#"<div style="height:6px;"></div>"#); }
        html.push_str(r#"<p style="font-size:10px;font-weight:700;color:var(--text-muted);letter-spacing:0.06em;text-transform:uppercase;margin:0 0 4px;">References</p>"#);
        for (link_id, link_title) in &outgoing {
            let title = if link_title.is_empty() { "Untitled" } else { link_title.as_str() };
            html.push_str(&format!(
                r#"<button style="display:block;width:100%;text-align:left;font-size:11px;color:var(--accent);background:transparent;border:none;cursor:pointer;padding:2px 0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;"
                        onclick="document.dispatchEvent(new CustomEvent('notes:open-editor',{{detail:{{id:'{link_id}'}}}}))"
                        onmouseenter="this.style.textDecoration='underline'"
                        onmouseleave="this.style.textDecoration='none'">↗ {title}</button>"#,
                link_id = link_id,
                title = html_escape(title),
            ));
        }
    }
    Html(html)
}

#[derive(Deserialize)]
pub struct ResolveQuery {
    pub title: String,
}

pub async fn resolve_by_title_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ResolveQuery>,
) -> impl IntoResponse {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM notes WHERE LOWER(title) = LOWER(?) AND deleted_at IS NULL LIMIT 1",
    )
    .bind(&params.title)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);
    match row {
        Some((id,)) => Json(json!({ "id": id })).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" }))).into_response(),
    }
}

pub async fn clear_notes_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    sqlx::query("UPDATE notes SET deleted_at = ? WHERE deleted_at IS NULL")
        .bind(now)
        .execute(&state.db)
        .await
        .ok();
    ([("HX-Trigger", "noteUpdated")], Html(String::new()))
}

pub async fn duplicate_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let row: Option<(String, String, String)> = sqlx::query_as(
        "SELECT title, content, tags FROM notes WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some((title, content, tags)) = row else {
        return (StatusCode::NOT_FOUND, Html(String::new())).into_response();
    };

    let new_id = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let new_title = format!("Copy of {title}");

    let result = sqlx::query(
        "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&new_id)
    .bind(&new_title)
    .bind(&content)
    .bind(&content)
    .bind(&tags)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            sync_note_tags(&state.db, &new_id, &new_title, &content).await;
            sync_note_links(&state.db, &new_id, &content).await;
            ([("HX-Trigger", "noteUpdated")],
             Html(render_note_card(&new_id, &new_title, &content, 0, now))).into_response()
        }
        Err(e) => {
            log::error!("Note duplicate failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Html(String::new())).into_response()
        }
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/notes", get(list_handler).post(create_handler).delete(clear_notes_handler))
        // Static paths must be declared before /:id (matchit static > param)
        .route("/api/notes/tags", get(tags_handler).delete(delete_tag_handler))
        .route("/api/notes/trash", get(trash_list_handler))
        .route("/api/notes/resolve", get(resolve_by_title_handler))
        .route(
            "/api/notes/:id",
            get(get_handler).put(update_handler).delete(delete_handler),
        )
        .route("/api/notes/:id/content", get(get_content_handler))
        .route("/api/notes/:id/pin", post(pin_toggle_handler))
        .route("/api/notes/:id/retag", put(retag_handler))
        .route("/api/notes/:id/restore", post(restore_handler))
        .route("/api/notes/:id/purge", axum::routing::delete(purge_handler))
        .route("/api/notes/:id/duplicate", post(duplicate_handler))
        .route("/api/notes/:id/links", get(links_handler))
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
        let audio_recording = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let mcp_sessions =
            std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
        let plugin_registry =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let plugin_processes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        Arc::new(AppState {
            db,
            session_token: "test-token".to_string(),
            port: 0,
            event_bus: crate::event_bus::EventBus::new(),
            clipboard_suppress_tx,
            download_states,
            voice_recording,
            screen_recording,
            audio_recording,
            mcp_sessions,
            plugin_registry,
            plugin_processes,
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

    async fn get(app: axum::Router, uri: &str) -> (StatusCode, String) {
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
    async fn create_and_fts_sync() {
        let state = make_test_state().await;
        let (status, _) = post_form(
            test_app(state.clone()),
            "/api/notes",
            &[("title", "Hello FTS"), ("content", "searchable content")],
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'searchable'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn update_and_fts_sync() {
        let state = make_test_state().await;
        let id = "note-upd";
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES (?, 'TestNote', 'xyzoldterm', 'xyzoldterm', '[]', ?, ?)",
        )
        .bind(id)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        // Call handler directly to bypass router (tests business logic, not routing)
        update_handler(
            State(state.clone()),
            Path(id.to_string()),
            Json(UpdateBody {
                content: Some("xyzupdatedterm content".to_string()),
                title: None,
                tags: None,
                pinned: None,
            }),
        )
        .await;

        let old_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'xyzoldterm'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(old_count.0, 0);

        let new_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'xyzupdatedterm'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(new_count.0, 1);
    }

    #[tokio::test]
    async fn delete_and_fts_removal() {
        let state = make_test_state().await;
        let id = "note-del";
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES (?, 'DelTitle', 'del content', 'del content', '[]', ?, ?)",
        )
        .bind(id)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        // Call handler directly to bypass router
        delete_handler(State(state.clone()), Path(id.to_string())).await;

        // Soft-delete: note still in FTS (not hard-deleted), but deleted_at is set
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notes WHERE title = 'DelTitle' AND deleted_at IS NOT NULL",
        )
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn list_pinned_first() {
        let state = make_test_state().await;
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, pinned, created_at, updated_at)
             VALUES ('unpinned', 'B', '', '', '[]', 0, ?, ?)",
        )
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, pinned, created_at, updated_at)
             VALUES ('pinned', 'A', '', '', '[]', 1, ?, ?)",
        )
        .bind(now + 1)
        .bind(now + 1)
        .execute(&state.db)
        .await
        .unwrap();

        let (status, body) = get(test_app(state), "/api/notes").await;
        assert_eq!(status, StatusCode::OK);
        let pos_pinned = body.find("note-pinned").unwrap();
        let pos_unpinned = body.find("note-unpinned").unwrap();
        assert!(pos_pinned < pos_unpinned, "pinned note should appear first");
    }

    #[tokio::test]
    async fn pin_toggle() {
        let state = make_test_state().await;
        let id = "toggle-note";
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, pinned, created_at, updated_at)
             VALUES (?, 'Pin me', '', '', '[]', 0, ?, ?)",
        )
        .bind(id)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        // Call handler directly to bypass router
        pin_toggle_handler(State(state.clone()), Path(id.to_string())).await;

        let row: (i64,) = sqlx::query_as("SELECT pinned FROM notes WHERE id = ?")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(row.0, 1);
    }

    // ── Strip inline tags unit tests ───────────────────────────────────────

    #[test]
    fn strip_tags_from_title() {
        assert_eq!(strip_inline_tags("List of pendings #newfolder"), "List of pendings");
        assert_eq!(strip_inline_tags("#work project notes"), "project notes");
        assert_eq!(strip_inline_tags("clean title"), "clean title");
        assert_eq!(strip_inline_tags("Meeting #work/client recap"), "Meeting recap");
    }

    // ── Tag extraction unit tests ──────────────────────────────────────────

    #[test]
    fn extract_tags_basic() {
        let tags = extract_tags("Hello #world and #rust are great");
        assert!(tags.contains(&"world".to_string()));
        assert!(tags.contains(&"rust".to_string()));
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn extract_tags_nested() {
        let tags = extract_tags("Note about #work/project/alpha tasks");
        assert!(tags.contains(&"work/project/alpha".to_string()));
    }

    #[test]
    fn extract_tags_dedup() {
        let tags = extract_tags("#rust is great, #rust is awesome");
        assert_eq!(tags.iter().filter(|t| t.as_str() == "rust").count(), 1);
    }

    #[test]
    fn extract_tags_must_start_with_letter() {
        let tags = extract_tags("Number #123 is not a tag, but #valid is");
        assert!(!tags.contains(&"123".to_string()));
        assert!(tags.contains(&"valid".to_string()));
    }

    #[test]
    fn extract_tags_ignores_markdown_headings() {
        // ##Heading and ###Heading must NOT produce tags (no whitespace before `#`)
        let tags = extract_tags("##Subtítulo\n### Heading Three\n#valid tag here");
        assert!(!tags.iter().any(|t| t.starts_with("subt")), "##Heading should not produce a tag");
        assert!(!tags.contains(&"heading".to_string()), "### Heading should not produce a tag");
        assert!(tags.contains(&"valid".to_string()), "#valid (preceded by whitespace/newline) should be a tag");
    }

    // ── Tag route integration tests ────────────────────────────────────────

    #[tokio::test]
    async fn tag_filter_route() {
        let state = make_test_state().await;
        let now = 1000i64;
        // Insert note with #work tag
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES ('n-work', 'Work Note', 'content #work task', 'content #work task', '[]', ?, ?)",
        )
        .bind(now).bind(now).execute(&state.db).await.unwrap();
        sync_note_tags(&state.db, "n-work", "", "content #work task").await;

        // Insert note WITHOUT #work tag
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES ('n-other', 'Other', 'no tags here', 'no tags here', '[]', ?, ?)",
        )
        .bind(now).bind(now).execute(&state.db).await.unwrap();

        let (status, body) = get(test_app(state), "/api/notes?tag=work").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("note-n-work"), "tagged note should appear");
        assert!(!body.contains("note-n-other"), "untagged note should be excluded");
    }

    #[tokio::test]
    async fn tags_handler_returns_tree() {
        let state = make_test_state().await;
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES ('nt1', 'T1', '#alpha #beta/sub', '#alpha #beta/sub', '[]', ?, ?)",
        )
        .bind(now).bind(now).execute(&state.db).await.unwrap();
        sync_note_tags(&state.db, "nt1", "", "#alpha #beta/sub").await;

        let (status, body) = get(test_app(state), "/api/notes/tags").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("alpha"), "root tag alpha should appear");
        assert!(body.contains("beta"), "root tag beta should appear");
        assert!(body.contains("sub"), "child tag sub should appear");
        assert!(body.contains("All Notes"), "reset button should appear");
    }

    #[tokio::test]
    async fn fts5_match_search() {
        let state = make_test_state().await;
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES ('n1', 'Rust Programming', 'learn ownership', 'learn ownership', '[]', ?, ?)",
        )
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES ('n2', 'Python Guide', 'async await syntax', 'async await syntax', '[]', ?, ?)",
        )
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        let (status, body) = get(test_app(state), "/api/notes?q=ownership").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Rust Programming"));
        assert!(!body.contains("Python Guide"));
    }
}
