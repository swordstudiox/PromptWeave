use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;

use crate::imports::PromptTemplateDraft;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplateRecord {
    pub id: String,
    pub title: String,
    pub category: String,
    pub source_repo: String,
    pub source_url: String,
    pub model_hint: String,
    pub language: String,
    pub prompt_original: String,
    pub negative_prompt: Option<String>,
    pub aspect_ratio: Option<String>,
    pub tags: Vec<String>,
    pub imported_at: String,
}

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

            CREATE TRIGGER IF NOT EXISTS prompt_templates_ai AFTER INSERT ON prompt_templates BEGIN
              INSERT INTO prompt_templates_fts(rowid, title, category, prompt_original, prompt_zh, prompt_en, tags_json)
              VALUES (new.rowid, new.title, new.category, new.prompt_original, new.prompt_zh, new.prompt_en, new.tags_json);
            END;

            CREATE TRIGGER IF NOT EXISTS prompt_templates_ad AFTER DELETE ON prompt_templates BEGIN
              INSERT INTO prompt_templates_fts(prompt_templates_fts, rowid, title, category, prompt_original, prompt_zh, prompt_en, tags_json)
              VALUES('delete', old.rowid, old.title, old.category, old.prompt_original, old.prompt_zh, old.prompt_en, old.tags_json);
            END;

            CREATE TRIGGER IF NOT EXISTS prompt_templates_au AFTER UPDATE ON prompt_templates BEGIN
              INSERT INTO prompt_templates_fts(prompt_templates_fts, rowid, title, category, prompt_original, prompt_zh, prompt_en, tags_json)
              VALUES('delete', old.rowid, old.title, old.category, old.prompt_original, old.prompt_zh, old.prompt_en, old.tags_json);
              INSERT INTO prompt_templates_fts(rowid, title, category, prompt_original, prompt_zh, prompt_en, tags_json)
              VALUES (new.rowid, new.title, new.category, new.prompt_original, new.prompt_zh, new.prompt_en, new.tags_json);
            END;

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

pub fn list_prompt_templates(database_path: &Path, limit: usize) -> Result<Vec<PromptTemplateRecord>, String> {
    let connection = Connection::open(database_path)
        .map_err(|err| format!("Failed to open database {}: {err}", database_path.display()))?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, title, category, source_repo, source_url, model_hint, language,
                   prompt_original, negative_prompt, aspect_ratio, tags_json, imported_at
            FROM prompt_templates
            ORDER BY imported_at DESC, title ASC
            LIMIT ?1
            "#,
        )
        .map_err(|err| format!("Failed to prepare template list query: {err}"))?;

    let rows = statement
        .query(params![limit])
        .map_err(|err| format!("Failed to query templates: {err}"))?;
    collect_template_rows(rows)
}

pub fn search_prompt_templates(database_path: &Path, query: &str, limit: usize) -> Result<Vec<PromptTemplateRecord>, String> {
    if query.trim().is_empty() {
        return list_prompt_templates(database_path, limit);
    }

    let connection = Connection::open(database_path)
        .map_err(|err| format!("Failed to open database {}: {err}", database_path.display()))?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT t.id, t.title, t.category, t.source_repo, t.source_url, t.model_hint, t.language,
                   t.prompt_original, t.negative_prompt, t.aspect_ratio, t.tags_json, t.imported_at
            FROM prompt_templates_fts f
            JOIN prompt_templates t ON t.rowid = f.rowid
            WHERE prompt_templates_fts MATCH ?1
            ORDER BY rank
            LIMIT ?2
            "#,
        )
        .map_err(|err| format!("Failed to prepare template search query: {err}"))?;

    let fts_query = sanitize_fts_query(query);
    let rows = statement
        .query(params![fts_query, limit])
        .map_err(|err| format!("Failed to search templates: {err}"))?;
    collect_template_rows(rows)
}

fn collect_template_rows(mut rows: rusqlite::Rows<'_>) -> Result<Vec<PromptTemplateRecord>, String> {
    let mut templates = Vec::new();
    while let Some(row) = rows.next().map_err(|err| format!("Failed to read template row: {err}"))? {
        let tags_json: String = row.get(10).map_err(|err| format!("Failed to read tags JSON: {err}"))?;
        let tags = serde_json::from_str(&tags_json).unwrap_or_default();
        templates.push(PromptTemplateRecord {
            id: row.get(0).map_err(|err| format!("Failed to read id: {err}"))?,
            title: row.get(1).map_err(|err| format!("Failed to read title: {err}"))?,
            category: row.get(2).map_err(|err| format!("Failed to read category: {err}"))?,
            source_repo: row.get(3).map_err(|err| format!("Failed to read source repo: {err}"))?,
            source_url: row.get(4).map_err(|err| format!("Failed to read source url: {err}"))?,
            model_hint: row.get(5).map_err(|err| format!("Failed to read model hint: {err}"))?,
            language: row.get(6).map_err(|err| format!("Failed to read language: {err}"))?,
            prompt_original: row.get(7).map_err(|err| format!("Failed to read prompt: {err}"))?,
            negative_prompt: row.get(8).map_err(|err| format!("Failed to read negative prompt: {err}"))?,
            aspect_ratio: row.get(9).map_err(|err| format!("Failed to read aspect ratio: {err}"))?,
            tags,
            imported_at: row.get(11).map_err(|err| format!("Failed to read imported_at: {err}"))?,
        });
    }
    Ok(templates)
}

fn sanitize_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|term| term.trim_matches(|char: char| !char.is_alphanumeric()))
        .filter(|term| !term.is_empty())
        .map(|term| format!("\"{term}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_database_path(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "promptweave-{name}-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        path
    }

    fn draft() -> PromptTemplateDraft {
        PromptTemplateDraft {
            id: "template-1".to_string(),
            title: "Snow Portrait".to_string(),
            category: "Portrait".to_string(),
            source_repo: "https://github.com/example/repo".to_string(),
            source_url: "https://raw.githubusercontent.com/example/repo/main/README.md".to_string(),
            source_license: None,
            author: None,
            model_hint: "gpt-image-2".to_string(),
            language: "en".to_string(),
            prompt_original: "A cinematic portrait in a snowy mountain scene.".to_string(),
            prompt_zh: None,
            prompt_en: None,
            negative_prompt: Some("watermark".to_string()),
            aspect_ratio: Some("16:9".to_string()),
            tags: vec!["portrait".to_string(), "snow".to_string()],
            preview_image_urls: Vec::new(),
            imported_at: "1".to_string(),
            content_hash: "template-1".to_string(),
        }
    }

    #[test]
    fn lists_inserted_prompt_templates() {
        let database_path = test_database_path("list");
        bootstrap(&database_path).expect("database should bootstrap");
        insert_prompt_templates(&database_path, &[draft()]).expect("template should insert");

        let templates = list_prompt_templates(&database_path, 20).expect("templates should list");

        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].title, "Snow Portrait");
        assert_eq!(templates[0].category, "Portrait");
    }

    #[test]
    fn searches_inserted_prompt_templates_with_fts() {
        let database_path = test_database_path("search");
        bootstrap(&database_path).expect("database should bootstrap");
        insert_prompt_templates(&database_path, &[draft()]).expect("template should insert");

        let templates = search_prompt_templates(&database_path, "snowy", 20).expect("templates should search");

        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].title, "Snow Portrait");
    }
}
