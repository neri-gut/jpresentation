use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::db::{configure_connection, migrate};
use crate::domain::media::ProviderRegistry;
use crate::domain::output::OutputState;
use crate::error::AppError;
use crate::jobs::JobQueue;

/// Process-wide state. Rust owns the database, stage, providers, and job pool.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub output: Mutex<OutputState>,
    /// Empty until a later change registers a `MediaProvider`.
    #[allow(dead_code)]
    pub providers: Mutex<ProviderRegistry>,
    /// Idle pool; later changes enqueue index/thumbnail/download work.
    #[allow(dead_code)]
    pub jobs: JobQueue,
}

impl AppState {
    /// Opens `app_data_dir/jpresentation.db`, migrates, and seeds a Default profile if needed.
    pub fn open(app: &AppHandle) -> Result<Self, AppError> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Io(e.to_string()))?;
        std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
        Self::open_path(&dir.join("jpresentation.db"))
    }

    /// Opens a specific database file. Used by tests with a temp path.
    pub fn open_path(db_path: &Path) -> Result<Self, AppError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Io(e.to_string()))?;
        }
        let conn = Connection::open(db_path)?;
        configure_connection(&conn)?;
        migrate(&conn)?;
        Ok(Self {
            db: Mutex::new(conn),
            output: Mutex::new(OutputState::idle()),
            providers: Mutex::new(ProviderRegistry::new()),
            jobs: JobQueue::new(),
        })
    }

    /// Locks SQLite. Poison is reported as `Busy` so a panicked command cannot take down the stage.
    pub fn lock_db(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.db.lock().map_err(|_| AppError::Busy)
    }

    /// Locks the in-memory output snapshots.
    pub fn lock_output(&self) -> Result<MutexGuard<'_, OutputState>, AppError> {
        self.output.lock().map_err(|_| AppError::Busy)
    }
}
