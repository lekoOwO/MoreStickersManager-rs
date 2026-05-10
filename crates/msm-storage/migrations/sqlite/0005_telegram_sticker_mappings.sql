CREATE TABLE telegram_sticker_mappings (
    id TEXT PRIMARY KEY NOT NULL,
    publication_id TEXT NOT NULL REFERENCES telegram_publications(id) ON DELETE CASCADE,
    target_id TEXT NOT NULL REFERENCES export_targets(id) ON DELETE CASCADE,
    sticker_set_name TEXT NOT NULL,
    source_sticker_id TEXT NOT NULL,
    telegram_file_id TEXT NOT NULL,
    telegram_file_unique_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (target_id, sticker_set_name, source_sticker_id)
);

CREATE INDEX idx_telegram_sticker_mappings_publication
    ON telegram_sticker_mappings(publication_id, position, source_sticker_id);
