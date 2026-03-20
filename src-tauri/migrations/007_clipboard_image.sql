-- Add image thumbnail storage to clipboard
-- image_thumb stores a base64-encoded PNG thumbnail (≤120×90) for image clipboard entries
-- NULL for text entries
ALTER TABLE clipboard ADD COLUMN image_thumb TEXT;
