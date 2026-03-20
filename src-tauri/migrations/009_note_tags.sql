-- Normalized tag index for notes.
-- Source of truth is notes.tags (JSON array), rebuilt by sync_note_tags() on every save.
-- ON DELETE CASCADE removes rows automatically when a note is deleted.

CREATE TABLE IF NOT EXISTS note_tags (
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    tag     TEXT NOT NULL,
    PRIMARY KEY (note_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_note_tags_tag     ON note_tags(tag);
CREATE INDEX IF NOT EXISTS idx_note_tags_note_id ON note_tags(note_id);
