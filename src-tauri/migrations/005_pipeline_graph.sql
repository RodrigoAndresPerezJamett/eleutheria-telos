-- Quick Actions: graph-based pipeline model (replaces linear pipeline_steps)
--
-- node_type values: 'trigger' | 'action' | 'condition' | 'loop' | 'end'
--
-- trigger node config:  {"trigger": "Manual"|"OcrCompleted"|"TranscriptionCompleted"|"ClipboardChanged"}
-- action node config:   {"tool": "translate"|"copy_clipboard"|"save_note", ...tool params}
-- condition node config:{"condition": "contains"|"length_gt"|"always_true", "value": "..."}
-- loop node config:     {"max_iterations": 10, "timeout_secs": 60}
-- end node config:      {}
--
-- edge_label values:
--   "default"   — single output (trigger, action, end)
--   "true"      — condition branch: condition is true
--   "false"     — condition branch: condition is false
--   "body"      — loop branch: enter loop body
--   "done"      — loop branch: exit loop

CREATE TABLE IF NOT EXISTS pipeline_nodes (
  id          TEXT PRIMARY KEY,
  pipeline_id TEXT NOT NULL REFERENCES pipelines(id) ON DELETE CASCADE,
  node_type   TEXT NOT NULL,
  config      TEXT NOT NULL DEFAULT '{}',
  pos_x       REAL NOT NULL DEFAULT 0,
  pos_y       REAL NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS pipeline_edges (
  id          TEXT PRIMARY KEY,
  pipeline_id TEXT NOT NULL REFERENCES pipelines(id) ON DELETE CASCADE,
  source_id   TEXT NOT NULL REFERENCES pipeline_nodes(id) ON DELETE CASCADE,
  target_id   TEXT NOT NULL REFERENCES pipeline_nodes(id) ON DELETE CASCADE,
  edge_label  TEXT NOT NULL DEFAULT 'default'
);

CREATE INDEX IF NOT EXISTS pipeline_nodes_pipeline_id ON pipeline_nodes(pipeline_id);
CREATE INDEX IF NOT EXISTS pipeline_edges_pipeline_id ON pipeline_edges(pipeline_id);
CREATE INDEX IF NOT EXISTS pipeline_edges_source      ON pipeline_edges(source_id);
CREATE INDEX IF NOT EXISTS pipeline_edges_target      ON pipeline_edges(target_id);

-- Migrate existing linear pipeline_steps → graph nodes + edges.
-- Each pipeline gets: trigger node → action nodes (chained) → end node.
-- Trigger node config is built from the pipeline's trigger JSON.
-- Positions are laid out horizontally: x = 60 + (index * 220), y = 200.

INSERT INTO pipeline_nodes (id, pipeline_id, node_type, config, pos_x, pos_y)
SELECT
  'migrated-trigger-' || p.id,
  p.id,
  'trigger',
  json_object('trigger', json_extract(p.trigger, '$.type')),
  60,
  200
FROM pipelines p
WHERE NOT EXISTS (
  SELECT 1 FROM pipeline_nodes pn WHERE pn.pipeline_id = p.id
);

INSERT INTO pipeline_nodes (id, pipeline_id, node_type, config, pos_x, pos_y)
SELECT
  'migrated-action-' || s.id,
  s.pipeline_id,
  'action',
  json_object('tool', s.tool, 'params', json(s.config)),
  60 + (s.step_order * 220),
  200
FROM pipeline_steps s
WHERE NOT EXISTS (
  SELECT 1 FROM pipeline_nodes pn
  WHERE pn.id = 'migrated-action-' || s.id
);

INSERT INTO pipeline_nodes (id, pipeline_id, node_type, config, pos_x, pos_y)
SELECT
  'migrated-end-' || p.id,
  p.id,
  'end',
  '{}',
  60 + ((SELECT COALESCE(MAX(step_order), 0) + 1 FROM pipeline_steps WHERE pipeline_id = p.id) * 220),
  200
FROM pipelines p
WHERE NOT EXISTS (
  SELECT 1 FROM pipeline_nodes pn WHERE pn.id = 'migrated-end-' || p.id
);

-- Edges: trigger → first action (or end if no steps)
INSERT INTO pipeline_edges (id, pipeline_id, source_id, target_id, edge_label)
SELECT
  'migrated-edge-trigger-' || p.id,
  p.id,
  'migrated-trigger-' || p.id,
  COALESCE(
    (SELECT 'migrated-action-' || s.id
     FROM pipeline_steps s
     WHERE s.pipeline_id = p.id
     ORDER BY s.step_order ASC
     LIMIT 1),
    'migrated-end-' || p.id
  ),
  'default'
FROM pipelines p
WHERE NOT EXISTS (
  SELECT 1 FROM pipeline_edges pe WHERE pe.id = 'migrated-edge-trigger-' || p.id
);

-- Edges: action → next action (or end)
INSERT INTO pipeline_edges (id, pipeline_id, source_id, target_id, edge_label)
SELECT
  'migrated-edge-' || s.id,
  s.pipeline_id,
  'migrated-action-' || s.id,
  COALESCE(
    (SELECT 'migrated-action-' || s2.id
     FROM pipeline_steps s2
     WHERE s2.pipeline_id = s.pipeline_id
       AND s2.step_order = s.step_order + 1
     LIMIT 1),
    'migrated-end-' || s.pipeline_id
  ),
  'default'
FROM pipeline_steps s
WHERE NOT EXISTS (
  SELECT 1 FROM pipeline_edges pe WHERE pe.id = 'migrated-edge-' || s.id
);
