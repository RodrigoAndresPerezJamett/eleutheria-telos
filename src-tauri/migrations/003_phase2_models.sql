-- Phase 2: add url column to models table for download tracking.
-- Size values are approximate for display; actual file size verified on download.
ALTER TABLE models ADD COLUMN url TEXT;

-- Seed the model catalog. INSERT OR IGNORE so re-runs are safe.
-- Whisper models: ggml format from ggerganov/whisper.cpp (HuggingFace)
-- Argos models:   url is NULL — argostranslate Python manages its own package index.
INSERT OR IGNORE INTO models (id, name, tool, size_bytes, url, downloaded) VALUES
  ('whisper-tiny',   'Whisper Tiny (~39 MB)',    'voice',      39000000, 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin',   0),
  ('whisper-base',   'Whisper Base (~74 MB)',    'voice',      74000000, 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin',   0),
  ('whisper-small',  'Whisper Small (~244 MB)',  'voice',     244000000, 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin',  0),
  ('whisper-medium', 'Whisper Medium (~769 MB)', 'voice',     769000000, 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin', 0),
  ('argos-en-es',    'Translate EN → ES',        'translate',      NULL, NULL, 0),
  ('argos-es-en',    'Translate ES → EN',        'translate',      NULL, NULL, 0),
  ('argos-en-fr',    'Translate EN → FR',        'translate',      NULL, NULL, 0),
  ('argos-fr-en',    'Translate FR → EN',        'translate',      NULL, NULL, 0),
  ('argos-en-de',    'Translate EN → DE',        'translate',      NULL, NULL, 0),
  ('argos-de-en',    'Translate DE → EN',        'translate',      NULL, NULL, 0),
  ('argos-en-pt',    'Translate EN → PT',        'translate',      NULL, NULL, 0),
  ('argos-pt-en',    'Translate PT → EN',        'translate',      NULL, NULL, 0);
