-- Pipeline folders: group pipelines in the left sidebar list.
--
-- Pipelines with folder_id = NULL appear in the uncategorised section at the bottom.

CREATE TABLE IF NOT EXISTS pipeline_folders (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0
);

ALTER TABLE pipelines ADD COLUMN folder_id TEXT;
