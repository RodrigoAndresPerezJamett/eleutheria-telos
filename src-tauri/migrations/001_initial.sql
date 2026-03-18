-- Notes
CREATE TABLE IF NOT EXISTS notes (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL DEFAULT '',
  content     TEXT NOT NULL DEFAULT '',
  content_fts TEXT,
  tags        TEXT DEFAULT '[]',
  pinned      INTEGER DEFAULT 0,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(title, content_fts, content='notes', content_rowid='rowid');

-- Clipboard History
CREATE TABLE IF NOT EXISTS clipboard (
  id           TEXT PRIMARY KEY,
  content      TEXT NOT NULL,
  content_type TEXT NOT NULL,
  source_app   TEXT,
  created_at   INTEGER NOT NULL
);

-- Settings
CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- Plugin sandboxed storage
CREATE TABLE IF NOT EXISTS plugin_data (
  plugin_id TEXT NOT NULL,
  key       TEXT NOT NULL,
  value     TEXT NOT NULL,
  PRIMARY KEY (plugin_id, key)
);

-- Model registry
CREATE TABLE IF NOT EXISTS models (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  tool          TEXT NOT NULL,
  size_bytes    INTEGER,
  path          TEXT,
  downloaded    INTEGER DEFAULT 0,
  downloaded_at INTEGER
);
