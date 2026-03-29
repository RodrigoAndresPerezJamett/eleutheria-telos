// Tool modules registered as they are implemented.
// Phase 1: clipboard, notes, search
// Phase 2: models, ocr, voice, translate
// Phase 3: recorder, photo_editor, video_processor

pub mod audio_recorder;

/// Escape user-supplied search strings for use in SQLite LIKE patterns.
/// Must be paired with `ESCAPE '\'` in the SQL query. See D-052.
///
/// Escapes: `\` → `\\`, `%` → `\%`, `_` → `\_`
pub fn like_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
pub mod quick_actions;
pub mod clipboard;
pub mod models;
pub mod notes;
pub mod ocr;
pub mod photo_editor;
pub mod screen_recorder;
pub mod search;
pub mod translate;
pub mod video_processor;
pub mod voice;
