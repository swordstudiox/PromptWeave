use rusqlite::Connection;
use std::path::Path;

pub fn bootstrap(database_path: &Path) -> Result<(), String> {
    let connection = Connection::open(database_path)
        .map_err(|err| format!("Failed to open database {}: {err}", database_path.display()))?;

    connection
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS prompt_templates (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              category TEXT NOT NULL DEFAULT '',
              source_repo TEXT NOT NULL DEFAULT '',
              source_url TEXT NOT NULL DEFAULT '',
              source_license TEXT,
              author TEXT,
              model_hint TEXT NOT NULL DEFAULT 'generic',
              language TEXT NOT NULL DEFAULT 'mixed',
              prompt_original TEXT NOT NULL,
              prompt_zh TEXT,
              prompt_en TEXT,
              negative_prompt TEXT,
              aspect_ratio TEXT,
              tags_json TEXT NOT NULL DEFAULT '[]',
              preview_image_urls_json TEXT NOT NULL DEFAULT '[]',
              imported_at TEXT NOT NULL,
              content_hash TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS prompt_templates_fts USING fts5(
              title,
              category,
              prompt_original,
              prompt_zh,
              prompt_en,
              tags_json,
              content='prompt_templates',
              content_rowid='rowid'
            );

            CREATE TABLE IF NOT EXISTS generation_history (
              id TEXT PRIMARY KEY,
              user_input TEXT NOT NULL,
              prompt_zh TEXT NOT NULL,
              prompt_en TEXT NOT NULL,
              export_format TEXT NOT NULL,
              image_path TEXT,
              created_at TEXT NOT NULL
            );
            "#,
        )
        .map_err(|err| format!("Failed to bootstrap database: {err}"))?;

    Ok(())
}
