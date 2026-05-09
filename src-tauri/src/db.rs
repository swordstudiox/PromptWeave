use rusqlite::{params, Connection};
use std::path::Path;

use crate::imports::PromptTemplateDraft;

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

pub fn insert_prompt_templates(database_path: &Path, items: &[PromptTemplateDraft]) -> Result<usize, String> {
    let mut connection = Connection::open(database_path)
        .map_err(|err| format!("Failed to open database {}: {err}", database_path.display()))?;
    let transaction = connection
        .transaction()
        .map_err(|err| format!("Failed to start import transaction: {err}"))?;
    let mut inserted = 0usize;

    {
        let mut statement = transaction
            .prepare(
                r#"
                INSERT OR IGNORE INTO prompt_templates (
                  id,
                  title,
                  category,
                  source_repo,
                  source_url,
                  source_license,
                  author,
                  model_hint,
                  language,
                  prompt_original,
                  prompt_zh,
                  prompt_en,
                  negative_prompt,
                  aspect_ratio,
                  tags_json,
                  preview_image_urls_json,
                  imported_at,
                  content_hash
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
                "#,
            )
            .map_err(|err| format!("Failed to prepare import statement: {err}"))?;

        for item in items {
            let tags_json = serde_json::to_string(&item.tags).map_err(|err| format!("Failed to serialize tags: {err}"))?;
            let preview_image_urls_json = serde_json::to_string(&item.preview_image_urls)
                .map_err(|err| format!("Failed to serialize preview URLs: {err}"))?;
            let changed = statement
                .execute(params![
                    &item.id,
                    &item.title,
                    &item.category,
                    &item.source_repo,
                    &item.source_url,
                    &item.source_license,
                    &item.author,
                    &item.model_hint,
                    &item.language,
                    &item.prompt_original,
                    &item.prompt_zh,
                    &item.prompt_en,
                    &item.negative_prompt,
                    &item.aspect_ratio,
                    &tags_json,
                    &preview_image_urls_json,
                    &item.imported_at,
                    &item.content_hash,
                ])
                .map_err(|err| format!("Failed to insert prompt template '{}': {err}", item.title))?;
            inserted += changed;
        }
    }

    transaction
        .commit()
        .map_err(|err| format!("Failed to commit import transaction: {err}"))?;
    Ok(inserted)
}
