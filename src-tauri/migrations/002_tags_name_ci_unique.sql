CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_name_ci ON tags(lower(name));

CREATE INDEX IF NOT EXISTS idx_project_tags_tag ON project_tags(tag_id);
