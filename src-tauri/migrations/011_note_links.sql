-- Bidirectional note references: [[Note Title]] → note_links rows
-- Rebuilt from scratch on every note save (derived read index)
CREATE TABLE IF NOT EXISTS note_links (
    from_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    to_id   TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    PRIMARY KEY (from_id, to_id)
);

CREATE INDEX IF NOT EXISTS idx_note_links_from ON note_links(from_id);
CREATE INDEX IF NOT EXISTS idx_note_links_to   ON note_links(to_id);
