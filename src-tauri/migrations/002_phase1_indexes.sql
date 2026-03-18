CREATE INDEX IF NOT EXISTS idx_clipboard_created_at ON clipboard(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notes_updated_at     ON notes(pinned DESC, updated_at DESC);

-- FTS5 sync triggers (notes_fts uses content='notes' — needs manual sync)
CREATE TRIGGER IF NOT EXISTS notes_fts_insert AFTER INSERT ON notes BEGIN
  INSERT INTO notes_fts(rowid, title, content_fts) VALUES (new.rowid, new.title, new.content_fts);
END;
CREATE TRIGGER IF NOT EXISTS notes_fts_delete AFTER DELETE ON notes BEGIN
  INSERT INTO notes_fts(notes_fts, rowid, title, content_fts)
    VALUES ('delete', old.rowid, old.title, old.content_fts);
END;
CREATE TRIGGER IF NOT EXISTS notes_fts_update AFTER UPDATE ON notes BEGIN
  INSERT INTO notes_fts(notes_fts, rowid, title, content_fts)
    VALUES ('delete', old.rowid, old.title, old.content_fts);
  INSERT INTO notes_fts(rowid, title, content_fts) VALUES (new.rowid, new.title, new.content_fts);
END;
