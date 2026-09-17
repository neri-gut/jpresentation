//! User event templates. System templates are not stored here.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::template::{
    system_templates, validate_parts, validate_template_name, EventTemplate, TemplateKind,
    TemplateSource,
};
use crate::domain::week::MeetingPart;
use crate::error::AppError;

/// SQLite persistence for per-profile user templates.
pub struct SqliteTemplateStore<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteTemplateStore<'a> {
    /// Borrow the process connection.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// System templates first, then this profile's user templates.
    pub fn list(&self, profile_id: &str, langwritten: &str) -> Result<Vec<EventTemplate>, AppError> {
        let mut out = system_templates(langwritten);
        let mut stmt = self.conn.prepare(
            "SELECT id, name, kind, payload_json FROM event_templates
             WHERE profile_id = ?1 ORDER BY name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([profile_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        for row in rows {
            let (id, name, kind, payload) = row?;
            out.push(decode_user(&id, &name, &kind, &payload)?);
        }
        Ok(out)
    }

    /// Loads one user template. System ids are resolved in `langwritten`.
    pub fn get(
        &self,
        profile_id: &str,
        id: &str,
        langwritten: &str,
    ) -> Result<Option<EventTemplate>, AppError> {
        if id.starts_with("sys:") {
            return Ok(crate::domain::template::system_template(id, langwritten));
        }
        let row: Option<(String, String, String)> = self
            .conn
            .query_row(
                "SELECT name, kind, payload_json FROM event_templates
                 WHERE id = ?1 AND profile_id = ?2",
                params![id, profile_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        match row {
            Some((name, kind, payload)) => Ok(Some(decode_user(id, &name, &kind, &payload)?)),
            None => Ok(None),
        }
    }

    /// Inserts or updates a user template. System ids are rejected.
    pub fn save(
        &self,
        profile_id: &str,
        id: Option<&str>,
        name: &str,
        kind: TemplateKind,
        parts: Vec<MeetingPart>,
    ) -> Result<EventTemplate, AppError> {
        validate_template_name(name)?;
        validate_parts(&parts)?;
        if let Some(existing) = id {
            if existing.starts_with("sys:") {
                return Err(AppError::Invariant("system templates cannot be overwritten".into()));
            }
        }
        let payload = serde_json::to_string(&parts)
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        let kind_sql = kind_sql(kind);
        let stamp = crate::db::now_stamp();
        let name = name.trim();
        if let Some(existing) = id {
            let n = self.conn.execute(
                "UPDATE event_templates SET name = ?1, kind = ?2, payload_json = ?3, updated_at = ?4
                 WHERE id = ?5 AND profile_id = ?6",
                params![name, kind_sql, payload, stamp, existing, profile_id],
            )?;
            if n == 0 {
                return Err(AppError::NotFound);
            }
            return Ok(EventTemplate {
                id: existing.to_string(),
                name: name.to_string(),
                source: TemplateSource::User,
                kind,
                parts,
            });
        }
        let new_id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO event_templates (id, profile_id, name, kind, payload_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![new_id, profile_id, name, kind_sql, payload, stamp],
        )?;
        Ok(EventTemplate {
            id: new_id,
            name: name.to_string(),
            source: TemplateSource::User,
            kind,
            parts,
        })
    }

    /// Deletes a user template. System ids are rejected.
    pub fn delete(&self, profile_id: &str, id: &str) -> Result<(), AppError> {
        if id.starts_with("sys:") {
            return Err(AppError::Invariant("system templates cannot be deleted".into()));
        }
        let n = self.conn.execute(
            "DELETE FROM event_templates WHERE id = ?1 AND profile_id = ?2",
            params![id, profile_id],
        )?;
        if n == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}

fn kind_sql(kind: TemplateKind) -> &'static str {
    match kind {
        TemplateKind::Midweek => "midweek",
        TemplateKind::Weekend => "weekend",
        TemplateKind::Event => "event",
    }
}

fn decode_user(
    id: &str,
    name: &str,
    kind: &str,
    payload: &str,
) -> Result<EventTemplate, AppError> {
    let parts: Vec<MeetingPart> =
        serde_json::from_str(payload).map_err(|e| AppError::Invariant(e.to_string()))?;
    Ok(EventTemplate {
        id: id.to_string(),
        name: name.to_string(),
        source: TemplateSource::User,
        kind: TemplateKind::parse(kind)?,
        parts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{configure_connection, migrate, SqliteProfileStore};
    use crate::domain::profile::{CreateProfileDto, ProfileStore};
    use crate::domain::week::MeetingPart;

    fn memory() -> (Connection, String) {
        let conn = Connection::open_in_memory().expect("memory");
        configure_connection(&conn).expect("pragma");
        migrate(&conn).expect("migrate");
        let profile = SqliteProfileStore::new(&conn)
            .create(CreateProfileDto {
                name: "Cong".into(),
            })
            .expect("create");
        (conn, profile.id.0)
    }

    fn sample_part() -> MeetingPart {
        MeetingPart {
            id: "p1".into(),
            title: "Memorial talk".into(),
            minutes: Some(45),
            tone: "other".into(),
            items: Vec::new(),
        }
    }

    #[test]
    fn list_includes_system_and_user() {
        let (conn, profile_id) = memory();
        let store = SqliteTemplateStore::new(&conn);
        store
            .save(&profile_id, None, "Memorial", TemplateKind::Event, vec![sample_part()])
            .expect("save");
        let list = store.list(&profile_id, "E").expect("list");
        assert!(list.iter().any(|t| t.id == "sys:midweek"));
        assert!(list.iter().any(|t| t.name == "Memorial" && t.source == TemplateSource::User));
    }

    #[test]
    fn cannot_delete_system() {
        let (conn, profile_id) = memory();
        let store = SqliteTemplateStore::new(&conn);
        let err = store.delete(&profile_id, "sys:midweek").expect_err("sys");
        assert!(matches!(err, AppError::Invariant(_)));
    }
}
