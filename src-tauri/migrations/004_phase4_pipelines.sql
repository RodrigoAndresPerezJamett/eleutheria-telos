-- Quick Actions: visual pipeline builder
-- trigger JSON format: {"type":"Manual"} | {"type":"OcrCompleted"} |
--                      {"type":"TranscriptionCompleted"} | {"type":"ClipboardChanged"}

CREATE TABLE IF NOT EXISTS pipelines (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL,
  trigger    TEXT NOT NULL DEFAULT '{"type":"Manual"}',
  enabled    INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL
);

-- Steps are ordered by step_order (ascending).
-- ON DELETE CASCADE ensures steps are removed with their pipeline.
CREATE TABLE IF NOT EXISTS pipeline_steps (
  id          TEXT PRIMARY KEY,
  pipeline_id TEXT NOT NULL REFERENCES pipelines(id) ON DELETE CASCADE,
  step_order  INTEGER NOT NULL,
  tool        TEXT NOT NULL,
  config      TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS pipeline_steps_pipeline_id ON pipeline_steps(pipeline_id, step_order);
