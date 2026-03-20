-- Soft-delete support: items moved to trash before permanent deletion
-- Both tables get deleted_at; NULL = active, unix_ts = deleted at that time
ALTER TABLE notes ADD COLUMN deleted_at INTEGER DEFAULT NULL;
ALTER TABLE clipboard ADD COLUMN deleted_at INTEGER DEFAULT NULL;

CREATE INDEX IF NOT EXISTS idx_notes_deleted_at ON notes(deleted_at);
CREATE INDEX IF NOT EXISTS idx_clipboard_deleted_at ON clipboard(deleted_at);
