-- Unicidad de nombres de etiqueta insensible a mayúsculas.
--
-- `tags.name` ya era UNIQUE en la migración 001, pero con la colación BINARY
-- por defecto de SQLite, así que "Cliente" y "cliente" habrían convivido como
-- etiquetas distintas. Este índice lleva la regla a la base de datos; la
-- validación en `core::tag` la repite antes de insertar solo para poder dar un
-- mensaje de error legible.
--
-- `IF NOT EXISTS` lo hace idempotente: una base que ya lo tuviera no falla.
CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_name_ci ON tags(lower(name));

-- Las consultas de etiquetas de un proyecto y de proyectos de una etiqueta
-- recorren `project_tags` por cada lado. La PRIMARY KEY (project_id, tag_id)
-- solo cubre el primero.
CREATE INDEX IF NOT EXISTS idx_project_tags_tag ON project_tags(tag_id);
